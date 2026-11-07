//! Java installation identity and executable probing.
//!
//! Mirrors `PCL.Core.Minecraft.Java.JavaInstallation`. Identity comes from files only:
//! the JDK's `release` properties plus the PE/ELF/Mach-O header of the executable, so no
//! `java` process is started and no JVM-reported value is trusted.

use crate::utils::platform::{Architecture, Compatibility};
use anyhow::{Context, ensure};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

/// Normalized Java version: `1.8.0_321` and `8.0.321` compare identically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct JavaVersion(pub u32, pub u32, pub u32, pub u32);

impl JavaVersion {
    pub const fn major(major: u32) -> Self {
        Self(major, 0, 0, 0)
    }

    pub const fn upper(major: u32) -> Self {
        Self(major, u32::MAX, u32::MAX, u32::MAX)
    }

    pub fn parse(value: &str) -> Option<Self> {
        let numeric = value.trim().split(['-', '+']).next()?;
        let parts: Vec<u32> = numeric
            .split(['.', '_'])
            .map(str::parse)
            .collect::<Result<_, _>>()
            .ok()?;
        if parts.is_empty() {
            return None;
        }
        // Vendor `JAVA_VERSION` may carry more than four segments (`25.0.5.0.3`); drop the tail.
        let n = |i| parts.get(i).copied().unwrap_or(0);
        if n(0) == 1 {
            (n(1) > 0).then_some(Self(n(1), n(2), n(3), 0))
        } else {
            Some(Self(n(0), n(1), n(2), n(3)))
        }
    }

    pub const fn major_version(&self) -> u32 {
        self.0
    }
}

impl Default for JavaVersion {
    fn default() -> Self {
        Self::major(0)
    }
}

impl std::fmt::Display for JavaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.1 == u32::MAX {
            write!(f, "{}.*", self.0)
        } else {
            write!(f, "{}.{}.{}", self.0, self.1, self.2)
        }
    }
}

/// A parsed Java runtime, mirroring `PCL.Core.Minecraft.Java.JavaInstallation`.
///
/// `java_folder` is the `bin` directory, as in PCL.Core. `java_exe` is kept
/// explicit because probing may canonicalize a symlink.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JavaInstallation {
    pub java_folder: PathBuf,
    pub java_exe: PathBuf,
    /// Raw `JAVA_VERSION`, e.g. `17.0.9` or `1.8.0_392`; empty when unidentifiable.
    pub version: String,
    /// Raw `IMPLEMENTOR` (or `java.vendor`); shown as-is, empty when unknown.
    #[serde(default)]
    pub vendor: String,
    pub architecture: Architecture,
    pub is_64_bit: bool,
    /// PCL.Core models the inverse (`IsJre`); Proto keeps the positive name.
    #[serde(default)]
    pub is_jre: bool,
    #[serde(default)]
    pub compatibility: Compatibility,
}

impl PartialEq for JavaInstallation {
    fn eq(&self, other: &Self) -> bool {
        self.java_exe == other.java_exe
    }
}

impl JavaInstallation {
    pub fn parsed_version(&self) -> JavaVersion {
        JavaVersion::parse(&self.version).unwrap_or_default()
    }

    /// PCL.Core `MajorVersion`: handles the `1.8` → `8` mapping.
    pub fn major_version(&self) -> u32 {
        self.parsed_version().major_version()
    }

    pub fn is_jdk(&self) -> bool {
        !self.is_jre
    }

    pub fn is_still_available(&self) -> bool {
        self.java_exe.is_file()
    }

    pub fn java_exe_path(&self) -> &Path {
        &self.java_exe
    }

    pub fn to_detailed_string(&self) -> String {
        format!(
            "{} {} {} {} | {}",
            if self.is_jre { "JRE" } else { "JDK" },
            if self.version.is_empty() {
                "Unknown"
            } else {
                &self.version
            },
            self.vendor,
            if self.is_64_bit { "64 Bit" } else { "32 Bit" },
            self.java_folder.display()
        )
    }

