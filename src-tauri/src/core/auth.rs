#[derive(Debug, Clone)]
pub struct Account {
    pub username: String,
    pub uuid: String,
}

impl Account {
    pub fn new(username: String, uuid: String) -> Self {
        Self { username, uuid }
    }
}
