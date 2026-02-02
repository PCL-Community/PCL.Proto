pub mod scanning;

// 辅助函数：检查 Minecraft 服务器连接
pub fn check_mc_connection(port: u16) -> bool {
    // 实现 Minecraft 服务器连接检查逻辑
    // 例如：尝试与本地 Minecraft 服务器建立连接
    use std::net::TcpStream;
    TcpStream::connect(("127.0.0.1", port)).is_ok()
}