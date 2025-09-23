use std::{
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use sha1::Digest;

/// find file of name
pub fn find_file_of_name(
    files_itr: &mut impl Iterator<Item = std::fs::DirEntry>,
    name: &str,
) -> Option<PathBuf> {
    while let Some(json_file) = files_itr.next() {
        let current_json_file_name = json_file.file_name();
        let current_json_file_name = current_json_file_name.to_string_lossy();
        if current_json_file_name.starts_with(name) {
            return Some(json_file.path());
        }
    }
    None
}

pub fn check_sha1(file: &Path, given: &str) -> Result<bool, std::io::Error> {
    let file = std::fs::File::open(file)?;
    let mut reader = BufReader::new(file);
    let mut hasher = sha1::Sha1::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    let result = hasher.finalize();
    let computed = format!("{:x}", result);
    Ok(computed == given)
}
