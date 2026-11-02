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
    tauri_build::build()
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
