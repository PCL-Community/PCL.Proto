fn main() {
    // 对照 CE 的 Source Generator：密钥在编译期读取，改动后需要重新构建本 crate。
    println!("cargo:rerun-if-env-changed=CURSEFORGE_API_KEY");
    println!("cargo:rerun-if-changed=../.env");
    if std::env::var_os("CURSEFORGE_API_KEY").is_none() {
        if let Ok(contents) = std::fs::read_to_string("../.env") {
            for line in contents.lines() {
                if let Some((name, value)) = line.split_once('=') {
                    if name.trim() == "CURSEFORGE_API_KEY" {
                        let value = value.trim();
                        let value = value
                            .strip_prefix('"')
                            .and_then(|s| s.strip_suffix('"'))
                            .or_else(|| value.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
                            .unwrap_or(value);
                        println!("cargo:rustc-env=CURSEFORGE_API_KEY={value}");
                        break;
                    }
                }
            }
        }
    }
    tauri_build::build()
}
