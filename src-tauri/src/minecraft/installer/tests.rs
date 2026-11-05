use super::*;
use serde_json::json;

#[test]
fn legacy_assets_preserve_existing_files_and_reject_conflicts() {
    use sha1::Digest;
    let root = std::env::temp_dir().join(format!("pcl-assets-test-{}", uuid::Uuid::new_v4()));
    let instance = root.join("instance");
    let hash = crate::io::files::sha1_hex(sha1::Sha1::digest(b"sound"));
    let source = root.join("assets/objects").join(&hash[..2]).join(&hash);
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, b"sound").unwrap();
    let index: AssetIndex = serde_json::from_value(json!({
        "map_to_resources": true, "objects": {"sounds/a.ogg": {"hash": hash, "size": 5}}
    }))
    .unwrap();
    materialize_assets(&index, &root, &instance, "test", true).unwrap();
    materialize_assets(&index, &root, &instance, "test", true).unwrap();
    let target = instance.join("resources/sounds/a.ogg");
    fs::write(&target, b"user data").unwrap();
    assert!(materialize_assets(&index, &root, &instance, "test", true).is_err());
    assert_eq!(fs::read(&target).unwrap(), b"user data");
    #[cfg(unix)]
    {
        fs::remove_dir_all(instance.join("resources/sounds")).unwrap();
        std::os::unix::fs::symlink(&root, instance.join("resources/sounds")).unwrap();
        assert!(materialize_assets(&index, &root, &instance, "test", true).is_err());
        assert!(!root.join("a.ogg").exists());
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn duplicate_assets_are_downloaded_once_and_invalid_metadata_is_rejected() {
    let hash = "a".repeat(40);
    let index: AssetIndex = serde_json::from_value(
        json!({"objects":{"a.ogg":{"hash":hash,"size":2},"b.ogg":{"hash":hash,"size":2}}}),
    )
    .unwrap();
    let mut plan = InstallPlan::new();
    plan.assets(
        &index,
        Path::new("repo"),
        "https://resources.download.minecraft.net",
    )
    .unwrap();
    assert_eq!(plan.files.len(), 1);
    let bad: AssetIndex =
        serde_json::from_value(json!({"objects":{"../escape":{"hash":"x","size":1}}})).unwrap();
    assert!(
        plan.assets(&bad, Path::new("repo"), "https://example.org")
            .is_err()
    );
    let bad: AssetIndex =
        serde_json::from_value(json!({"objects":{"asset":{"hash":"x","size":1}}})).unwrap();
    assert!(
        plan.assets(&bad, Path::new("repo"), "https://example.org")
            .is_err()
    );
    let mut conflict = plan.files[0].info.clone();
    conflict.size = 3;
    assert!(
        plan.add(3, &conflict, plan.files[0].out_path.clone())
            .is_err()
    );
}

#[test]
fn inherited_native_libraries_prefer_child_version() {
    let download = |name| json!({"path":format!("{name}.jar"), "url":"https://example.org/native.jar", "sha1":"a".repeat(40), "size":1});
    let details: McVersion = serde_json::from_value(json!({
        "id":"test", "type":"release", "assets":"test", "mainClass":"Main",
        "downloads":{"client":download("client")}, "assetIndex":{"id":"test", "sha1":"a".repeat(40), "size":1,"url":"https://example.org/index"},
        "libraries":[
            {"name":"org.lwjgl:lwjgl:3.3", "natives":{"linux":"natives-linux"}, "downloads":{"classifiers":{"natives-linux":download("child")}}},
            {"name":"org.lwjgl:lwjgl:3.2", "natives":{"linux":"natives-linux"}, "downloads":{"classifiers":{"natives-linux":download("parent")}}}
        ]
    })).unwrap();
    let context = McRuleContext {
        os: "linux".into(),
        arch: "x86_64".into(),
        version: String::new(),
        features: Default::default(),
    };
    let mut plan = InstallPlan::new();
    plan.libraries(&details, Path::new("repo"), &context, 2)
        .unwrap();
    assert_eq!(plan.natives.len(), 1);
    assert_eq!(plan.natives[0].path, Path::new("repo/libraries/child.jar"));
    assert_eq!(plan.files.len(), 1);
}

#[test]
fn embedded_library_without_url_must_be_present_and_verified() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("libraries/example/loader/1/loader-1.jar");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"embedded").unwrap();
    let hash = crate::io::files::sha1_hex(sha1::Sha1::digest(b"embedded"));
    let download = json!({"sha1":"a".repeat(40),"size":1,"url":"https://example.org/file"});
    let details: McVersion = serde_json::from_value(json!({
        "id":"test", "type":"release", "assets":"test", "mainClass":"Main",
        "downloads":{"client":download}, "assetIndex":{"id":"test","sha1":"a".repeat(40),"size":1,"url":"https://example.org/index"},
        "libraries":[{"name":"example:loader:1", "downloads":{"artifact":{
            "path":"example/loader/1/loader-1.jar", "url":"", "sha1":hash, "size":8
        }}}]
    })).unwrap();
    let plan = || InstallPlan::new().libraries(&details, root.path(), &McRuleContext::current(), 2);
    assert!(plan().is_ok());
    fs::write(&path, b"corrupt!").unwrap();
    assert!(plan().is_err());
    fs::remove_file(path).unwrap();
    assert!(plan().is_err());
}

#[test]
fn loader_metadata_inherits_missing_nested_fields() {
    let mut vanilla = json!({
        "libraries": [], "mainClass": "Vanilla",
        "downloads": {"client": {"url": "client"}, "server": {"url": "server"}},
        "logging": {"client": {"argument": "vanilla"}}
    });
    let loader = json!({
        "libraries": [], "mainClass": "Forge", "logging": {},
        "downloads": {"server": {"url": "loader"}}
    });
    merge_loader_profile(&mut vanilla, &loader).unwrap();
    assert_eq!(vanilla["downloads"]["client"]["url"], "client");
    assert_eq!(vanilla["downloads"]["server"]["url"], "loader");
    assert_eq!(vanilla["logging"]["client"]["argument"], "vanilla");
}

