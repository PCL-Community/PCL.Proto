//! A managed Java entry, mirroring `PCL.Core.Minecraft.Java.JavaEntry` and
//! `JavaStorageItem`.

use super::installation::{JavaInstallation, JavaVersion};
use serde::{Deserialize, Serialize};
use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

/// Where an entry came from, mirroring `PCL.Core.Minecraft.Java.JavaSource`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum JavaSource {
    #[default]
    AutoScanned,
    AutoInstalled,
    ManualAdded,
}

/// Persisted Java record, mirroring `PCL.Core.Minecraft.Java.JavaStorageItem`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaStorageItem {
    pub path: PathBuf,
    #[serde(default = "enabled_by_default")]
    pub is_enable: bool,
    #[serde(default)]
    pub source: JavaSource,
}

fn enabled_by_default() -> bool {
    true
}

/// A runtime plus its user-facing enable state and provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaEntry {
    #[serde(flatten)]
    pub installation: JavaInstallation,
    #[serde(default = "enabled_by_default")]
    pub is_enabled: bool,
    #[serde(default)]
    pub source: JavaSource,
}

impl Deref for JavaEntry {
    type Target = JavaInstallation;

    fn deref(&self) -> &Self::Target {
        &self.installation
    }
}

impl TryFrom<&str> for JavaEntry {
    type Error = anyhow::Error;

    fn try_from(path: &str) -> Result<Self, Self::Error> {
        Self::probe(Path::new(path))
    }
}

impl JavaEntry {
    /// Probe a Java executable and wrap it as a manual entry.
    pub fn probe(java_exe: &Path) -> anyhow::Result<Self> {
        Ok(Self::new(
            JavaInstallation::probe(java_exe)?,
            JavaSource::AutoScanned,
        ))
    }

    pub fn new(installation: JavaInstallation, source: JavaSource) -> Self {
        let is_enabled = installation.should_enable_by_default();
        Self {
            installation,
            is_enabled,
            source,
        }
    }

    /// Build an entry from a legacy persisted record without re-probing.
    pub fn from_installation(
        installation: JavaInstallation,
        is_enabled: bool,
        source: JavaSource,
    ) -> Self {
        Self {
            installation,
            is_enabled,
            source,
        }
    }

    pub fn java_exe(&self) -> &Path {
        &self.installation.java_exe
    }

    pub fn java_exe_path(&self) -> PathBuf {
        self.installation.java_exe.clone()
    }

    pub fn java_folder(&self) -> &Path {
        &self.installation.java_folder
    }

    pub fn version(&self) -> &str {
        &self.installation.version
    }

    pub fn parsed_version(&self) -> JavaVersion {
        self.installation.parsed_version()
    }

    pub fn major_version(&self) -> u32 {
        self.installation.major_version()
    }

    pub fn vendor(&self) -> &str {
        &self.installation.vendor
    }

    pub fn is_64_bit(&self) -> bool {
        self.installation.is_64_bit
    }

    pub fn is_jdk(&self) -> bool {
        self.installation.is_jdk()
    }

    pub fn is_still_available(&self) -> bool {
        self.installation.is_still_available()
    }

    pub fn storage_item(&self) -> JavaStorageItem {
        JavaStorageItem {
            path: self.installation.java_exe.clone(),
            is_enable: self.is_enabled,
            source: self.source,
        }
    }

    pub fn same_executable(&self, path: &Path) -> bool {
        self.installation.java_exe == path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn installation() -> JavaInstallation {
        JavaInstallation {
            java_folder: "/jdk/bin".into(),
            java_exe: "/jdk/bin/java".into(),
            version: "21.0.7".into(),
            vendor: "Eclipse Adoptium".into(),
            architecture: crate::utils::platform::Architecture::X64,
            is_64_bit: true,
            is_jre: false,
            compatibility: crate::utils::platform::Compatibility::Perfect,
        }
    }

    #[test]
    fn serializes_as_a_flat_runtime_view() {
        let entry = JavaEntry::from_installation(installation(), true, JavaSource::ManualAdded);
        let value = serde_json::to_value(&entry).unwrap();
        assert_eq!(value["java_folder"], "/jdk/bin");
        assert_eq!(value["is_jre"], false);
        assert_eq!(value["source"], "manual_added");
        assert_eq!(value["vendor"], "Eclipse Adoptium");
        assert!(value.get("installation").is_none());
    }
}
