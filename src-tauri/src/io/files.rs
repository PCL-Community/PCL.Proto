use std::{
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use sha1::Digest;

/// Open a folder in the platform file manager.
pub fn open_directory(path: &Path) -> Result<(), String> {
    std::process::Command::new(if cfg!(windows) {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    })
    .arg(path)
    .spawn()
    .map_err(|error| error.to_string())?;
    Ok(())
}

/// Lowercase hex of a SHA-1 output; hybrid-array (sha1 0.11) no longer formats as `{:x}`.
pub fn sha1_hex(digest: impl AsRef<[u8]>) -> String {
    digest.as_ref().iter().map(|b| format!("{b:02x}")).collect()
}

pub fn check_sha1(file: &Path, given: &str) -> Result<bool, std::io::Error> {
    let file = std::fs::File::open(file)?;
    let mut reader = BufReader::new(file);
    let mut hasher = sha1::Sha1::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    let result = hasher.finalize();
    let computed = crate::io::files::sha1_hex(result);
    Ok(computed == given)
}

/// Validate names using the same portable restrictions on every supported platform.
pub fn validate_file_name(name: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        !name.is_empty()
            && name.trim() == name
            && !name.ends_with('.')
            && ![".", ".."].contains(&name)
            && !name
                .chars()
                .any(|c| c.is_control() || "\\/:*?\"<>|".contains(c)),
        "Invalid file or instance name: {name:?}"
    );
    let stem = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    anyhow::ensure!(
        !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            && !(stem.len() == 4
                && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && matches!(stem.as_bytes()[3], b'1'..=b'9')),
        "Reserved file or instance name: {name}"
    );
    Ok(())
}

/// Copy one archive entry, failing once it expands beyond `limit` bytes.
/// Archive headers are attacker-controlled, so the cap counts real bytes written,
/// not the size the archive claims.
pub fn copy_entry_bounded<R: std::io::Read>(
    entry: &mut R,
    output: &mut impl std::io::Write,
    limit: u64,
) -> std::io::Result<u64> {
    let mut limited = entry.take(limit);
    let written = std::io::copy(&mut limited, output)?;
    let mut probe = [0_u8; 1];
    if std::io::Read::read(&mut limited, &mut probe)? != 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Archive entry expands beyond the allowed size",
        ));
    }
    output.flush()?;
    Ok(written)
}

/// Remote manifest paths are relative slash-separated paths, even on Windows.
pub fn safe_join(root: &Path, relative: &str) -> anyhow::Result<PathBuf> {
    anyhow::ensure!(!relative.is_empty(), "Empty download path");
    for component in relative.split('/') {
        validate_file_name(component)?;
    }
    Ok(root.join(relative))
}

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn rejects_portable_path_traversal_and_reserved_names() {
        for value in [
            "../escape",
            "/absolute",
            "C:/escape",
            "a/../../b",
            "a\\b",
            "a//b",
            "a/./b",
            "NUL.txt",
            "con",
            "COM1",
            "a.",
            "a\n",
        ] {
            assert!(safe_join(Path::new("root"), value).is_err(), "{value:?}");
        }
        for value in [
            "org/lwjgl/lwjgl.jar",
            "1.21.1",
            "3D Shareware v1.34",
            "20w14∞",
            "自定义实例",
        ] {
            assert!(safe_join(Path::new("root"), value).is_ok(), "{value:?}");
        }
    }
}
