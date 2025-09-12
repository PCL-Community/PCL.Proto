#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum Account {
    Offline {
        username: String,
        uuid: String,
    },
    Yggdrasil {
        username: String,
        uuid: String,
        access_token: String,
    },
    Microsoft {
        username: String,
        uuid: String,
        access_token: String,
        refresh_token: String,
        xuid: String,
    },
    /// 外置登录（Authlib-Injector）
    AuthlibInjector {
        /// 第三方登录 API 提供方的 API 链接，登录的请求将通过这个 API 发送
        api_location: String,
        /// 第三方登录 API 提供方的服务器名称，用于 GUI 显示
        server_name: String,
        /// 第三方登录 API 提供方的网页主页，用于 GUI 显示跳转
        server_homepage: String,
        /// 第三方登录 API 提供方的元数据，需要在启动时携带这个作为参数
        server_meta: String,
        /// 第三方登录令牌，将会作为启动参数的一部分传入游戏实例
        access_token: String,
        /// 第三方正版玩家的统一标识
        uuid: String,
        /// 第三方正版玩家的名称
        username: String,
    },
}

impl Account {
    pub fn username(&self) -> &str {
        match self {
            Account::Offline { username, .. } => username,
            Account::Yggdrasil { username, .. } => username,
            Account::Microsoft { username, .. } => username,
            Account::AuthlibInjector { username, .. } => username,
        }
    }

    pub fn uuid(&self) -> &str {
        match self {
            Account::Offline { uuid, .. } => uuid,
            Account::Yggdrasil { uuid, .. } => uuid,
            Account::Microsoft { uuid, .. } => uuid,
            Account::AuthlibInjector { uuid, .. } => uuid,
        }
    }

    // 对于只有部分变体有的字段，返回 Option
    pub fn access_token(&self) -> Option<&str> {
        match self {
            Account::Offline { .. } => None,
            Account::Yggdrasil { access_token, .. } => Some(access_token),
            Account::Microsoft { access_token, .. } => Some(access_token),
            Account::AuthlibInjector { access_token, .. } => Some(access_token),
        }
    }
}
