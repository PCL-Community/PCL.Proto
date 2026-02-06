use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // 如果是Windows系统，需要下载npcap SDK
    if cfg!(target_os = "windows") {
        let url = "https://npcap.com/dist/npcap-sdk-1.16.zip";
        let zip_path = "npcap-sdk-1.16.zip";
        let extract_dir = "npcap-sdk-1.16";
        if !Path::new(extract_dir).exists() {
            println!("Downloading npcap SDK...");
            let response = reqwest::blocking::get(url).expect("Failed to download npcap SDK");
            let mut file = fs::File::create(zip_path).expect("Failed to create zip file");
            let content = response.bytes().expect("Failed to get response bytes");
            std::io::copy(&mut content.as_ref(), &mut file).expect("Failed to write to file");
            println!("Extracting npcap SDK...");
            let file = fs::File::open(zip_path).expect("Failed to open zip file");
            let mut archive = zip::ZipArchive::new(file).expect("Failed to create zip archive");
            archive.extract(".").expect("Failed to extract zip file");
            fs::remove_file(zip_path).ok();
        }
        
        let lib_path = Path::new(extract_dir).join("Lib");
        let lib_path_str = lib_path.to_str().expect("Failed to convert path to string");
        let mut lib_env = env::var("LIB").unwrap_or_default();
        if !lib_env.is_empty() {
            lib_env.push(';');
        }
        lib_env.push_str(lib_path_str);
        println!("Set LIB environment variable to: {}", lib_env);
        unsafe {
            env::set_var("LIB", lib_env);
        }
    }
    
    tauri_build::build()
}
