//! PCL 的哈希函数；离线 UUID 与启动器侧的标识都基于它。

pub fn get_pcl_hash(str: &str) -> u64 {
    let mut result = 5381_u64;
    for c in str.chars() {
        result = (result << 5) ^ result ^ c as u64;
    }
    result ^ 0xA98F501BC684032F_u64
}