    /// Identify a Java executable, reading its files first.
    ///
    /// `bin/java` points at a home that owns a `release` file, which supplies the version, the
    /// vendor and `OS_ARCH`. A runtime without one is asked for `-version` as a fallback (that is
    /// also how a shim like macOS `/usr/bin/java` finds its real home), and still gets registered
    /// with an empty version when neither works.
    pub fn probe(java_path: &Path) -> anyhow::Result<Self> {
        ensure!(
            java_path.is_file(),
            "Java executable does not exist: {}",
            java_path.display()
        );
        let mut java_exe = crate::utils::canonicalize(java_path)
            .with_context(|| format!("Invalid Java path: {}", java_path.display()))?;
        let mut release = read_release_near(&java_exe);
        let mut probed = None;
        if release.is_none() {
            probed = probe_process(&java_exe);
            if let Some(actual) = probed
                .as_ref()
                .and_then(|probe| probe.home.as_ref())
                .map(|home| home.join("bin").join(java_binary()))
                .filter(|actual| actual.is_file())
                .and_then(|actual| crate::utils::canonicalize(actual).ok())
            {
                java_exe = actual;
                release = read_release_near(&java_exe);
            }
        }
        let (version, vendor, os_arch) = release
            .map(|release| (release.version, release.implementor, release.architecture))
            .or_else(|| probed.map(|probe| (probe.version, probe.vendor, probe.os_arch)))
            .unwrap_or_default();
        let java_folder = java_exe
            .parent()
            .context("Missing Java directory")?
            .to_owned();
        let architecture = architecture_of(&os_arch)
            .or_else(|| Architecture::read_header(&java_exe).ok())
            .unwrap_or(Architecture::Unknown);
        Ok(Self {
            is_jre: !java_folder.join(java_compiler()).is_file(),
            compatibility: crate::utils::platform::assert_compatibility(&architecture),
            is_64_bit: matches!(
                architecture,
                Architecture::Arm64 | Architecture::X64 | Architecture::FatFile
            ),
            version,
            vendor,
            architecture,
            java_folder,
            java_exe,
        })
    }

    /// CE's default enablement rule: usable JDK, matching architecture, and
    /// (for JREs) a legacy runtime that still ships `rt.jar`.
    pub fn should_enable_by_default(&self) -> bool {
        let lib_dir = self
            .java_folder
            .parent()
            .map(|parent| parent.join("lib"))
            .unwrap_or_default();
        let usable = if self.is_jre {
            lib_dir.join("rt.jar").is_file()
        } else {
            lib_dir.join("jvm.lib").is_file()
                || lib_dir.join("jvm.dll").is_file()
                || lib_dir.join("libjvm.so").is_file()
                || lib_dir.join("libjvm.dylib").is_file()
                || self.java_folder.is_dir()
        };
        !((self.is_jre && self.major_version() > 8)
            || (self.is_64_bit
                != matches!(
                    crate::utils::platform::host_architecture(),
                    "x86_64" | "aarch64"
                ))
            || !usable)
    }
}

pub fn java_binary() -> &'static str {
    if cfg!(windows) { "java.exe" } else { "java" }
}

fn java_compiler() -> &'static str {
    if cfg!(windows) { "javac.exe" } else { "javac" }
}

/// The subset of a JDK's `release` file that identifies the runtime.
struct Release {
    version: String,
    /// Raw `IMPLEMENTOR`, falling back to `IMPLEMENTOR_VERSION`, and shown as written.
    implementor: String,
    architecture: String,
}

fn read_release(home: &Path) -> Option<Release> {
    let text = std::fs::read_to_string(home.join("release")).ok()?;
    let property = |key: &str| {
        text.lines().find_map(|line| {
            let (name, value) = line.split_once('=')?;
            (name.trim() == key).then(|| value.trim().trim_matches('"').to_owned())
        })
    };
    Some(Release {
        version: property("JAVA_VERSION")?,
        implementor: property("IMPLEMENTOR")
            .or_else(|| property("IMPLEMENTOR_VERSION"))
            .unwrap_or_default(),
        architecture: property("OS_ARCH").unwrap_or_default(),
    })
}

/// Read the `release` file that identifies this `bin/java`, ignoring a corrupt one.
fn read_release_near(java_exe: &Path) -> Option<Release> {
    let home = java_exe.parent()?.parent()?;
    read_release(home)
        // Java 8 keeps a second runtime in `<jdk>/jre`, which has no release file of its own.
        .or_else(|| home.parent().and_then(read_release))
        .filter(|release| JavaVersion::parse(&release.version).is_some_and(|parsed| parsed.0 > 0))
}

/// What a JVM reports about itself, used when no usable `release` file exists.
#[derive(Default)]
struct ProcessProbe {
    version: String,
    vendor: String,
    os_arch: String,
    home: Option<PathBuf>,
}

