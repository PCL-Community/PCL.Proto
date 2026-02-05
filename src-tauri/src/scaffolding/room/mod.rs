pub mod member;

pub struct Room {
    pub members: Vec<member::Member>,
    pub server_port: u16,
}

impl Room {
    /// 创建一个新的房间
    pub fn new(server_port: u16) -> Self {
        Self {
            members: Vec::new(),
            server_port,
        }
    }
}
