use pcl_core::minecraft::java::{JavaEntry, requirements::JavaRequirements};
use pcl_core::minecraft::java::{candidate_order, reprobe};

use anyhow::{Context, ensure};
use pcl_core::minecraft::instance::McInstance;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum JavaPreference {
    #[default]
    Global,
    Auto,
    #[serde(rename = "exist")]
    Existing {
        java_exe: PathBuf,
    },
    Relative {
        relative_path: PathBuf,
    },
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InstanceJavaSettings {
    pub preference: JavaPreference,
    pub ignore_compatibility: bool,
}
#[derive(Debug, Serialize)]
pub struct JavaLaunchInfo {
    pub instance_directory: PathBuf,
    pub requirements: JavaRequirements,
    pub runtime: Option<JavaEntry>,
    pub warnings: Vec<JavaWarning>,
    pub download_major: Option<u32>,
}
#[derive(Debug, Serialize)]
pub struct JavaWarning {
    pub key: String,
    pub version: String,
    pub min: String,
    pub max: String,
}
impl JavaLaunchInfo {
    pub fn for_instance(
        instance: &McInstance,
        runtimes: &[JavaEntry],
        global: Option<&Path>,
        settings: &InstanceJavaSettings,
    ) -> anyhow::Result<Self> {
        let requirements = JavaRequirements::for_instance(instance)?;
        let (runtime, warnings) = select(runtimes, global, settings, &requirements);
        let download_major = requirements.download_major();
        Ok(Self {
            instance_directory: instance.directory.clone(),
            runtime,
            warnings,
            download_major,
            requirements,
        })
    }
}
pub fn current_instance<'a>(
    active: Option<&'a McInstance>,
    expected: &Path,
) -> anyhow::Result<&'a McInstance> {
    let instance = active.context("No active Minecraft instance")?;
    ensure!(
        instance.directory == expected,
        "The active Minecraft instance changed; try again"
    );
    Ok(instance)
}

pub fn relative_java(root: &Path, relative: &Path) -> anyhow::Result<PathBuf> {
    ensure!(
        !relative.is_absolute() && !relative.as_os_str().is_empty(),
        "Java path must be relative to the launcher directory"
    );
    let root = root.canonicalize()?;
    let path = root.join(relative).canonicalize()?;
    ensure!(
        path.starts_with(&root),
        "Java path is outside the launcher directory"
    );
    ensure!(path.is_file(), "Java executable not found");
    Ok(path)
}

pub fn is_game_set_64_bit(
    known: &[JavaEntry],
    global: Option<&Path>,
    preference: &JavaPreference,
) -> bool {
    let any_64_bit = || known.iter().any(|j| j.is_64_bit);
    match preference {
        JavaPreference::Auto => any_64_bit(),
        JavaPreference::Existing { java_exe } => {
            JavaEntry::probe(java_exe).is_ok_and(|j| j.is_64_bit)
        }
        JavaPreference::Relative { relative_path } => crate::config::paths::launcher_directory()
            .and_then(|root| relative_java(&root, relative_path))
            .and_then(|path| JavaEntry::probe(&path))
            .is_ok_and(|j| j.is_64_bit),
        JavaPreference::Global => global
            .and_then(|path| JavaEntry::probe(path).ok())
            .map(|j| j.is_64_bit)
            .unwrap_or_else(any_64_bit),
    }
}

pub fn select(
    known: &[JavaEntry],
    global: Option<&Path>,
    settings: &InstanceJavaSettings,
    requirements: &JavaRequirements,
) -> (Option<JavaEntry>, Vec<JavaWarning>) {
    let mut warnings = Vec::new();
    let instance_path = match &settings.preference {
        JavaPreference::Existing { java_exe } => {
            Some((java_exe.clone(), "InstanceSelectedOutOfRange"))
        }
        JavaPreference::Relative { relative_path } => crate::config::paths::launcher_directory()
            .and_then(|root| relative_java(&root, relative_path))
            .ok()
            .map(|p| (p, "RelativePathSelectedOutOfRange")),
        _ => None,
    };
    // CE falls through to the global preference when an instance choice is absent or unusable.
    for (path, warning) in instance_path
        .into_iter()
        .chain(global.map(|p| (p.to_owned(), "GlobalSelectedOutOfRange")))
    {
        if let Some(runtime) = reprobe(&path, known) {
            if !settings.ignore_compatibility && !requirements.accepts(runtime.parsed_version()) {
                warnings.push(JavaWarning {
                    key: format!("Minecraft.Launch.Java.Compatibility.{warning}"),
                    version: runtime.version.clone(),
                    min: requirements.min.to_string(),
                    max: requirements.max.to_string(),
                });
            }
            return (Some(runtime), warnings);
        }
    }
    let mut candidates = known
        .iter()
        .filter(|j| j.is_enabled)
        .filter_map(|j| reprobe(&j.java_exe, known))
        .filter(|j| requirements.accepts(j.parsed_version()))
        .collect::<Vec<_>>();
    candidates.sort_by(candidate_order);
    (candidates.into_iter().next(), warnings)
}

