use super::*;
use super::{
    installation::command_output,
    preference::{InstanceJavaSettings, JavaPreference},
    requirements::JavaRequirements,
    scanner as discovery,
};
use crate::utils::platform::{OS, compatibility_for};
use serde_json::json;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

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
fn runtime(version: &str, vendor: &str, jdk: bool) -> JavaEntry {
    JavaEntry {
        installation: JavaInstallation {
            java_folder: PathBuf::new(),
            java_exe: PathBuf::from(format!("/{vendor}/{version}/{}", java_binary())),
            version: version.into(),
            vendor: vendor.into(),
            architecture: Architecture::Arm64,
            is_64_bit: true,
            is_jre: !jdk,
            compatibility: Compatibility::Perfect,
        },
        is_enabled: true,
        source: JavaSource::AutoScanned,
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

#[test]
fn versions_normalize_legacy_updates_and_ea_builds() {
    for (input, expected) in [
        ("1.8.0_321-b07", JavaVersion(8, 0, 321, 0)),
        ("8.0.321", JavaVersion(8, 0, 321, 0)),
        ("25-ea+12", JavaVersion::major(25)),
        ("21.0.7+6-LTS", JavaVersion(21, 0, 7, 0)),
        ("17.0.1.12", JavaVersion(17, 0, 1, 12)),
        ("25.0.5.0.3", JavaVersion(25, 0, 5, 0)),
    ] {
        assert_eq!(JavaVersion::parse(input), Some(expected), "{input}");
    }
    for input in ["", "unknown", "1", "a.25", "25.0.invalid"] {
        assert_eq!(JavaVersion::parse(input), None, "{input}");
    }
    assert!(JavaVersion::parse("1.8.0_321").unwrap() < JavaVersion::parse("1.8.0_452").unwrap());
}

#[test]
fn ce_game_and_loader_rules_cover_legacy_modern_and_snapshot_versions() {
    let cases = [
        ("1.5.2", json!({}), 0, 8),
        ("1.12.2", json!({}), 8, 999),
        ("1.17.1", json!({}), 16, 999),
        ("1.18.2", json!({}), 17, 999),
        ("1.20.4", json!({}), 17, 999),
        ("1.20.5", json!({}), 21, 999),
        (
            "26.3",
            json!({"javaVersion":{"majorVersion":25,"component":"java-runtime-epsilon"}}),
            25,
            999,
        ),
        (
            "24w14a",
            json!({"releaseTime":"2024-04-03T00:00:00Z"}),
            21,
            999,
        ),
        (
            "1.8.9",
            json!({"libraries":[{"name":"optifine:OptiFine:HD_U"}]}),
            8,
            8,
        ),
        (
            "1.7.2",
            json!({"libraries":[{"name":"net.minecraftforge:forge:1.7.2-10.12.2.0"}]}),
            7,
            7,
        ),
        (
            "1.7.10",
            json!({"libraries":[{"name":"net.minecraftforge:forge:1.7.10-10.13.4.1614"}]}),
            0,
            8,
        ),
        (
            "1.14.4",
            json!({"libraries":[{"name":"net.minecraftforge:forge:1.14.4-28.2.0"}]}),
            8,
            10,
        ),
        (
            "1.15.2",
            json!({"libraries":[{"name":"net.minecraftforge:forge:1.15.2-31.2.0"}]}),
            8,
            15,
        ),
        (
            "1.18.2",
            json!({"libraries":[{"name":"net.minecraftforge:forge:1.18.2-40.2.0"},{"name":"optifine:OptiFine:HD_U"}]}),
            17,
            18,
        ),
        (
            "1.16.5",
            json!({"libraries":[{"name":"net.fabricmc:fabric-loader:0.16.0"}]}),
            8,
            999,
        ),
        (
            "1.12.2",
            json!({"libraries":[{"name":"com.mumfrey:liteloader:1.12.2"}]}),
            8,
            8,
        ),
        (
            "1.12.2",
            json!({"libraries":[{"name":"com.cleanroommc:cleanroom:0.4.2"}]}),
            21,
            999,
        ),
        (
            "1.12.2",
            json!({"libraries":[{"name":"com.cleanroommc:cleanroom:0.5.0"}]}),
            25,
            999,
        ),
        (
            "1.8.9",
            json!({"libraries":[{"name":"net.labymod:labymod:4.0"}]}),
            21,
            999,
        ),
    ];
    for (version, meta, low, high) in cases {
        let req = JavaRequirements::from_metadata(&meta, version);
        assert_eq!((req.min.0, req.max.0), (low, high), "{version}: {meta}");
    }
}

#[test]
fn forge_update_limits_and_declared_major_sets_are_not_continuous_ranges() {
    let forge = JavaRequirements::from_metadata(
        &json!({"libraries":[{"name":"net.minecraftforge:forge:1.16.5-36.2.25"}]}),
        "1.16.5",
    );
    assert!(forge.accepts(JavaVersion(8, 0, 321, 0)));
    assert!(!forge.accepts(JavaVersion(8, 0, 322, 0)));
    assert_eq!(forge.download_major(), None);
    let metadata = json!({"libraries":[{"name":"net.minecraftforge:forge:old"}], "compatibleJavaMajors":[8,17,17,0,-1]});
    let req = JavaRequirements::from_metadata(&metadata, "1.12.2");
    assert!(req.accepts(JavaVersion(8, 0, 452, 0)));
    assert!(!req.accepts(JavaVersion::major(11)));
    assert!(
        !req.accepts(JavaVersion::major(17)),
        "CE intersects the old Forge upper bound before filtering by membership"
    );
    let overridden =
        JavaRequirements::from_metadata(&json!({"compatibleJavaMajors":[17,21]}), "1.5.2");
    assert!(overridden.accepts(JavaVersion::major(17)));
    assert!(!overridden.accepts(JavaVersion::major(18)));
    assert_eq!(overridden.download_major(), Some(17));
    let explicit =
        JavaRequirements::from_metadata(&json!({"javaVersion":{"majorVersion":25}}), "1.5.2");
    assert_eq!((explicit.min.0, explicit.max.0), (25, 999));
    assert_eq!(requirements(7, 7).download_major(), None);
}

#[test]
fn candidate_sorting_prefers_the_lowest_compatible_major_and_native_architecture() {
    let mut list = [
        runtime("25.0.1", "Eclipse", true),
        runtime("21.0.8", "Azul", true),
        runtime("21.0.1", "Eclipse", false),
        runtime("21.0.1", "Eclipse", true),
        runtime("21.0.7", "Eclipse", true),
    ];
    list.sort_by(preference::candidate_order);
    assert_eq!(
        list.iter()
            .map(|j| (j.version.as_str(), j.vendor(), j.is_jdk()))
            .collect::<Vec<_>>(),
        vec![
            ("21.0.8", "Azul", true),
            ("21.0.7", "Eclipse", true),
            ("21.0.1", "Eclipse", true),
            ("21.0.1", "Eclipse", false),
            ("25.0.1", "Eclipse", true)
        ]
    );
    let native = runtime("21.0.1", "Azul", true);
    let mut translated = runtime("21.0.8", "Eclipse", true);
    translated.installation.compatibility = Compatibility::Translation;
    assert!(preference::candidate_order(&native, &translated).is_lt());
}

#[test]
fn runtime_merge_preserves_disabled_state_and_imports_added_during_scan() {
    let initial = runtime("21.0.1", "Eclipse", true);
    let mut disabled = initial.clone();
    disabled.is_enabled = false;
    let imported = runtime("25.0.1", "Azul", true);
    let disappeared = runtime("17.0.1", "Azul", true);
    let mut manager =
        JavaManager::from_entries(vec![disabled, imported.clone(), disappeared.clone()]);
    manager.merge_scanned(vec![initial.clone()], &[initial.clone(), disappeared]);
    let merged = manager.sorted();
    assert_eq!(merged.len(), 2);
    assert!(merged.iter().any(|j| j.java_exe() == imported.java_exe()));
    assert!(
        !merged
            .iter()
            .find(|j| j.major_version() == 21)
            .unwrap()
            .is_enabled
    );
}

#[test]
fn platform_runtime_keys_and_compatibility_are_explicit() {
    for (os, arch, expected) in [
        ("windows", "x86_64", "windows-x64"),
        ("windows", "aarch64", "windows-arm64"),
        ("windows", "x86", "windows-x86"),
        ("macos", "x86_64", "mac-os"),
        ("macos", "aarch64", "mac-os-arm64"),
        ("linux", "x86_64", "linux"),
        ("linux", "i686", "linux-i386"),
    ] {
        assert_eq!(install::platform_key(os, arch), Some(expected));
    }
    assert_eq!(install::platform_key("linux", "aarch64"), None);
    assert!(matches!(
        compatibility_for(OS::MacOs, "aarch64", &Architecture::Arm64),
        Compatibility::Perfect
    ));
    assert!(matches!(
        compatibility_for(OS::MacOs, "aarch64", &Architecture::X64),
        Compatibility::Translation
    ));
    assert!(matches!(
        compatibility_for(OS::MacOs, "x86_64", &Architecture::X86),
        Compatibility::No
    ));
    assert!(matches!(
        compatibility_for(OS::Windows, "aarch64", &Architecture::X86),
        Compatibility::Translation
    ));
    assert!(matches!(
        compatibility_for(OS::Windows, "x86", &Architecture::X64),
        Compatibility::No
    ));
    assert!(matches!(
        compatibility_for(OS::Linux, "aarch64", &Architecture::X64),
        Compatibility::No
    ));
    assert!(matches!(
        compatibility_for(OS::Linux, "i686", &Architecture::X86),
        Compatibility::Perfect
    ));
}

#[test]
fn bounded_search_finds_bundles_sdks_and_linux_homes() {
    let temp = TestDirectory::new();
    for subdir in [
        "zulu.jdk/Contents/Home/bin",
        "jdk-25/bin",
        "runtime/java-runtime-epsilon/linux/java-runtime-epsilon/bin",
    ] {
        let path = temp.0.join(subdir);
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join(java_binary()), b"fixture").unwrap();
    }
    let excluded = temp.0.join("node_modules/jdk/bin");
    fs::create_dir_all(&excluded).unwrap();
    fs::write(excluded.join(java_binary()), b"fixture").unwrap();
    let found = discovery::discover_paths(vec![temp.0.clone(), temp.0.clone()]);
    assert_eq!(found.len(), 3);
    assert!(!found.iter().any(|p| p.starts_with(&excluded)));
}

#[test]
fn relative_and_manifest_links_cannot_escape_their_roots() {
    let temp = TestDirectory::new();
    let root = temp.0.join("launcher");
    let executable = root.join("runtime/bin").join(java_binary());
    fs::create_dir_all(executable.parent().unwrap()).unwrap();
    fs::write(&executable, b"java").unwrap();
    assert_eq!(
        preference::relative_java(&root, Path::new(&format!("runtime/bin/{}", java_binary())))
            .unwrap(),
        executable.canonicalize().unwrap()
    );
    for path in [Path::new("../outside"), executable.as_path()] {
        assert!(preference::relative_java(&root, path).is_err());
    }
    let link = root.join("legal/license");
    assert_eq!(
        install::link_target(&root, &link, "../lib/license").unwrap(),
        root.join("lib/license")
    );
    for target in ["../../escape", "/absolute", "C:\\escape"] {
        assert!(install::link_target(&root, &link, target).is_err());
    }
    #[cfg(unix)]
    {
        let outside = temp.0.join("outside");
        fs::write(&outside, b"outside").unwrap();
        std::os::unix::fs::symlink(&outside, root.join("symlink")).unwrap();
        assert!(preference::relative_java(&root, Path::new("symlink")).is_err());
        std::os::unix::fs::symlink(&root, root.join("cycle")).unwrap();
        assert_eq!(discovery::discover_paths(vec![root]).len(), 1);
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
    assert!(preference::is_game_set_64_bit(
        &known,
        Some(&java21.java_exe),
        &JavaPreference::Global
    ));
    assert!(preference::is_game_set_64_bit(
        &known,
        None,
        &JavaPreference::Auto
    ));
    assert!(!preference::is_game_set_64_bit(
        &known,
        None,
        &JavaPreference::Existing {
            java_exe: temp.0.join("missing")
        }
    ));
    let req = requirements(25, 999);
    let (automatic, warnings) =
        preference::select(&known, None, &InstanceJavaSettings::default(), &req);
    assert_eq!(automatic.unwrap().major_version(), 25);
    assert!(warnings.is_empty());
    let (explicit, warnings) = preference::select(
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
        preference::select(&known, Some(&java21.java_exe), &settings, &req)
            .0
            .unwrap()
            .major_version(),
        25
    );
    let mut disabled = java21.clone();
    disabled.is_enabled = false;
    assert_eq!(
        preference::select(
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
        preference::select(&known, None, &missing, &req)
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
        preference::select(&[relabeled], None, &InstanceJavaSettings::default(), &req)
            .0
            .is_none(),
        "Java 21 does not satisfy a Java 25 requirement"
    );
    let ignored = InstanceJavaSettings {
        ignore_compatibility: true,
        ..Default::default()
    };
    assert!(
        preference::select(&known, Some(&java21.java_exe), &ignored, &req)
            .1
            .is_empty()
    );
    let auto = InstanceJavaSettings {
        preference: JavaPreference::Auto,
        ..Default::default()
    };
    assert_eq!(
        preference::select(&known, Some(&java21.java_exe), &auto, &req)
            .0
            .unwrap()
            .major_version(),
        21,
        "CE's Auto preference still falls through to the global override"
    );
}

#[test]
fn instance_requirements_inherit_metadata_and_detect_cycles() {
    use crate::minecraft::instance::{McInstance, folder::McFolder};
    let temp = TestDirectory::new();
    let parent = temp.0.join("versions/26.3");
    let child = temp.0.join("versions/Custom");
    fs::create_dir_all(&parent).unwrap();
    fs::create_dir_all(&child).unwrap();
    fs::write(
        parent.join("26.3.json"),
        serde_json::to_vec(&json!({"id":"26.3", "javaVersion":{"majorVersion":25}})).unwrap(),
    )
    .unwrap();
    fs::write(
        child.join("Custom.json"),
        serde_json::to_vec(&json!({"id":"Custom","inheritsFrom":"26.3"})).unwrap(),
    )
    .unwrap();
    fs::write(child.join("Custom.jar"), b"fixture").unwrap();
    let instance =
        McInstance::from_version_folder(&child, &McFolder::new("test", temp.0.clone())).unwrap();
    assert_eq!(JavaRequirements::for_instance(&instance).unwrap().min.0, 25);
    fs::write(
        parent.join("26.3.json"),
        serde_json::to_vec(&json!({"id":"26.3","inheritsFrom":"Custom"})).unwrap(),
    )
    .unwrap();
    assert!(JavaRequirements::for_instance(&instance).is_err());
}

#[cfg(unix)]
mod download_tests {
    use super::*;
    use crate::io::net::http::api_client::{ApiBases, ApiProvider};
    use sha1::Digest;
    use std::{collections::HashMap, sync::Arc};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
        task::JoinSet,
    };

    struct Mirror {
        bases: ApiBases,
        task: tokio::task::JoinHandle<()>,
    }
    impl Drop for Mirror {
        fn drop(&mut self) {
            self.task.abort();
        }
    }
    async fn mirror(corrupt: bool, traversal: bool, slow: bool) -> Mirror {
        let release = format!(
            "JAVA_VERSION=\"25.0.1\"\nIMPLEMENTOR=\"OpenJDK\"\nOS_ARCH=\"{}\"\n",
            std::env::consts::ARCH
        )
        .into_bytes();
        let info = |path: &str, body: &[u8]| json!({"url": format!("https://piston-data.mojang.com/{path}"),"sha1":crate::io::files::sha1_hex(sha1::Sha1::digest(body)),"size":body.len()});
        let mut manifest = json!({"files":{
            "bin/java":{"type":"file","executable":true,"downloads":{"raw":info("files/java",b"jvm")}},
            "release":{"type":"file","downloads":{"raw":info("files/release",&release)}},
            "lib/NOTICE":{"type":"file","downloads":{"raw":info("files/notice",b"license")}},
            "legal/NOTICE":{"type":"link","target":"../lib/NOTICE"}
        }});
        if traversal {
            manifest["files"]["../escape"] = json!({"type":"directory"});
        }
        let manifest = serde_json::to_vec(&manifest).unwrap();
        let manifest_info = json!({"url":"https://piston-meta.mojang.com/manifest.json","sha1":crate::io::files::sha1_hex(sha1::Sha1::digest(&manifest)),"size":manifest.len()});
        let platform = install::platform_key(std::env::consts::OS, std::env::consts::ARCH).unwrap();
        let index = serde_json::to_vec(&json!({platform:{"java-runtime-epsilon":[{"manifest":manifest_info,"version":{"name":"25.0.1"}}]}})).unwrap();
        let mut routes = HashMap::new();
        routes.insert(
            "/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json"
                .to_owned(),
            index,
        );
        routes.insert("/manifest.json".to_owned(), manifest);
        routes.insert("/files/java".to_owned(), b"jvm".to_vec());
        routes.insert("/files/release".to_owned(), release);
        routes.insert(
            "/files/notice".to_owned(),
            if corrupt {
                b"bad-data".to_vec()
            } else {
                b"license".to_vec()
            },
        );
        let routes = Arc::new(routes);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base: &'static str =
            Box::leak(format!("http://{}", listener.local_addr().unwrap()).into_boxed_str());
        let mut bases = ApiBases::new(&ApiProvider::Official);
        bases.meta_base = base;
        let task = tokio::spawn(async move {
            let mut connections = JoinSet::new();
            loop {
                tokio::select! {
                    accepted = listener.accept() => {
                        let (mut socket,_) = accepted.unwrap(); let routes = routes.clone();
                        connections.spawn(async move {
                            let mut request = Vec::new();
                            loop { request.push(socket.read_u8().await?); if request.ends_with(b"\r\n\r\n") { break; } assert!(request.len()<8192); }
                            let request = String::from_utf8_lossy(&request);
                            let path = request.split_whitespace().nth(1).unwrap_or("");
                            if slow && path.starts_with("/files/") { tokio::time::sleep(Duration::from_secs(30)).await; }
                            let body = routes.get(path).unwrap();
                            socket.write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len()).as_bytes()).await?;
                            socket.write_all(body).await?;
                            Ok::<_,std::io::Error>(())
                        });
                    }
                    _ = connections.join_next(), if !connections.is_empty() => {}
                }
            }
        });
        Mirror { bases, task }
    }

    #[tokio::test]
    async fn installation_verifies_files_sets_permissions_and_materializes_links() {
        let temp = TestDirectory::new();
        let mirror = mirror(false, false, false).await;
        let mut req = requirements(25, 999);
        req.component = Some("java-runtime-epsilon".into());
        let runtime = install::install(
            req,
            temp.0.clone(),
            mirror.bases.clone(),
            tauri::ipc::Channel::new(|_| Ok(())),
        )
        .await
        .unwrap();
        assert_eq!(runtime.major_version(), 25);
        assert!(runtime.is_enabled);
        assert!(!runtime.is_jdk());
        assert!(runtime.java_exe.starts_with(temp.0.canonicalize().unwrap()));
        let root = runtime.java_folder.parent().unwrap();
        assert_eq!(fs::read(root.join("legal/NOTICE")).unwrap(), b"license");
        assert!(fs::read_dir(&temp.0).unwrap().all(|e| {
            !e.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".install-")
        }));
    }
    #[tokio::test]
    async fn failed_or_unsafe_manifests_preserve_existing_runtimes_and_clean_staging() {
        for (corrupt, traversal) in [(true, false), (false, true)] {
            let temp = TestDirectory::new();
            let existing = temp.0.join("existing");
            fs::create_dir(&existing).unwrap();
            fs::write(existing.join("keep"), b"keep").unwrap();
            let mirror = mirror(corrupt, traversal, false).await;
            assert!(
                install::install(
                    requirements(25, 999),
                    temp.0.clone(),
                    mirror.bases.clone(),
                    tauri::ipc::Channel::new(|_| Ok(()))
                )
                .await
                .is_err()
            );
            assert_eq!(fs::read(existing.join("keep")).unwrap(), b"keep");
            assert_eq!(fs::read_dir(&temp.0).unwrap().count(), 1);
        }
    }
    #[tokio::test]
    async fn cancelling_installation_cleans_its_private_directory() {
        let temp = TestDirectory::new();
        let mirror = mirror(false, false, true).await;
        let started = Arc::new(tokio::sync::Notify::new());
        let notify = started.clone();
        let progress = tauri::ipc::Channel::new(move |_| {
            notify.notify_one();
            Ok(())
        });
        let task = tokio::spawn(install::install(
            requirements(25, 999),
            temp.0.clone(),
            mirror.bases.clone(),
            progress,
        ));
        tokio::time::timeout(Duration::from_secs(5), started.notified())
            .await
            .unwrap();
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        assert_eq!(fs::read_dir(&temp.0).unwrap().count(), 0);
    }
}

