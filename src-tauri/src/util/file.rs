use std::path::PathBuf;

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
