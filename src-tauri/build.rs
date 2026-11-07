/// 对照 CE 的 Source Generator：密钥在编译期读取，改动后需要重新构建本 crate。
const SECRET_KEYS: [&str; 2] = ["CURSEFORGE_API_KEY", "MS_CLIENT_ID"];

fn main() {
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