#[test]
fn java8_nested_jre_is_recognized_as_its_parent_jdk() {
    let temp = TestDirectory::new();
    let jdk = temp.0.join("jdk8");
    let original = fake_java(&jdk, "1.8.0_452", "Azul");
    let nested = jdk.join("jre/bin").join(installation::java_binary());
    fs::create_dir_all(nested.parent().unwrap()).unwrap();
    fs::write(&nested, b"fixture").unwrap();
    let info = JavaInstallation::probe(&nested).unwrap();
    assert_eq!(
        info.parsed_version(),
        original.installation.parsed_version()
    );
    assert_eq!(info.vendor, original.installation.vendor);
    assert!(info.is_jre, "the nested runtime has no javac of its own");
}

#[test]
fn runtime_without_a_usable_release_file_is_still_registered() {
    let temp = TestDirectory::new();
    let bin = temp.0.join("jdk/bin");
    fs::create_dir_all(&bin).unwrap();
    let java = bin.join(installation::java_binary());
    fs::write(&java, b"fixture").unwrap();
    let info = JavaInstallation::probe(&java).unwrap();
    assert_eq!(info.version, "");
    assert_eq!(info.vendor, "");
    assert!(info.to_detailed_string().contains("Unknown"));
    for release in [
        "JAVA_VERSION=\"1.x\"\nIMPLEMENTOR=\"Azul Systems, Inc.\"\n",
        "IMPLEMENTOR=\"Azul Systems, Inc.\"\n",
    ] {
        fs::write(temp.0.join("jdk/release"), release).unwrap();
        let info = JavaInstallation::probe(&java).unwrap();
        assert_eq!(info.version, "", "{release:?}");
        assert_eq!(info.vendor, "", "{release:?}");
    }
}

