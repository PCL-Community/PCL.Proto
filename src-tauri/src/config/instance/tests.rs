use super::*;

fn fixture(yaml: &str) -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("versions/fixture");
    fs::create_dir_all(path.join("PCL")).unwrap();
    fs::write(path.join("PCL/config.v1.yml"), yaml).unwrap();
    (root, path)
}

#[test]
fn ce_values_round_trip_and_unknown_metadata_survives() {
    let (_root, path) = fixture(
        r#"
VersionAdvanceJvm: '-Dce=true'
VersionRamType: 1
VersionRamCustom: 15
VersionRamCustomInitial: 12
VersionArgumentIndieV2: false
VersionArgumentJavaSelect: '{"kind":"relative","RelativePath":"runtime/bin/java"}'
VersionAdvanceJava: 'True'
VersionServerLoginRequire: 2
VersionServerAuthServer: 'https://example.com/authserver'
VersionServerLoginLock: true
IsStar: true
DisplayType: 4
CustomInfo: 测试描述
VersionLaunchCount: 7
VersionModpackId: '1234'
FutureSetting: { nested: [1, true, 保留] }
"#,
    );
    let config = InstanceConfig::load(&path).unwrap();
    assert!(!config.get("VersionArgumentIndieV2", true).unwrap());
    assert!(config.launch_settings().unwrap().prelaunch_wait);
    assert_eq!(config.arguments().unwrap().jvm, "-Dce=true");
    assert!(config.java().unwrap().ignore_compatibility);
    assert_eq!(
        config.java().unwrap().preference,
        JavaPreference::Relative {
            relative_path: "runtime/bin/java".into()
        }
    );
    let memory = config.memory().unwrap();
    assert_eq!(memory.custom.maximum_mib, 3072);
    assert_eq!(memory.custom.initial_mib, Some(1536));
    assert_eq!(config.metadata().unwrap().description, "测试描述");
    let unknown = config.values[Value::from("FutureSetting")].clone();
    let mut launch = config.launch_settings().unwrap();
    launch.prelaunch_command = "echo ready".into();
    InstanceConfig::update(&path, |config| {
        config.set_launch_settings(&launch)?;
        config.set_java(&InstanceJavaSettings {
            preference: JavaPreference::Existing {
                java_exe: "/runtime/bin/java".into(),
            },
            ignore_compatibility: true,
        })?;
        config.set_memory(memory.settings.as_ref(), &memory.custom)?;
        Ok(())
    })
    .unwrap();
    let saved = InstanceConfig::load(&path).unwrap();
    assert_eq!(saved.get::<u32>("VersionRamCustom", 0).unwrap(), 15);
    assert_eq!(saved.values[Value::from("FutureSetting")], unknown);
    assert_eq!(
        saved
            .get::<String>("VersionModpackId", String::new())
            .unwrap(),
        "1234"
    );
    let java: serde_json::Value = serde_json::from_str(
        &saved
            .get::<String>("VersionArgumentJavaSelect", String::new())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(java["JavaExePath"], "/runtime/bin/java");
    assert!(java.get("java_exe").is_none());
    launch.server_login_locked = false;
    let before = fs::read(path.join("PCL/config.v1.yml")).unwrap();
    assert!(InstanceConfig::update(&path, |config| config.set_launch_settings(&launch)).is_err());
    assert_eq!(fs::read(path.join("PCL/config.v1.yml")).unwrap(), before);
}

#[test]
fn invalid_files_and_failed_edits_are_never_overwritten() {
    for yaml in ["broken: [", "- sequence", "null", "VersionRamType: invalid"] {
        let (_root, path) = fixture(yaml);
        let result = InstanceConfig::update(&path, |config| {
            config.memory()?;
            config.set("IsStar", true)
        });
        assert!(result.is_err(), "{yaml}");
        assert_eq!(
            fs::read_to_string(path.join("PCL/config.v1.yml")).unwrap(),
            yaml
        );
    }
    let (_root, path) = fixture("IsStar: true\n");
    let result: Result<(), String> = InstanceConfig::update(&path, |config| {
        config.set("IsStar", false)?;
        Err("cancel edit".into())
    });
    assert!(result.is_err());
    assert!(
        InstanceConfig::load(&path)
            .unwrap()
            .metadata()
            .unwrap()
            .starred
    );
}

#[test]
fn independent_updates_survive_concurrency_and_directory_rename() {
    let (root, path) = fixture("VersionLaunchCount: 0\n");
    std::thread::scope(|scope| {
        for _ in 0..8 {
            let path = &path;
            scope.spawn(move || {
                InstanceConfig::update(path, |config| {
                    let count: u64 = config.get("VersionLaunchCount", 0)?;
                    config.set("VersionLaunchCount", count + 1)
                })
                .unwrap();
            });
        }
        let path = &path;
        scope.spawn(move || {
            InstanceConfig::update(path, |config| config.set("CustomInfo", "preserved")).unwrap()
        });
    });
    let renamed = root.path().join("versions/renamed");
    fs::rename(&path, &renamed).unwrap();
    let saved = InstanceConfig::load(&renamed).unwrap().metadata().unwrap();
    assert_eq!(saved.launch_count, 8);
    assert_eq!(saved.description, "preserved");
    assert!(InstanceConfig::load(&path).is_err());
    let other = root.path().join("versions/other");
    fs::create_dir_all(&other).unwrap();
    assert_eq!(
        InstanceConfig::load(&other)
            .unwrap()
            .metadata()
            .unwrap()
            .launch_count,
        0
    );
    assert!(!other.join("PCL").exists());
}

#[test]
fn absent_values_and_java_encodings_match_ce() {
    let (_root, path) = fixture("");
    assert!(
        InstanceConfig::load(&path)
            .unwrap()
            .launch_settings()
            .unwrap()
            .prelaunch_wait
    );
    assert!(
        InstanceConfig::load(&path)
            .unwrap()
            .memory()
            .unwrap()
            .settings
            .is_none()
    );
    for (encoded, expected) in [
        ("使用全局设置", JavaPreference::Global),
        ("", JavaPreference::Auto),
        (r#"{"kind":"global"}"#, JavaPreference::Global),
        (r#"{"kind":"auto"}"#, JavaPreference::Auto),
        (
            r#"{"kind":"exist","JAVAEXEPATH":"/java"}"#,
            JavaPreference::Existing {
                java_exe: "/java".into(),
            },
        ),
        (
            "/plain/java",
            JavaPreference::Existing {
                java_exe: "/plain/java".into(),
            },
        ),
    ] {
        InstanceConfig::update(&path, |config| {
            config.set("VersionArgumentJavaSelect", encoded)
        })
        .unwrap();
        assert_eq!(
            InstanceConfig::load(&path)
                .unwrap()
                .java()
                .unwrap()
                .preference,
            expected
        );
    }
    for (tick, mib) in [
        (0, 307),
        (12, 1536),
        (15, 3072),
        (25, 8192),
        (33, 16384),
        (49, 49152),
    ] {
        assert_eq!(memory_tick_to_mib(tick).unwrap(), mib);
        assert_eq!(memory_mib_to_tick(mib), tick);
    }
}
