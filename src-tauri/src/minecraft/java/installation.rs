//! Java installation identity and executable probing.
//!
//! Mirrors `PCL.Core.Minecraft.Java.JavaInstallation` while keeping Proto's
//! cross-platform `-XshowSettings` probe and ELF/Mach-O/PE architecture reader.

use crate::utils::platform::{Architecture, Compatibility};
use anyhow::{Context, ensure};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

/// Java distribution brand, mirroring `PCL.Core.Minecraft.Java.JavaBrandType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum JavaBrandType {
    #[serde(rename = "eclipseTemurin")]
    EclipseTemurin,
    #[serde(rename = "liberica")]
    Liberica,
    #[serde(rename = "zulu")]
    Zulu,
    #[serde(rename = "corretto")]
    Corretto,
    #[serde(rename = "microsoft")]
    Microsoft,
    #[serde(rename = "ibmSemeru")]
    IBMSemeru,
    #[serde(rename = "oracle")]
    Oracle,
    #[serde(rename = "dragonwell")]
    Dragonwell,
    #[serde(rename = "tencentKona")]
    TencentKona,
    #[serde(rename = "openJdk")]
    OpenJDK,
    #[serde(rename = "graalVm")]
    GraalVmCommunity,
    #[serde(rename = "jetBrains")]
    JetBrains,
    #[default]
    #[serde(rename = "unknown")]
    Unknown,
}

impl JavaBrandType {
    /// Map a vendor string such as `java.vendor` to a known brand.
    pub fn from_vendor(vendor: &str) -> Self {
        let vendor = vendor.to_ascii_lowercase();
        let has = |needle: &str| vendor.contains(needle);
        if has("temurin") || has("adoptium") || has("eclipse") {
            Self::EclipseTemurin
        } else if has("liberica") || has("bellsoft") {
            Self::Liberica
        } else if has("zulu") || has("azul") {
            Self::Zulu
        } else if has("corretto") || has("amazon") {
            Self::Corretto
        } else if has("microsoft") {
            Self::Microsoft
        } else if has("semeru") || has("ibm") {
            Self::IBMSemeru
        } else if has("oracle") || has("sun microsystems") {
            Self::Oracle
        } else if has("dragonwell") || has("alibaba") {
            Self::Dragonwell
        } else if has("tencent") || has("kona") {
            Self::TencentKona
        } else if has("graalvm") {
            Self::GraalVmCommunity
        } else if has("jetbrains") {
            Self::JetBrains
        } else if has("openjdk") || has("hotspot") {
            Self::OpenJDK
        } else {
            Self::Unknown
        }
    }