#[cfg(unix)]
#[test]
fn fallback_reads_version_and_vendor_from_the_running_jvm() {
    use std::os::unix::fs::PermissionsExt;
    let temp = TestDirectory::new();
    let jdk = temp.0.join("jdk");
    let bin = jdk.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let java = bin.join(installation::java_binary());
    fs::write(
        &java,
        "#!/bin/sh\ncat >&2 <<'PROPERTIES'\njava.version = 1.6.0_45\njava.vendor = Sun Microsystems Inc.\nos.arch = aarch64\nPROPERTIES\n",
    )
    .unwrap();
    fs::set_permissions(&java, fs::Permissions::from_mode(0o755)).unwrap();
    let info = JavaInstallation::probe(&java).unwrap();
    assert_eq!(info.parsed_version(), JavaVersion(6, 0, 45, 0));
    assert_eq!(info.vendor, "Sun Microsystems Inc.");
    assert_eq!(info.architecture, Architecture::Arm64);
    assert!(info.is_jre);
}

#[tokio::test]
#[ignore = "Read-only probe of locally installed Java runtimes; run explicitly"]
async fn native_java_discovery_smoke() {
    let list = discovery::scan(Vec::new(), Vec::new()).await.unwrap();
    for java in &list {
        eprintln!(
            "Java {} {:?} enabled={} {}",
            java.version,
            java.architecture,
            java.is_enabled,
            java.java_exe.display()
        );
    }
    assert!(
        !list.is_empty(),
        "This smoke test expects an installed Java runtime"
    );
    for java in list {
        assert!(java.parsed_version().0 > 0);
        assert!(java.java_exe.is_file());
    }
}

