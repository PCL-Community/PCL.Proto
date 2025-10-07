pub mod crypto;

pub mod file;
pub mod toys;

pub fn get_pcl_hash(str: &str) -> u64 {
    let mut result = 5381 as u64;
    for c in str.chars() {
        result = (result << 5) ^ result ^ c as u64;
    }
    result ^ 0xA98F501BC684032F as u64
}