/// Last resort for runtimes that ship no `release` file: ask the JVM what it is.
///
/// `-XshowSettings` also reports the vendor and home, but Java 6 rejects it, so a bare
/// `-version` retry covers the banner-only case.
fn probe_process(java_path: &Path) -> Option<ProcessProbe> {
    let banner = regex::Regex::new(r#"(?:openjdk|java) version\s+"([^"]+)""#).unwrap();
    for arguments in [
        &["-XshowSettings:properties", "-version"][..],
        &["-version"][..],
    ] {
        let mut command = Command::new(java_path);
        command.args(arguments);
        let Ok(output) = command_output(command) else {
            continue;
        };
        let text = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
        let property = |name: &str| {
            text.lines().find_map(|line| {
                let (key, value) = line.trim().split_once('=')?;
                (key.trim() == name).then(|| value.trim().to_owned())
            })
        };
        let Some(version) = property("java.version").or_else(|| {
            banner
                .captures(&text)
                .map(|captures| captures[1].to_owned())
        }) else {
            continue;
        };
        return Some(ProcessProbe {
            version,
            vendor: property("java.vendor")
                .or_else(|| property("java.vm.vendor"))
                .unwrap_or_default(),
            os_arch: property("os.arch").unwrap_or_default(),
            home: property("java.home").map(PathBuf::from),
        });
    }
    None
}

fn architecture_of(value: &str) -> Option<Architecture> {
    match value {
        "amd64" | "x86_64" | "x64" | "x86_amd64" => Some(Architecture::X64),
        "aarch64" | "arm64" => Some(Architecture::Arm64),
        "x86" | "i386" | "i486" | "i586" | "i686" => Some(Architecture::X86),
        _ => None,
    }
}

pub(super) fn command_output(mut command: Command) -> anyhow::Result<std::process::Output> {
    use std::io::Read;
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    // Drain both pipes while the process runs; registry scans can exceed a pipe's capacity.
    let (send, receive) = std::sync::mpsc::channel();
    for (is_stdout, mut pipe) in [
        (
            true,
            Box::new(child.stdout.take().unwrap()) as Box<dyn Read + Send>,
        ),
        (
            false,
            Box::new(child.stderr.take().unwrap()) as Box<dyn Read + Send>,
        ),
    ] {
        let send = send.clone();
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let result = pipe
                .by_ref()
                .take(1024 * 1024 + 1)
                .read_to_end(&mut bytes)
                .map(|_| bytes);
            let _ = send.send((is_stdout, result));
        });
    }
    let (mut stdout, mut stderr) = (None, None);
    let start = Instant::now();
    loop {
        while let Ok((is_stdout, result)) = receive.try_recv() {
            if is_stdout {
                stdout = Some(result);
            } else {
                stderr = Some(result);
            }
        }
        if let Some(status) = child.try_wait()?
            && stdout.is_some()
            && stderr.is_some()
        {
            let (stdout, stderr) = (stdout.unwrap()?, stderr.unwrap()?);
            ensure!(
                stdout.len() <= 1024 * 1024 && stderr.len() <= 1024 * 1024,
                "Java probe output is too large"
            );
            return Ok(std::process::Output {
                status,
                stdout,
                stderr,
            });
        }
        if start.elapsed() > Duration::from_secs(5) {
            let _ = child.kill();
            let _ = child.wait();
            anyhow::bail!("Java probe timed out");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendor_is_kept_as_written_in_the_release_file() {
        let mut values = [
            ("IMPLEMENTOR=\"Azul Systems, Inc.\"", "Azul Systems, Inc."),
            (
                "IMPLEMENTOR=\"Eclipse Adoptium\"\nIMPLEMENTOR_VERSION=\"Temurin-21\"",
                "Eclipse Adoptium",
            ),
            ("IMPLEMENTOR_VERSION=\"Semeru-21\"", "Semeru-21"),
            ("OS_ARCH=\"aarch64\"", ""),
        ]
        .map(|(property, _)| property);
        let home = std::env::temp_dir().join(format!("pcl-release-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&home).unwrap();
        for property in values.iter_mut() {
            std::fs::write(
                home.join("release"),
                format!("JAVA_VERSION=\"21.0.7\"\n{property}\n"),
            )
            .unwrap();
            let release = read_release(&home).unwrap();
            assert_eq!(release.version, "21.0.7");
            if !property.starts_with("OS_ARCH") {
                assert!(!release.implementor.is_empty(), "{property}");
            }
        }
        std::fs::remove_dir_all(&home).unwrap();
    }
}