#[tokio::test]
#[ignore = "Downloads and executes the official native Java 25 in a disposable directory"]
async fn official_native_java25_install_smoke() {
    let temp = TestDirectory::new();
    let req = JavaRequirements::from_metadata(
        &json!({"javaVersion":{"majorVersion":25,"component":"java-runtime-epsilon"}}),
        "26.3",
    );
    let runtime = install::install(
        req,
        temp.0.clone(),
        crate::io::net::http::api_client::ApiBases::new(
            &crate::io::net::http::api_client::ApiProvider::Official,
        ),
        tauri::ipc::Channel::new(|_| Ok(())),
    )
    .await
    .unwrap();
    assert_eq!(runtime.major_version(), 25);
    assert!(runtime.is_enabled);
    eprintln!(
        "Official Java {} {:?}, native={:?}",
        runtime.version, runtime.architecture, runtime.installation.compatibility
    );
}

#[cfg(unix)]
#[test]
fn discovery_aliases_do_not_reenable_a_disabled_runtime() {
    let temp = TestDirectory::new();
    let mut actual = fake_java(&temp.0.join("jdk"), "21.0.7", "Azul");
    let real = actual.java_exe.clone();
    let alias = temp.0.join("alias");
    std::os::unix::fs::symlink(&real, &alias).unwrap();
    actual.installation.java_exe = alias;
    actual.is_enabled = false;
    let list = discovery::probe_paths(&[actual], [real].into());
    assert_eq!(list.len(), 1);
    assert!(!list[0].is_enabled);
}

#[cfg(unix)]
#[cfg(unix)]
#[test]
fn probe_drains_large_output_and_times_out_unresponsive_processes() {
    let mut command = Command::new("/bin/sh");
    command.args([
        "-c",
        "head -c 131072 /dev/zero; head -c 131072 /dev/zero >&2",
    ]);
    let output = command_output(command).unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout.len(), 131072);
    assert_eq!(output.stderr.len(), 131072);
    let mut command = Command::new("/bin/sh");
    command.args(["-c", "exec sleep 10"]);
    assert!(
        command_output(command)
            .unwrap_err()
            .to_string()
            .contains("timed out")
    );
}
