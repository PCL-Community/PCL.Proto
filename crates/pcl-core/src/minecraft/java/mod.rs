//! CE Java management, with executable probing and discovery for each desktop OS.
//!
//! The module layout follows `PCL.Core.Minecraft.Java`:
//! `installation`, `entry`, `manager`, scanners, parser, installer and
//! runtime requirements.

pub mod entry;
pub mod install;
pub mod installation;
pub mod manager;
pub mod requirements;
mod scanner;
#[cfg(test)]
mod tests;

#[allow(unused_imports)]
pub use crate::utils::platform::{Architecture, Compatibility};
#[allow(unused_imports)]
pub use entry::{JavaEntry, JavaSource, JavaStorageItem};
#[allow(unused_imports)]
pub use installation::{JavaInstallation, JavaVersion, java_binary};
pub use manager::JavaManager;
pub use scanner::scan;

pub fn sort_list(list: &mut [JavaEntry]) {
    JavaManager::sort(list);
}

use std::path::Path;

pub fn reprobe(path: &Path, known: &[JavaEntry]) -> Option<JavaEntry> {
    let mut actual = JavaEntry::probe(path)
        .map_err(|e| log::warn!("Java {} is unavailable: {e}", path.display()))
        .ok()?;
    if let Some(saved) = known
        .iter()
        .find(|j| j.java_exe == actual.java_exe || j.java_exe == path)
    {
        actual.is_enabled = saved.is_enabled;
        actual.source = saved.source;
    }
    (actual.is_enabled
        && !matches!(
            actual.installation.compatibility,
            crate::utils::platform::Compatibility::No
                | crate::utils::platform::Compatibility::Unknown
        ))
    .then_some(actual)
}

pub fn candidate_order(a: &JavaEntry, b: &JavaEntry) -> std::cmp::Ordering {
    let rank = |j: &JavaEntry| match j.installation.compatibility {
        crate::utils::platform::Compatibility::Perfect => 0,
        _ => 1,
    };
    // Lowest acceptable major, then JDK, newest patch and path; vendor no longer ranks.
    a.major_version()
        .cmp(&b.major_version())
        .then(rank(a).cmp(&rank(b)))
        .then(b.is_jdk().cmp(&a.is_jdk()))
        .then(b.parsed_version().cmp(&a.parsed_version()))
        .then(a.java_exe.cmp(&b.java_exe))
}
