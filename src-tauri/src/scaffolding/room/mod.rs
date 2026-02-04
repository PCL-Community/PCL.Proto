pub mod member;

pub struct Room {
    members: Vec<member::Member>,
    server_port: u16,
}