pub fn validate_preference(preference: &JavaPreference) -> anyhow::Result<()> {
    match preference {
        JavaPreference::Existing { java_exe } => {
            ensure!(
                java_exe.is_absolute(),
                "Java executable must use an absolute path"
            );
        }
        JavaPreference::Relative { relative_path } => {
            relative_java(&crate::config::paths::launcher_directory()?, relative_path)?;
        }
        _ => (),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pcl_core::minecraft::java::{JavaVersion, installation, java_binary};
    use std::{collections::BTreeSet, fs};
    struct TestDirectory(PathBuf);
    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("pcl-java-test-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn requirements(min: u32, max: u32) -> JavaRequirements {
        JavaRequirements {
            min: JavaVersion::major(min),
            max: JavaVersion::upper(max),
            compatible_majors: BTreeSet::new(),
            component: None,
        }
    }
    fn fake_java(root: &Path, version: &str, vendor: &str) -> JavaEntry {
        let bin = root.join("bin");
        fs::create_dir_all(&bin).unwrap();
        let path = bin.join(installation::java_binary());
        fs::write(&path, b"fixture").unwrap();
        fs::write(
            root.join("release"),
            format!(
                "JAVA_VERSION=\"{version}\"\nIMPLEMENTOR=\"{vendor}\"\nOS_ARCH=\"{}\"\n",
                std::env::consts::ARCH
            ),
        )
        .unwrap();
        fs::write(bin.join("javac"), b"fixture").unwrap();
        JavaEntry::probe(&path).unwrap()
    }

    #[cfg(unix)]
    #[test]
    fn selection_reprobes_overrides_warns_and_falls_back_from_disabled_or_missing_paths() {
        let temp = TestDirectory::new();
        let java21 = fake_java(&temp.0.join("jdk21"), "21.0.7", "Azul");
        let java25 = fake_java(&temp.0.join("jdk25"), "25.0.1", "Eclipse");
        let known = vec![java21.clone(), java25.clone()];
        assert!(super::is_game_set_64_bit(
            &known,
            Some(&java21.java_exe),
            &JavaPreference::Global
        ));
        assert!(super::is_game_set_64_bit(
            &known,
            None,
            &JavaPreference::Auto
        ));
        assert!(!super::is_game_set_64_bit(
            &known,
            None,
            &JavaPreference::Existing {
                java_exe: temp.0.join("missing")
            }
        ));
        let req = requirements(25, 999);
        let (automatic, warnings) =
            super::select(&known, None, &InstanceJavaSettings::default(), &req);
        assert_eq!(automatic.unwrap().major_version(), 25);
        assert!(warnings.is_empty());
        let (explicit, warnings) = super::select(
            &known,
            Some(&java21.java_exe),
            &InstanceJavaSettings::default(),
            &req,
        );
        assert_eq!(explicit.unwrap().major_version(), 21);
        assert_eq!(
            warnings[0].key,
            "Minecraft.Launch.Java.Compatibility.GlobalSelectedOutOfRange"
        );
        let settings = InstanceJavaSettings {
            preference: JavaPreference::Existing {
                java_exe: java25.java_exe.clone(),
            },
            ignore_compatibility: false,
        };
        assert_eq!(
            super::select(&known, Some(&java21.java_exe), &settings, &req)
                .0
                .unwrap()
                .major_version(),
            25
        );
        let mut disabled = java21.clone();
        disabled.is_enabled = false;
        assert_eq!(
            super::select(
                &[disabled, java25.clone()],
                Some(&java21.java_exe),
                &InstanceJavaSettings::default(),
                &req
            )
            .0
            .unwrap()
            .major_version(),
            25
        );
        let missing = InstanceJavaSettings {
            preference: JavaPreference::Existing {
                java_exe: temp.0.join("missing"),
            },
            ignore_compatibility: false,
        };
        assert_eq!(
            super::select(&known, None, &missing, &req)
                .0
                .unwrap()
                .major_version(),
            25
        );
        // The probed version is authoritative; there is no separate cached major
        // that could disagree with it.
        let mut relabeled = java21.clone();
        relabeled.installation.version = "25".into();
        assert!(
            super::select(&[relabeled], None, &InstanceJavaSettings::default(), &req)
                .0
                .is_none(),
            "Java 21 does not satisfy a Java 25 requirement"
        );
        let ignored = InstanceJavaSettings {
            ignore_compatibility: true,
            ..Default::default()
        };
        assert!(
            super::select(&known, Some(&java21.java_exe), &ignored, &req)
                .1
                .is_empty()
        );
        let auto = InstanceJavaSettings {
            preference: JavaPreference::Auto,
            ..Default::default()
        };
        assert_eq!(
            super::select(&known, Some(&java21.java_exe), &auto, &req)
                .0
                .unwrap()
                .major_version(),
            21,
            "CE's Auto preference still falls through to the global override"
        );
    }

    #[test]
    fn relative_java_cannot_escape_the_launcher_directory() {
        let temp = TestDirectory::new();
        let root = temp.0.join("launcher");
        let executable = root.join("runtime/bin").join(java_binary());
        fs::create_dir_all(executable.parent().unwrap()).unwrap();
        fs::write(&executable, b"java").unwrap();
        assert_eq!(
            relative_java(&root, Path::new(&format!("runtime/bin/{}", java_binary()))).unwrap(),
            executable.canonicalize().unwrap()
        );
        for path in [Path::new("../outside"), executable.as_path()] {
            assert!(relative_java(&root, path).is_err());
        }
        #[cfg(unix)]
        {
            let outside = temp.0.join("outside");
            fs::write(&outside, b"outside").unwrap();
            std::os::unix::fs::symlink(&outside, root.join("symlink")).unwrap();
            assert!(relative_java(&root, Path::new("symlink")).is_err());
        }
    }
}
