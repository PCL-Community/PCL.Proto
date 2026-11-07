//! Java list management, mirroring `PCL.Core.Minecraft.Java.JavaManager`.
//!
//! Proto keeps the manager as a pure in-memory collection so the authoritative
//! persisted state stays in the host configuration. Scanning is delegated to
//! [`super::scanner`] because Proto's cross-platform discovery is stricter than
//! the Windows-only scanners in PCL.Core.

use super::{
    entry::{JavaEntry, JavaSource},
    installation::{JavaInstallation, JavaVersion},
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// Ordered collection of known Java runtimes keyed by executable path.
#[derive(Debug, Default, Clone)]
pub struct JavaManager {
    entries: BTreeMap<PathBuf, JavaEntry>,
}

#[allow(dead_code)]
impl JavaManager {
    pub fn from_entries(entries: Vec<JavaEntry>) -> Self {
        let mut manager = Self::default();
        for entry in entries {
            manager.insert(entry);
        }
        manager
    }

    pub fn into_entries(self) -> Vec<JavaEntry> {
        self.entries.into_values().collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn contains(&self, java_exe: &Path) -> bool {
        self.entries.contains_key(java_exe)
    }

    pub fn get(&self, java_exe: &Path) -> Option<&JavaEntry> {
        self.entries.get(java_exe)
    }

    pub fn get_mut(&mut self, java_exe: &Path) -> Option<&mut JavaEntry> {
        self.entries.get_mut(java_exe)
    }

    pub fn iter(&self) -> impl Iterator<Item = &JavaEntry> {
        self.entries.values()
    }

    fn insert(&mut self, entry: JavaEntry) {
        self.entries
            .insert(entry.installation.java_exe.clone(), entry);
    }

    /// Newest version first, then executable path so the order stays stable.
    pub fn sorted(&self) -> Vec<JavaEntry> {
        let mut list: Vec<_> = self.entries.values().cloned().collect();
        Self::sort(&mut list);
        list
    }

    pub fn sort(list: &mut [JavaEntry]) {
        list.sort_by(|a, b| {
            b.parsed_version()
                .cmp(&a.parsed_version())
                .then(a.installation.java_exe.cmp(&b.installation.java_exe))
        });
    }

    /// Merge a fresh scan into the known list.
    ///
    /// `before` is the snapshot taken when the scan started. Entries created
    /// while the scan was running (for example a manual import) survive, while
    /// entries that existed before the scan but are no longer reported are
    /// dropped.
    pub fn merge_scanned(&mut self, scanned: Vec<JavaEntry>, before: &[JavaEntry]) {
        let mut merged = BTreeMap::new();
        for mut entry in scanned {
            if let Some(current) = self.entries.get(entry.installation.java_exe.as_path()) {
                entry.is_enabled = current.is_enabled;
                entry.source = current.source;
            }
            merged.insert(entry.installation.java_exe.clone(), entry);
        }
        for (path, entry) in &self.entries {
            let existed_before = before
                .iter()
                .any(|known| known.installation.java_exe == *path);
            if !existed_before && !merged.contains_key(path) {
                merged.insert(path.clone(), entry.clone());
            }
        }
        self.entries = merged;
    }

    /// Parse, insert, or return the existing entry for a manually selected path.
    pub fn add_or_get(&mut self, java_exe: &Path) -> anyhow::Result<&JavaEntry> {
        let installation = JavaInstallation::probe(java_exe)?;
        let key = installation.java_exe.clone();
        if !self.entries.contains_key(&key) {
            self.entries.insert(
                key.clone(),
                JavaEntry::new(installation, JavaSource::ManualAdded),
            );
        }
        Ok(self.entries.get(&key).expect("entry was inserted"))
    }

    /// Update an entry's enable state; returns whether the path was known.
    pub fn set_enabled(&mut self, java_exe: &Path, enabled: bool) -> bool {
        match self.entries.get_mut(java_exe) {
            Some(entry) => {
                entry.is_enabled = enabled;
                true
            }
            None => false,
        }
    }

    /// Remove entries whose executable no longer exists.
    pub fn check_all_availability(&mut self) {
        self.entries.retain(|_, entry| entry.is_still_available());
    }

    /// CE `SelectSuitableJava`: enabled, still available, inside the Java range.
    pub fn select_suitable(&self, min: JavaVersion, max: JavaVersion) -> Vec<JavaEntry> {
        let mut candidates: Vec<_> = self
            .entries
            .values()
            .filter(|entry| {
                entry.is_enabled
                    && entry.is_still_available()
                    && Self::is_version_suitable(entry.parsed_version(), min, max)
            })
            .cloned()
            .collect();
        candidates.sort_by(|a, b| {
            a.major_version()
                .cmp(&b.major_version())
                .then(a.is_jdk().cmp(&b.is_jdk()))
                .then(b.parsed_version().cmp(&a.parsed_version()))
        });
        candidates
    }

    /// Normalize `1.8.0` to `8.0.0`, mirroring PCL.Core `NormalizeVersion`.
    pub fn normalize_version(version: JavaVersion) -> JavaVersion {
        if version.0 == 1 {
            JavaVersion(version.1, version.2, version.3, 0)
        } else {
            version
        }
    }

    pub fn is_version_suitable(java: JavaVersion, min: JavaVersion, max: JavaVersion) -> bool {
        let java = Self::normalize_version(java);
        let min = Self::normalize_version(min);
        let max = Self::normalize_version(max);
        java >= min && java <= max
    }

    pub fn existing_64_bit(&self) -> bool {
        self.entries.values().any(|entry| entry.is_64_bit())
    }

    pub fn exists_any(&self) -> bool {
        !self.entries.is_empty()
    }
}