    /// Preference order used when several runtimes satisfy the same version.
    ///
    /// Lower is preferred, matching CE's `JavaInstallation.Brand` ordering.
    pub fn rank(self) -> usize {
        match self {
            Self::EclipseTemurin => 0,
            Self::Liberica => 1,
            Self::Zulu => 2,
            Self::Corretto => 3,
            Self::Microsoft => 4,
            Self::IBMSemeru => 5,
            Self::Oracle => 6,
            Self::Dragonwell => 7,
            Self::TencentKona => 8,
            Self::OpenJDK => 9,
            Self::GraalVmCommunity => 10,
            Self::JetBrains => 11,
            Self::Unknown => 12,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::EclipseTemurin => "Eclipse Temurin",
            Self::Liberica => "BellSoft Liberica",
            Self::Zulu => "Azul Zulu",
            Self::Corretto => "Amazon Corretto",
            Self::Microsoft => "Microsoft OpenJDK",
            Self::IBMSemeru => "IBM Semeru",
            Self::Oracle => "Oracle",
            Self::Dragonwell => "Alibaba Dragonwell",
            Self::TencentKona => "Tencent Kona",
            Self::OpenJDK => "OpenJDK",
            Self::GraalVmCommunity => "GraalVM Community",
            Self::JetBrains => "JetBrains Runtime",
            Self::Unknown => "Unknown",
        }
    }
}

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
        // Vendor builds can report more than four segments (`25.0.5.0.3`); drop the tail.
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
    /// Raw `java.version`, e.g. `17.0.9` or `1.8.0_392`.
    pub version: String,
    #[serde(default)]
    pub brand: JavaBrandType,
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
            self.version,
            self.brand.label(),
            if self.is_64_bit { "64 Bit" } else { "32 Bit" },
            self.java_folder.display()
        )
    }

    /// Parse a Java executable without executing the game.
    pub fn probe(java_path: &Path) -> anyhow::Result<Self> {
        ensure!(
            java_path.is_file(),
            "Java executable does not exist: {}",
            java_path.display()
        );
        let mut command = Command::new(java_path);
        command.args(["-XshowSettings:properties", "-version"]);
        let output = command_output(command)?;
        let text = |output: &std::process::Output| {
            format!(
                "{}\n{}",
                String::from_utf8_lossy(&output.stderr),
                String::from_utf8_lossy(&output.stdout)
            )
        };
        if output.status.success()
            && let Ok(runtime) = Self::from_probe(java_path, &text(&output))
        {
            return Ok(runtime);
        }
        // Java 6 and some older vendor builds do not implement -XshowSettings.
        let mut command = Command::new(java_path);
        command.arg("-version");
        let output = command_output(command)?;
        ensure!(
            output.status.success(),
            "Java executable could not start: {}",
            java_path.display()
        );
        Self::from_probe(java_path, &text(&output))
    }

    pub(crate) fn from_probe(java_path: &Path, text: &str) -> anyhow::Result<Self> {
        let property = |name: &str| {
            text.lines().find_map(|line| {
                let (key, value) = line.trim().split_once('=')?;
                (key.trim() == name).then(|| value.trim().to_owned())
            })
        };
        let version = property("java.version")
            .or_else(|| {
                let re = regex::Regex::new(r#"(?:openjdk|java) version\s+"([^"]+)"#).unwrap();
                re.captures(text).map(|v| v[1].to_owned())
            })
            .context("Cannot identify Java version")?;
        let parsed = JavaVersion::parse(&version).context("Invalid Java version")?;
        ensure!(parsed.0 > 0, "Invalid Java major version");
        let home = property("java.home").map(PathBuf::from).map(|home| {
            // Java 8 JDKs report their nested jre as java.home; retain the JDK executable and its javac.
            if home.file_name().is_some_and(|name| name == "jre")
                && let Some(parent) = home.parent()
                && parent
                    .join("bin")
                    .join(if cfg!(windows) { "javac.exe" } else { "javac" })
                    .is_file()
            {
                return parent.to_owned();
            }
            home
        });
        let actual = home
            .as_ref()
            .map(|p| p.join("bin").join(java_binary()))
            .filter(|p| p.is_file())
            .unwrap_or_else(|| java_path.to_owned());
        let java_exe = actual
            .canonicalize()
            .with_context(|| format!("Invalid Java path: {}", actual.display()))?;
        let java_folder = java_exe
            .parent()
            .context("Missing Java directory")?
            .to_owned();
        let architecture = match property("os.arch").as_deref() {
            Some("amd64" | "x86_64" | "x64") => Architecture::X64,
            Some("aarch64" | "arm64") => Architecture::Arm64,
            Some("x86" | "i386" | "i486" | "i586" | "i686") => Architecture::X86,
            _ => Architecture::read_header(&java_exe).unwrap_or(Architecture::Unknown),
        };
        let is_64_bit = property("sun.arch.data.model")
            .map(|v| v == "64")
            .unwrap_or(matches!(
                architecture,
                Architecture::Arm64 | Architecture::X64 | Architecture::FatFile
            ));
        let compatibility = crate::utils::platform::assert_compatibility(&architecture);
        let is_jre = !java_folder
            .join(if cfg!(windows) { "javac.exe" } else { "javac" })
            .is_file();
        let brand = JavaBrandType::from_vendor(
            &property("java.vendor")
                .or_else(|| property("java.vm.vendor"))
                .unwrap_or_default(),
        );
        Ok(Self {
            java_folder,
            java_exe,
            version,
            brand,
            architecture,
            is_64_bit,
            is_jre,
            compatibility,
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

pub fn launcher_directory() -> anyhow::Result<PathBuf> {
    Ok(std::env::current_exe()?
        .parent()
        .context("Missing launcher directory")?
        .to_owned())
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
    fn brand_names_match_the_frontend_contract() {
        for (brand, name) in [
            (JavaBrandType::EclipseTemurin, "eclipseTemurin"),
            (JavaBrandType::IBMSemeru, "ibmSemeru"),
            (JavaBrandType::OpenJDK, "openJdk"),
            (JavaBrandType::GraalVmCommunity, "graalVm"),
            (JavaBrandType::Unknown, "unknown"),
        ] {
            assert_eq!(
                serde_json::to_value(brand).unwrap(),
                serde_json::json!(name)
            );
        }
    }
}