#[test]
fn failed_installations_clean_only_their_own_directory() {
    let root = std::env::temp_dir().join(format!("pcl-install-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    let directory = InstallDirectory::create(&root, "custom").unwrap();
    fs::write(directory.path.join("world.dat"), "keep").unwrap();
    assert!(InstallDirectory::create(&root, "custom").is_err());
    assert_eq!(
        fs::read_to_string(directory.path.join("world.dat")).unwrap(),
        "keep"
    );
    drop(directory);
    assert!(!root.join("versions/custom").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn fabric_api_is_staged_inside_the_instance_mods_folder() {
    let install: LoaderInstall = serde_json::from_value(json!({
        "family": "fabric",
        "version": "0.16",
        "api": {
            "url": "https://cdn.modrinth.com/data/P7dR8mSH/versions/test/fabric-api.jar",
            "size": 1,
            "sha1": "a".repeat(40),
            "filename": "fabric-api.jar"
        }
    }))
    .unwrap();
    let api = install.api.unwrap();
    assert_eq!(
        api.file.url,
        "https://cdn.modrinth.com/data/P7dR8mSH/versions/test/fabric-api.jar"
    );
    assert_eq!(api.filename, "fabric-api.jar");

    let file = DownloadFile {
        url: "https://cdn.modrinth.com/data/P7dR8mSH/versions/test/fabric-api.jar".into(),
        size: 1,
        sha1: "a".repeat(40),
        path: None,
    };
    let mut plan = InstallPlan::new();
    let instance = Path::new("/minecraft/versions/Fabric");
    add_fabric_api(
        &mut plan,
        instance,
        &FabricApiInstall {
            file: file.clone(),
            filename: "fabric-api.jar".into(),
        },
    )
    .unwrap();
    assert_eq!(plan.files[0].out_path, instance.join("mods/fabric-api.jar"));
    assert_eq!(plan.stages[&8], [1]);
    assert!(
        add_fabric_api(
            &mut plan,
            instance,
            &FabricApiInstall {
                file,
                filename: "../fabric-api.jar".into(),
            },
        )
        .is_err()
    );
}

#[tokio::test]
#[ignore = "Downloads an official Minecraft installation; run explicitly"]
async fn official_vanilla_install_with_custom_name() {
    let root = std::env::temp_dir().join(format!("pcl-install-smoke-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    let client = MinecraftApiClient::new(
        reqwest::Client::new(),
        &crate::io::net::http::api_client::ApiProvider::Official,
    );
    let result = install_version(
        &client,
        &root,
        "1.2.5",
        "Custom Vanilla",
        None,
        0,
        &Channel::new(|_| Ok(())),
    )
    .await;
    if result.is_ok() {
        let repo = crate::minecraft::instance::folder::McFolder::new("test", root.clone());
        let instances = repo.game_instances();
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].name, "Custom Vanilla");
        assert_eq!(instances[0].version, "1.2.5");
        assert!(instances[0].jar_path.is_file());
    }
    fs::remove_dir_all(root).unwrap();
    result.unwrap();
}

use sha1::Digest;
use std::{
    io::{Read, Write},
    sync::atomic::{AtomicBool, Ordering},
};

struct MockMirror {
    url: &'static str,
    stop: Arc<AtomicBool>,
    requests: Arc<std::sync::Mutex<Vec<String>>>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl MockMirror {
    fn new(files: HashMap<String, Vec<u8>>, transient_client_error: bool) -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = Box::leak(format!("http://{}", listener.local_addr().unwrap()).into_boxed_str());
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let requests = Arc::new(std::sync::Mutex::new(Vec::new()));
        let recorded = requests.clone();
        let worker = std::thread::spawn(move || {
            let mut fail_client = transient_client_error;
            while !stopped.load(Ordering::Relaxed) {
                let (mut stream, _) = match listener.accept() {
                    Ok(connection) => connection,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(std::time::Duration::from_millis(2));
                        continue;
                    }
                    Err(error) => panic!("Mock accept failed: {error}"),
                };
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                    .unwrap();
                let mut buffer = [0; 8192];
                let count = stream.read(&mut buffer).unwrap();
                let request = String::from_utf8_lossy(&buffer[..count]);
                let path = request.split_whitespace().nth(1).unwrap().to_owned();
                recorded.lock().unwrap().push(path.clone());
                let (status, body) = if path == "/client.jar" && fail_client {
                    fail_client = false;
                    ("503 Service Unavailable", b"retry".as_slice())
                } else if let Some(body) = files.get(&path) {
                    ("200 OK", body.as_slice())
                } else {
                    ("404 Not Found", b"missing".as_slice())
                };
                write!(
                    stream,
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                )
                .unwrap();
                let _ = stream.write_all(body);
            }
        });
        Self {
            url,
            stop,
            requests,
            worker: Some(worker),
        }
    }

    fn client(&self) -> MinecraftApiClient {
        let mut bases = crate::io::net::http::api_client::ApiBases::new(
            &crate::io::net::http::api_client::ApiProvider::Official,
        );
        bases.meta_base = self.url;
        bases.fabric_base = Box::leak(format!("{}/fabric", self.url).into_boxed_str());
        bases.legacy_fabric_base =
            Box::leak(format!("{}/legacy-fabric", self.url).into_boxed_str());
        bases.resources_base = Box::leak(format!("{}/assets", self.url).into_boxed_str());
        MinecraftApiClient::with_test_bases(bases)
    }

    fn count(&self, path: &str) -> usize {
        self.requests
            .lock()
            .unwrap()
            .iter()
            .filter(|p| p.as_str() == path)
            .count()
    }
}

impl Drop for MockMirror {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let result = self.worker.take().unwrap().join();
        if !std::thread::panicking() {
            result.unwrap();
        }
    }
}

#[tokio::test]
async fn maven_libraries_are_verified_and_published_before_launch() {
    let body = b"verified fabric library";
    let hash = crate::io::files::sha1_hex(sha1::Sha1::digest(body));
    let artifact = "/maven/net/fabricmc/fabric-loader/0.16/fabric-loader-0.16.jar";
    let server = MockMirror::new(
        HashMap::from([
            (artifact.into(), body.to_vec()),
            (format!("{artifact}.sha1"), hash.as_bytes().to_vec()),
        ]),
        false,
    );
    let downloader = Downloader::with_bases(server.client().api_bases_async().await);
    let root = std::env::temp_dir().join(format!("pcl-maven-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    let download = json!({"sha1":"a".repeat(40),"size":1,"url":"https://example.org/file"});
    let mut details: McVersion = serde_json::from_value(json!({
        "id":"test", "type":"release", "assets":"test", "mainClass":"Main",
        "downloads":{"client":download}, "assetIndex":{"id":"test","sha1":"a".repeat(40),"size":1,"url":"https://example.org/index"},
        "libraries":[{"name":"net.fabricmc:fabric-loader:0.16", "url":"https://libraries.minecraft.net/"}]
    })).unwrap();
    let natives = prepare_launch_libraries(
        &details,
        &root,
        &root,
        &McRuleContext::current(),
        &downloader,
        true,
    )
    .await
    .unwrap();
    let path = root.join("libraries/net/fabricmc/fabric-loader/0.16/fabric-loader-0.16.jar");
    assert_eq!(fs::read(&path).unwrap(), body);
    assert_eq!(server.count(&format!("{artifact}.sha1")), 1);
    drop(natives);
    fs::write(&path, b"previous").unwrap();
    details.libraries[0].sha1 = Some("b".repeat(40));
    details.libraries[0].size = Some(body.len() as u64);
    assert!(
        prepare_launch_libraries(
            &details,
            &root,
            &root,
            &McRuleContext::current(),
            &downloader,
            true
        )
        .await
        .is_err()
    );
    assert_eq!(fs::read(&path).unwrap(), b"previous");
    details.libraries[0].url = Some("http://localhost/".into());
    assert!(
        prepare_launch_libraries(
            &details,
            &root,
            &root,
            &McRuleContext::current(),
            &downloader,
            true
        )
        .await
        .is_err()
    );
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
#[ignore = "Downloads official Fabric profile libraries into a temporary directory"]
async fn official_fabric_maven_libraries_smoke() {
    use crate::io::net::http::api_client::{ApiProvider, FabricFamily};
    let client = MinecraftApiClient::new(reqwest::Client::new(), &ApiProvider::Official);
    for (game, version, family) in [
        ("1.20.1", "0.16.14", FabricFamily::Fabric),
        ("1.12.2", "0.20.1", FabricFamily::Legacyfabric),
    ] {
        let profile = client
            .get_fabric_profile(game, version, family)
            .await
            .unwrap();
        let download = json!({"sha1":"a".repeat(40),"size":1,"url":"https://example.org/file"});
        let details: McVersion = serde_json::from_value(json!({
        "id":"test", "type":"release", "assets":"test", "mainClass":"Main",
        "downloads":{"client":download}, "assetIndex":{"id":"test","sha1":"a".repeat(40),"size":1,"url":"https://example.org/index"},
        "libraries":profile["libraries"]
    })).unwrap();
        let root = std::env::temp_dir().join(format!("pcl-fabric-smoke-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        let result = prepare_launch_libraries(
            &details,
            &root,
            &root,
            &McRuleContext::current(),
            &Downloader::default(),
            true,
        )
        .await;
        let valid = result.is_ok()
            && details.libraries.iter().all(|library| {
                library
                    .classpath_path(&root, &root)
                    .unwrap()
                    .is_none_or(|path| path.is_file())
            });
        let error = result.as_ref().err().map(|error| format!("{error:#}"));
        drop(result);
        fs::remove_dir_all(root).unwrap();
        assert!(
            valid,
            "{family:?} libraries could not be prepared: {error:?}"
        );
    }
}

#[cfg(unix)]
#[tokio::test]
#[ignore = "Downloads a complete official Legacy Fabric instance into a temporary directory"]
async fn official_legacy_fabric_install_smoke() {
    official_fabric_install(
        "1.12.2",
        "0.20.1",
        crate::io::net::http::api_client::FabricFamily::Legacyfabric,
        "net/legacyfabric/intermediary/1.12.2/",
    )
    .await;
}

#[cfg(unix)]
#[tokio::test]
#[ignore = "Downloads a complete official Fabric instance into a temporary directory"]
async fn official_fabric_install_smoke() {
    official_fabric_install(
        "1.20.1",
        "0.16.14",
        crate::io::net::http::api_client::FabricFamily::Fabric,
        "net/fabricmc/intermediary/1.20.1/",
    )
    .await;
}

#[cfg(unix)]
async fn official_fabric_install(
    game: &str,
    version: &str,
    family: crate::io::net::http::api_client::FabricFamily,
    intermediary: &str,
) {
    use crate::{
        io::net::http::api_client::ApiProvider,
        minecraft::{launch::launcher::LaunchOption, profile::McProfile},
    };
    use std::os::unix::fs::PermissionsExt;
    let root = std::env::temp_dir().join(format!("pcl-fabric-full-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    let result: anyhow::Result<()> = async {
        let client = MinecraftApiClient::new(reqwest::Client::new(), &ApiProvider::Official);
        let choice = LoaderInstall {
            family: match family {
                crate::io::net::http::api_client::FabricFamily::Fabric => LoaderFamily::Fabric,
                crate::io::net::http::api_client::FabricFamily::Legacyfabric => {
                    LoaderFamily::Legacyfabric
                }
            },
            version: version.into(),
            api: None,
            install_java: false,
        };
        install_version(
            &client,
            &root,
            game,
            "Fabric Test",
            Some(&choice),
            0,
            &Channel::new(|_| Ok(())),
        )
        .await?;
        let repo = crate::minecraft::instance::folder::McFolder::new("test", root.clone());
        let instance = repo
            .game_instances()
            .pop()
            .context("Installed instance not found")?;
        let executable = root.join("fake-java");
        fs::write(
            &executable,
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > captured-args\n",
        )?;
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755))?;
        let runtime = serde_json::from_value(json!({
            "java_folder":root, "version":"8", "slug_version":8,
            "is_64_bit":true, "architecture":"X64", "compatibility":"Perfect",
            "java_exe":executable, "vendor":"OpenJDK"
        }))?;
        let state = crate::app::config::AppState {
            profile: crate::app::states::ProfileConfig {
                last_used: 0,
                profiles: vec![McProfile::offline("test", "uuid")],
            },
            active_game_instance: Some(Arc::new(instance)),
            ..Default::default()
        };
        let mut option = LaunchOption::from_state(&state, Arc::new(runtime))?;
        let natives = option
            .prepare_natives(&Downloader::with_bases(client.api_bases_async().await))
            .await?;
        ensure!(
            option.launch_checked()?.wait()?.success(),
            "Fake Java failed"
        );
        let args = fs::read_to_string(root.join("versions/Fabric Test/captured-args"))?;
        ensure!(
            args.lines()
                .any(|arg| arg == "net.fabricmc.loader.impl.launch.knot.KnotClient"),
            "Missing Fabric main class"
        );
        ensure!(args.contains(intermediary), "Missing Fabric intermediary");
        drop(natives);
        Ok(())
    }
    .await;
    fs::remove_dir_all(root).unwrap();
    result.unwrap();
}

fn fixture_archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut archive = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, bytes) in entries {
        archive
            .start_file(*name, zip::write::SimpleFileOptions::default())
            .unwrap();
        archive.write_all(bytes).unwrap();
    }
    archive.finish().unwrap().into_inner()
}

#[test]
fn legacy_lwjgl_checks_actual_binary_architecture() {
    fn macho(cpu: u32) -> Vec<u8> {
        [0xfeedfacfu32, cpu, 3, 6, 0, 0, 0, 0]
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect()
    }
    let x64 = macho(goblin::mach::constants::cputype::CPU_TYPE_X86_64);
    let arm = macho(goblin::mach::constants::cputype::CPU_TYPE_ARM64);
    assert!(native_binary_matches(&x64, "osx", "x86_64").unwrap());
    assert!(!native_binary_matches(&x64, "osx", "aarch64").unwrap());
    assert!(native_binary_matches(&arm, "osx", "aarch64").unwrap());
    assert!(!native_binary_matches(&arm, "linux", "aarch64").unwrap());
    let root = std::env::temp_dir().join(format!("pcl-native-arch-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    let path = root.join("lwjgl.jar");
    fs::write(&path, fixture_archive(&[("liblwjgl.dylib", &x64)])).unwrap();
    let archive = NativeArchive {
        path: path.clone(),
        exclude: vec![],
        lwjgl_core: true,
    };
    let mut context = McRuleContext {
        os: "osx".into(),
        arch: "x86_64".into(),
        version: String::new(),
        features: Default::default(),
    };
    assert!(validate_lwjgl_natives(std::slice::from_ref(&archive), &context).is_ok());
    context.arch = "aarch64".into();
    assert!(
        validate_lwjgl_natives(std::slice::from_ref(&archive), &context)
            .unwrap_err()
            .to_string()
            .contains("does not support")
    );
    fs::write(
        &path,
        fixture_archive(&[("liblwjgl.dylib", &x64), ("liblwjgl-arm.dylib", &arm)]),
    )
    .unwrap();
    assert!(validate_lwjgl_natives(&[archive], &context).is_ok());
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
#[ignore = "Downloads the official 1.16.5 LWJGL macOS native archive"]
async fn official_legacy_lwjgl_architecture_smoke() {
    let info: DownloadFile = serde_json::from_value(json!({
        "path":"org/lwjgl/lwjgl/3.2.1/lwjgl-3.2.1-natives-macos.jar",
        "sha1":"5a4c271d150906858d475603dcb9479453c60555", "size":39835,
        "url":"https://libraries.minecraft.net/org/lwjgl/lwjgl/3.2.1/lwjgl-3.2.1-natives-macos.jar"
    }))
    .unwrap();
    let root = std::env::temp_dir().join(format!("pcl-official-native-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    let path = root.join("native.jar");
    let result = async {
        Downloader::default().download(&info, &path).await?;
        let archive = NativeArchive {
            path,
            exclude: vec![],
            lwjgl_core: true,
        };
        let mut context = McRuleContext {
            os: "osx".into(),
            arch: "x86_64".into(),
            version: String::new(),
            features: Default::default(),
        };
        validate_lwjgl_natives(std::slice::from_ref(&archive), &context)?;
        context.arch = "aarch64".into();
        assert!(validate_lwjgl_natives(&[archive], &context).is_err());
        Ok::<(), anyhow::Error>(())
    }
    .await;
    fs::remove_dir_all(root).unwrap();
    result.unwrap();
}

#[tokio::test]
#[ignore = "Downloads the official 1.2.5 LWJGL 2 native archives"]
async fn official_lwjgl2_architectures_smoke() {
    let root = std::env::temp_dir().join(format!("pcl-official-lwjgl2-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    let result = async {
        for (os, suffix, size, sha1) in [
            ("osx", "osx", 518924, "6621b382cb14cc409b041d8d72829156a87c31aa"),
            ("linux", "linux", 569061, "2ba5dcb11048147f1a74eff2deb192c001321f77"),
            ("windows", "windows", 609967, "3f11873dc8e84c854ec7c5a8fd2e869f8aaef764"),
        ] {
            let path = root.join(format!("{suffix}.jar"));
            let info = DownloadFile { path: None, size, sha1: sha1.into(), url: format!("https://libraries.minecraft.net/org/lwjgl/lwjgl/lwjgl-platform/2.9.0/lwjgl-platform-2.9.0-natives-{suffix}.jar") };
            Downloader::default().download(&info, &path).await?;
            let native = NativeArchive { path, exclude: vec![], lwjgl_core: true };
            let mut context = McRuleContext { os: os.into(), arch: "x86_64".into(), version: String::new(), features: Default::default() };
            validate_lwjgl_natives(std::slice::from_ref(&native), &context)?;
            context.arch = "x86".into();
            validate_lwjgl_natives(std::slice::from_ref(&native), &context)?;
            context.arch = "aarch64".into();
            assert!(validate_lwjgl_natives(&[native], &context).is_err());
        }
        Ok::<(), anyhow::Error>(())
    }.await;
    fs::remove_dir_all(root).unwrap();
    result.unwrap();
}

#[tokio::test]
async fn launch_preparation_selects_java_classifier_and_isolates_natives() {
    let root =
        std::env::temp_dir().join(format!("pcl-launch-native-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(root.join("libraries/test")).unwrap();
    let archive32 = fixture_archive(&[("native.bin", b"32"), ("META-INF/ignored", b"ignored")]);
    let archive64 = fixture_archive(&[("native.bin", b"64")]);
    let info = |name: &str, data: &[u8]| {
        json!({
            "path":format!("test/{name}.jar"), "url":format!("https://libraries.minecraft.net/test/{name}.jar"),
            "sha1":crate::io::files::sha1_hex(sha1::Sha1::digest(data)), "size":data.len()
        })
    };
    fs::write(root.join("libraries/test/32.jar"), &archive32).unwrap();
    fs::write(root.join("libraries/test/64.jar"), &archive64).unwrap();
    let details: McVersion = serde_json::from_value(json!({
        "id":"test", "type":"release", "assets":"test", "mainClass":"Main",
        "downloads":{"client":info("client", b"client")},
        "assetIndex":{"id":"test", "sha1":"a".repeat(40), "size":0, "url":"https://example.org/index"},
        "libraries":[{"name":"test:native:1", "natives":{"linux":"natives-linux-${arch}"},
            "downloads":{"classifiers":{"natives-linux-32":info("32", &archive32), "natives-linux-64":info("64", &archive64)}}}]
    })).unwrap();
    let context = |arch: &str| McRuleContext {
        os: "linux".into(),
        arch: arch.into(),
        version: String::new(),
        features: Default::default(),
    };
    let downloader = Downloader::default();
    let x86 = context("x86");
    let x64 = context("x86_64");
    let (a, b) = tokio::join!(
        prepare_launch_libraries(&details, &root, &root, &x86, &downloader, true),
        prepare_launch_libraries(&details, &root, &root, &x64, &downloader, true),
    );
    let (a, b) = (a.unwrap(), b.unwrap());
    assert_ne!(a, b);
    assert_eq!(fs::read(a.join("native.bin")).unwrap(), b"32");
    assert_eq!(fs::read(b.join("native.bin")).unwrap(), b"64");
    assert!(!a.join("META-INF").exists());
    fs::write(a.join("runtime-extracted"), b"runtime").unwrap();
    let same = prepare_launch_libraries(&details, &root, &root, &x86, &downloader, true)
        .await
        .unwrap();
    assert_eq!(a, same);
    fs::write(a.join("native.bin"), b"damaged").unwrap();
    let repaired = prepare_launch_libraries(&details, &root, &root, &x86, &downloader, true)
        .await
        .unwrap();
    assert_ne!(a, repaired);
    assert_eq!(fs::read(repaired.join("native.bin")).unwrap(), b"32");
    fs::remove_file(root.join("libraries/test/32.jar")).unwrap();
    let mirror = MockMirror::new([("/maven/test/32.jar".into(), archive32)].into(), false);
    let downloader = Downloader::with_bases(mirror.client().api_bases_async().await);
    let downloaded = prepare_launch_libraries(&details, &root, &root, &x86, &downloader, true)
        .await
        .unwrap();
    assert_eq!(fs::read(downloaded.join("native.bin")).unwrap(), b"32");
    assert_eq!(mirror.count("/maven/test/32.jar"), 1);
    drop(downloaded);
    let cached = prepare_launch_libraries(&details, &root, &root, &x86, &downloader, true)
        .await
        .unwrap();
    assert_eq!(mirror.count("/maven/test/32.jar"), 1);
    drop(cached);
    fs::remove_dir_all(root).unwrap();
}

fn fixture_files(bad_client_hash: bool) -> HashMap<String, Vec<u8>> {
    let hash = |bytes: &[u8]| crate::io::files::sha1_hex(sha1::Sha1::digest(bytes));
    let info = |url: &str, bytes: &[u8]| json!({"sha1":hash(bytes), "size":bytes.len(), "url":url});
    let jar = b"verified client";
    let asset = b"sound";
    let asset_hash = hash(asset);
    let natives = fixture_archive(&[
        ("libtest.dylib", b"native"),
        ("META-INF/MANIFEST.MF", b"skip"),
        ("excluded/ignored", b"skip"),
    ]);
    let index = serde_json::to_vec(&json!({"virtual":true,"map_to_resources":true,"objects":{
        "sounds/a.ogg":{"hash":asset_hash,"size":asset.len()},
        "sounds/duplicate.ogg":{"hash":asset_hash,"size":asset.len()}
    }}))
    .unwrap();
    let mut artifact = info(
        "https://libraries.minecraft.net/test/library.jar",
        b"library",
    );
    artifact["path"] = "test/library.jar".into();
    let mut native_info = info("https://libraries.minecraft.net/test/native.jar", &natives);
    native_info["path"] = "test/native.jar".into();
    let mut client = info("https://piston-data.mojang.com/client.jar", jar);
    if bad_client_hash {
        client["sha1"] = "0".repeat(40).into();
    }
    let metadata = serde_json::to_vec(&json!({
        "id":"1.test", "type":"release", "assets":"test-index", "mainClass":"net.minecraft.client.main.Main",
        "downloads":{"client":client},
        "assetIndex":{"id":"test-index", "sha1":hash(&index), "size":index.len(), "url":"https://piston-meta.mojang.com/index.json"},
        "libraries":[
            {"name":"test:library:1", "downloads":{"artifact":artifact}},
            {"name":"test:native:1", "natives":{(McRuleContext::current().os):"natives-test"}, "extract":{"exclude":["excluded/"]}, "downloads":{"classifiers":{"natives-test":native_info}}},
            {"name":"test:skip:1", "rules":[{"action":"disallow"}], "downloads":{"artifact":{"sha1":"0".repeat(40),"size":1,"url":"https://libraries.minecraft.net/forbidden.jar","path":"forbidden.jar"}}}
        ],
        "logging":{"client":{"argument":"-Dlog4j.configurationFile=${path}", "file":{"id":"client.xml","sha1":hash(b"<xml/>"),"size":6,"url":"https://piston-data.mojang.com/log.xml"}}},
        "javaVersion":{"majorVersion":21}, "arguments":{"game":["--fixture","preserved"]}
    })).unwrap();
    let manifest = serde_json::to_vec(&json!({"latest":{"release":"1.test","snapshot":"1.test"},"versions":[{
        "id":"1.test","type":"release","time":"2026-01-01T00:00:00Z","releaseTime":"2026-01-01T00:00:00Z",
        "url":"https://piston-meta.mojang.com/version.json","sha1":hash(&metadata)
    }]})).unwrap();
    [
        ("/mc/game/version_manifest_v2.json".into(), manifest),
        ("/version.json".into(), metadata),
        ("/index.json".into(), index),
        ("/client.jar".into(), jar.to_vec()),
        ("/maven/test/library.jar".into(), b"library".to_vec()),
        ("/maven/test/native.jar".into(), natives),
        ("/log.xml".into(), b"<xml/>".to_vec()),
        (
            format!("/assets/{}/{}", &asset_hash[..2], asset_hash),
            asset.to_vec(),
        ),
    ]
    .into()
}

#[tokio::test]
async fn fabric_install_merges_profile_and_publishes_only_after_verified_files() {
    let mut files = fixture_files(false);
    let body = b"fabric loader";
    let profile = json!({"id":"fabric-loader-0.16-1.test", "inheritsFrom":"1.test",
        "mainClass":"net.fabricmc.loader.impl.launch.knot.KnotClient",
        "arguments":{"jvm":["-Dfabric=true"],"game":["--fabric"]},
        "libraries":[{"name":"net.fabricmc:fabric-loader:0.16", "url":"https://libraries.minecraft.net/",
            "sha1":crate::io::files::sha1_hex(sha1::Sha1::digest(body)), "size":body.len()}]});
    files.insert(
        "/fabric/1.test/0.16/profile/json".into(),
        serde_json::to_vec(&profile).unwrap(),
    );
    files.insert(
        "/maven/net/fabricmc/fabric-loader/0.16/fabric-loader-0.16.jar".into(),
        body.to_vec(),
    );
    let mirror = MockMirror::new(files.clone(), false);
    let root = std::env::temp_dir().join(format!("pcl-fabric-install-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    install_version(
        &mirror.client(),
        &root,
        "1.test",
        "Fabric Custom",
        Some(&LoaderInstall {
            family: LoaderFamily::Fabric,
            version: "0.16".into(),
            api: None,
            install_java: false,
        }),
        1,
        &Channel::new(|_| Ok(())),
    )
    .await
    .unwrap();
    let repo = crate::minecraft::instance::folder::McFolder::new("test", root.clone());
    let instance = repo.game_instances().pop().unwrap();
    let metadata = instance.resolved_metadata().unwrap();
    assert_eq!(metadata["mainClass"], profile["mainClass"]);
    assert_eq!(
        instance.plugins_version.get("fabric").map(String::as_str),
        Some("0.16")
    );
    assert_eq!(
        metadata["arguments"]["game"],
        json!(["--fabric", "--fixture", "preserved"])
    );
    assert_eq!(metadata["arguments"]["jvm"], json!(["-Dfabric=true"]));
    assert_eq!(
        metadata["libraries"][0]["name"],
        "net.fabricmc:fabric-loader:0.16"
    );
    assert_eq!(
        fs::read(root.join("libraries/net/fabricmc/fabric-loader/0.16/fabric-loader-0.16.jar"))
            .unwrap(),
        body
    );
    assert_eq!(instance.version, "1.test");
    assert!(
        instance
            .jar_path
            .ends_with("Fabric Custom/Fabric Custom.jar")
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let executable = root.join("fake-java");
        fs::write(
            &executable,
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > captured-args\n",
        )
        .unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
        let runtime = serde_json::from_value(json!({
            "java_folder":root, "version":"21", "slug_version":21,
            "is_64_bit":true, "architecture":"Arm64", "compatibility":"Perfect",
            "java_exe":executable, "vendor":"OpenJDK"
        }))
        .unwrap();
        let state = crate::app::config::AppState {
            profile: crate::app::states::ProfileConfig {
                last_used: 0,
                profiles: vec![crate::minecraft::profile::McProfile::offline(
                    "test", "uuid",
                )],
            },
            active_game_instance: Some(Arc::new(instance)),
            ..Default::default()
        };
        let option =
            crate::minecraft::launch::launcher::LaunchOption::from_state(&state, Arc::new(runtime))
                .unwrap();
        assert!(option.launch_checked().unwrap().wait().unwrap().success());
        let args = fs::read_to_string(root.join("versions/Fabric Custom/captured-args")).unwrap();
        let lines: Vec<_> = args.lines().collect();
        assert!(lines.contains(&"net.fabricmc.loader.impl.launch.knot.KnotClient"));
        assert!(lines.contains(&"-Dfabric=true"));
        let classpath = lines.windows(2).find(|pair| pair[0] == "-cp").unwrap()[1];
        assert!(
            std::env::split_paths(classpath).any(
                |path| path.ends_with("net/fabricmc/fabric-loader/0.16/fabric-loader-0.16.jar")
            )
        );
    }
    drop(mirror);
    files.insert(
        "/maven/net/fabricmc/fabric-loader/0.16/fabric-loader-0.16.jar".into(),
        b"corrupt".to_vec(),
    );
    let bad = MockMirror::new(files, false);
    fs::remove_file(root.join("libraries/net/fabricmc/fabric-loader/0.16/fabric-loader-0.16.jar"))
        .unwrap();
    assert!(
        install_version(
            &bad.client(),
            &root,
            "1.test",
            "Broken Fabric",
            Some(&LoaderInstall {
                family: LoaderFamily::Fabric,
                version: "0.16".into(),
                api: None,
                install_java: false,
            }),
            2,
            &Channel::new(|_| Ok(()))
        )
        .await
        .is_err()
    );
    assert!(!root.join("versions/Broken Fabric").exists());
    assert!(
        root.join("versions/Fabric Custom/Fabric Custom.json")
            .is_file()
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn fabric_merge_preserves_legacy_arguments() {
    let mut metadata = json!({"libraries":[], "minecraftArguments":"--title 'Legacy Title'"});
    let profile = json!({"mainClass":"FabricMain", "libraries":[], "arguments":{"game":["--fabric"], "jvm":[]}});
    merge_loader_profile(&mut metadata, &profile).unwrap();
    assert_eq!(
        metadata["arguments"]["game"],
        json!(["--fabric", "--title", "Legacy Title"])
    );
}

#[tokio::test]
async fn fabric_profile_rejects_wrong_game_or_loader() {
    let profile = json!({"inheritsFrom":"other", "mainClass":"FabricMain",
        "libraries":[{"name":"net.fabricmc:fabric-loader:0.16"}]});
    let mirror = MockMirror::new(
        HashMap::from([(
            "/fabric/1.test/0.16/profile/json".into(),
            serde_json::to_vec(&profile).unwrap(),
        )]),
        false,
    );
    assert!(
        mirror
            .client()
            .get_fabric_profile(
                "1.test",
                "0.16",
                crate::io::net::http::api_client::FabricFamily::Fabric
            )
            .await
            .is_err()
    );
    assert!(
        mirror
            .client()
            .get_fabric_profile(
                "../bad",
                "0.16",
                crate::io::net::http::api_client::FabricFamily::Fabric
            )
            .await
            .is_err()
    );
}

#[tokio::test]
async fn legacy_fabric_list_and_install_use_legacy_intermediary() {
    use crate::io::net::http::api_client::FabricFamily;
    let mut files = fixture_files(false);
    let hash = crate::io::files::sha1_hex(sha1::Sha1::digest(b"library"));
    let library = |name: &str| json!({"name":name,"url":"https://libraries.minecraft.net/","sha1":hash,"size":7});
    let native = fixture_archive(&[("native.bin", b"native")]);
    let native_path = "/maven/example/platform/1/platform-1-natives-test.jar";
    files.insert(native_path.into(), native.clone());
    files.insert(
        format!("{native_path}.sha1"),
        crate::io::files::sha1_hex(sha1::Sha1::digest(&native)).into_bytes(),
    );
    let profile = json!({"inheritsFrom":"1.test", "mainClass":"LegacyMain", "libraries":[
        library("net.legacyfabric:intermediary:1.test"), library("net.fabricmc:fabric-loader:0.16"),
        {"name":"example:platform:1", "url":"https://libraries.minecraft.net/", "natives":{(McRuleContext::current().os):"natives-test"}}
    ]});
    files.insert(
        "/legacy-fabric/1.test".into(),
        serde_json::to_vec(&json!([{"loader":{"version":"0.16","stable":true}}])).unwrap(),
    );
    files.insert(
        "/legacy-fabric/1.test/0.16/profile/json".into(),
        serde_json::to_vec(&profile).unwrap(),
    );
    for path in [
        "/maven/net/legacyfabric/intermediary/1.test/intermediary-1.test.jar",
        "/maven/net/fabricmc/fabric-loader/0.16/fabric-loader-0.16.jar",
    ] {
        files.insert(path.into(), b"library".to_vec());
    }
    let mirror = MockMirror::new(files, false);
    let client = mirror.client();
    let list = client
        .get_fabric_versions("1.test", FabricFamily::Legacyfabric)
        .await
        .unwrap();
    assert_eq!(list[0].version, "0.16");
    let root = std::env::temp_dir().join(format!("pcl-legacy-fabric-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    let choice = LoaderInstall {
        family: LoaderFamily::Legacyfabric,
        version: "0.16".into(),
        api: None,
        install_java: false,
    };
    install_version(
        &client,
        &root,
        "1.test",
        "Legacy",
        Some(&choice),
        1,
        &Channel::new(|_| Ok(())),
    )
    .await
    .unwrap();
    let repo = crate::minecraft::instance::folder::McFolder::new("test", root.clone());
    let instance = repo.game_instances().pop().unwrap();
    let metadata = instance.resolved_metadata().unwrap();
    assert_eq!(metadata["mainClass"], "LegacyMain");
    assert_eq!(
        instance
            .plugins_version
            .get("legacyfabric")
            .map(String::as_str),
        Some("0.16")
    );
    assert!(!instance.plugins_version.contains_key("fabric"));
    assert_eq!(
        metadata["arguments"]["game"],
        json!(["--fixture", "preserved"])
    );
    assert_eq!(
        metadata["libraries"][0]["name"],
        "net.legacyfabric:intermediary:1.test"
    );
    assert!(
        root.join("libraries/net/legacyfabric/intermediary/1.test/intermediary-1.test.jar")
            .is_file()
    );
    assert_eq!(
        fs::read(root.join("versions/Legacy/natives/native.bin")).unwrap(),
        b"native"
    );
    assert_eq!(mirror.count("/fabric/1.test/0.16/profile/json"), 0);
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn full_install_retries_publishes_custom_name_and_reuses_verified_cache() {
    let mirror = MockMirror::new(fixture_files(false), true);
    let client = mirror.client();
    let root = std::env::temp_dir().join(format!("pcl-full-install-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    let repo = crate::minecraft::instance::folder::McFolder::new("test", root.clone());
    assert!(repo.game_instances().is_empty());
    let reports = Arc::new(std::sync::Mutex::new(Vec::new()));
    let events = reports.clone();
    let channel = Channel::new(move |event| {
        events.lock().unwrap().push(format!("{event:?}"));
        Ok(())
    });
    install_version(
        &client,
        &root,
        "1.test",
        "Custom Vanilla",
        None,
        7,
        &channel,
    )
    .await
    .unwrap();
    let path = root.join("versions/Custom Vanilla");
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(path.join("Custom Vanilla.json")).unwrap()).unwrap();
    assert_eq!(value["id"], "Custom Vanilla");
    assert_eq!(value["clientVersion"], "1.test");
    assert_eq!(value["javaVersion"]["majorVersion"], 21);
    assert_eq!(value["arguments"]["game"][0], "--fixture");
    assert_eq!(
        fs::read(path.join("natives/libtest.dylib")).unwrap(),
        b"native"
    );
    assert!(!path.join("natives/META-INF").exists());
    assert!(!path.join("natives/excluded").exists());
    assert_eq!(
        fs::read(path.join("resources/sounds/a.ogg")).unwrap(),
        b"sound"
    );
    assert_eq!(
        fs::read(root.join("assets/virtual/test-index/sounds/duplicate.ogg")).unwrap(),
        b"sound"
    );
    let instances = repo.game_instances();
    assert_eq!(instances.len(), 1, "the same repository sees new installs");
    assert_eq!(instances[0].version, "1.test");
    let details = instances[0].read_version_json().unwrap();
    let downloader = Downloader::with_bases(client.api_bases_async().await);
    prepare_legacy_assets(&details, &root, &root, &downloader, true)
        .await
        .unwrap();
    assert_eq!(
        fs::read(root.join("resources/sounds/a.ogg")).unwrap(),
        b"sound"
    );
    let mut external_details = details.clone();
    external_details.asset_index.download.sha1.clear();
    external_details.asset_index.download.url.clear();
    prepare_legacy_assets(&external_details, &root, &root, &downloader, true)
        .await
        .unwrap();
    fs::write(root.join("resources/sounds/a.ogg"), b"user data").unwrap();
    assert!(
        prepare_legacy_assets(&details, &root, &root, &downloader, true)
            .await
            .is_err()
    );
    assert_eq!(
        fs::read(root.join("resources/sounds/a.ogg")).unwrap(),
        b"user data"
    );
    assert_eq!(mirror.count("/client.jar"), 2, "503 is retried");
    assert_eq!(mirror.count("/forbidden.jar"), 0);
    assert!(
        install_version(
            &client,
            &root,
            "1.test",
            "Custom Vanilla",
            None,
            8,
            &channel
        )
        .await
        .is_err()
    );
    assert!(
        path.join("Custom Vanilla.json").is_file(),
        "existing installations survive conflicts"
    );
    install_version(
        &client,
        &root,
        "1.test",
        "Second",
        None,
        9,
        &Channel::new(|_| Err(tauri::Error::Io(std::io::Error::other("UI closed")))),
    )
    .await
    .unwrap();
    assert_eq!(repo.game_instances().len(), 2);
    assert_eq!(mirror.count("/maven/test/library.jar"), 1);
    assert_eq!(mirror.count("/maven/test/native.jar"), 1);
    assert_eq!(mirror.count("/index.json"), 1);
    assert_eq!(mirror.count("/log.xml"), 1);
    assert!(!reports.lock().unwrap().is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn failed_install_never_publishes_an_instance_and_preserves_existing_data() {
    let mirror = MockMirror::new(fixture_files(true), false);
    let root = std::env::temp_dir().join(format!("pcl-failed-install-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("keep.txt"), b"keep").unwrap();
    let result = install_version(
        &mirror.client(),
        &root,
        "1.test",
        "Broken",
        None,
        0,
        &Channel::new(|_| Ok(())),
    )
    .await;
    assert!(result.is_err());
    assert!(!root.join("versions/Broken").exists());
    assert_eq!(fs::read(root.join("keep.txt")).unwrap(), b"keep");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_extraction_refuses_zip_slip() {
    let root = std::env::temp_dir().join(format!("pcl-native-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    let path = root.join("bad.jar");
    fs::write(&path, fixture_archive(&[("../outside", b"bad")])).unwrap();
    assert!(
        extract_natives(
            &[NativeArchive {
                path,
                exclude: vec![],
                lwjgl_core: false,
            }],
            &root.join("natives")
        )
        .is_err()
    );
    assert!(!root.join("outside").exists());
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn failed_transfers_preserve_existing_files_and_remove_partial_files() {
    let mirror = MockMirror::new(fixture_files(false), false);
    let root = std::env::temp_dir().join(format!("pcl-transfer-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    let path = root.join("existing.jar");
    fs::write(&path, b"keep existing data").unwrap();
    let downloader = Downloader::default();
    let mut info = DownloadFile {
        sha1: "0".repeat(40),
        size: b"verified client".len() as u64,
        url: format!("{}/client.jar", mirror.url),
        path: None,
    };
    assert!(downloader.download(&info, &path).await.is_err());
    assert_eq!(fs::read(&path).unwrap(), b"keep existing data");
    assert_eq!(
        fs::read_dir(&root).unwrap().count(),
        1,
        "failed .part files are removed"
    );
    assert_eq!(mirror.count("/client.jar"), 3);
    info.size = 1;
    assert!(downloader.download(&info, &path).await.is_err());
    assert_eq!(fs::read(&path).unwrap(), b"keep existing data");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    info.sha1 = crate::io::files::sha1_hex(sha1::Sha1::digest(b"keep existing data"));
    info.size = b"keep existing data".len() as u64;
    downloader.download(&info, &path).await.unwrap();
    assert_eq!(
        mirror.count("/client.jar"),
        6,
        "verified cache does not access the network"
    );
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn tampered_version_json_fails_before_downloading_files() {
    let mut files = fixture_files(false);
    files.get_mut("/version.json").unwrap().push(b' ');
    let mirror = MockMirror::new(files, false);
    let root = std::env::temp_dir().join(format!("pcl-metadata-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    assert!(
        install_version(
            &mirror.client(),
            &root,
            "1.test",
            "Broken",
            None,
            0,
            &Channel::new(|_| Ok(()))
        )
        .await
        .is_err()
    );
    assert!(!root.join("versions/Broken").exists());
    assert_eq!(mirror.count("/index.json"), 0);
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
#[ignore = "Fetches official version JSON and asset indexes; run explicitly"]
async fn official_metadata_across_generations() {
    let client = MinecraftApiClient::new(
        reqwest::Client::new(),
        &crate::io::net::http::api_client::ApiProvider::Official,
    );
    let root = std::env::temp_dir().join(format!("pcl-metadata-smoke-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    let manifest: crate::minecraft::version::McVersionManifest = client
        .get_with_endpoint(crate::minecraft::version::VERSION_MANIFEST_ENDPOINT, true)
        .await
        .unwrap();
    let downloader = Downloader::default();
    for id in [
        "rd-132211",
        "b1.7.3",
        "1.2.5",
        "1.16.5",
        "1.21.1",
        &manifest.latest.release,
    ] {
        let value = client.get_version_metadata(id).await.unwrap();
        let details: McVersion = serde_json::from_value(value).unwrap();
        let index_path = root.join(format!("{}.json", details.asset_index.id));
        downloader
            .download(&details.asset_index.download, &index_path)
            .await
            .unwrap();
        let index: AssetIndex = serde_json::from_slice(&fs::read(index_path).unwrap()).unwrap();
        let mut plan = InstallPlan::new();
        plan.add(1, &details.downloads.client, root.join("client.jar"))
            .unwrap();
        plan.libraries(&details, &root, &McRuleContext::current(), 2)
            .unwrap();
        plan.assets(&index, &root, "https://resources.download.minecraft.net")
            .unwrap();
        println!(
            "{id}: {} downloads, {} legacy native archives, {} logical assets",
            plan.files.len(),
            plan.natives.len(),
            index.objects.len()
        );
    }
    fs::remove_dir_all(root).unwrap();
}
