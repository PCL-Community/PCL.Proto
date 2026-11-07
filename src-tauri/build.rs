/// 对照 CE 的 Source Generator：密钥在编译期读取，改动后需要重新构建本 crate。
const SECRET_KEYS: [&str; 2] = ["CURSEFORGE_API_KEY", "MS_CLIENT_ID"];

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        prepare_windows_driver();
    }
    for key in SECRET_KEYS {
        println!("cargo:rerun-if-env-changed={key}");
    }
    println!("cargo:rerun-if-changed=../.env");
    let dotenv = std::fs::read_to_string("../.env").unwrap_or_default();
    for key in SECRET_KEYS {
        if std::env::var_os(key).is_some() {
            continue;
        }
        if let Some(value) = dotenv_value(&dotenv, key) {
            println!("cargo:rustc-env={key}={value}");
        }
    }
    let mut attributes = tauri_build::Attributes::new();
    if std::env::var("TARGET").unwrap().ends_with("windows-msvc") {
        // Common Controls v6 must also be activated for the library's test executable.
        let manifest =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("windows-app-manifest.xml");
        println!("cargo:rerun-if-changed={}", manifest.display());
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
        attributes = attributes
            .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
    }
    tauri_build::try_build(attributes).expect("failed to build Tauri resources")
}

fn prepare_windows_driver() {
    use std::path::PathBuf;
    println!("cargo:rerun-if-env-changed=WINDIVERT_DRIVER");
    let driver = std::env::var_os("WINDIVERT_DRIVER")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../target/build-tools/windivert-2.2.2/WinDivert-2.2.2-A/x64/WinDivert64.sys")
        });
    println!("cargo:rerun-if-changed={}", driver.display());
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let profile = out.ancestors().nth(3).expect("Cargo profile directory");
    for directory in [profile.to_path_buf(), profile.join("deps")] {
        std::fs::create_dir_all(&directory).expect("create native runtime directory");
        std::fs::copy(&driver, directory.join("WinDivert64.sys"))
            .expect("could not stage WinDivert64.sys; run bun run setup:windows");
    }
}

/// 从 `.env` 文本里取出一个键的值，允许两侧引号。
fn dotenv_value(contents: &str, key: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let (name, value) = line.split_once('=')?;
        (name.trim() == key).then(|| {
            let value = value.trim();
            value
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .or_else(|| {
                    value
                        .strip_prefix('\'')
                        .and_then(|value| value.strip_suffix('\''))
                })
                .unwrap_or(value)
                .to_owned()
        })
    })
}
