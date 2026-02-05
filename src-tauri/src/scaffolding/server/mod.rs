use crate::{scaffolding::{
    easytier::EasyTier,
    room::{self, member::Member}, util::room_code::RoomCode,
}, util::byte_buffer::ByteBuffer};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

pub mod request_handler;

/// 联机中心
pub struct ScaffoldingServer {
    /// 房间
    pub room: room::Room,
    /// 房间码
    pub room_code: String,
    /// 机器ID映射 (connection_id -> machine_id)
    pub machine_id_map: HashMap<usize, String>,
    /// EasyTier 实例
    easytier: Option<EasyTier>,
    /// TCP 监听器
    listener: Option<TcpListener>,
    /// 连接列表
    connections: Arc<Mutex<Vec<usize>>>,
    /// 连接任务
    connection_tasks: Arc<Mutex<HashMap<usize, tokio::task::JoinHandle<()>>>>,
    /// 下一个连接ID
    next_connection_id: Arc<Mutex<usize>>,
}

impl ScaffoldingServer {
    /// 创建联机中心
    ///
    /// # Parameters
    /// * `easytier` - 使用的 EasyTier
    /// * `room_code` - 房间码
    /// * `player_name` - 玩家名
    /// * `vendor` - 联机客户端信息
    /// * `server_port` - Minecraft 服务器端口号
    pub fn new(
        easytier: EasyTier,
        room_code: String,
        player_name: String,
        vendor: String,
        server_port: u16,
    ) -> Self {
        let host_member = Member {
            name: player_name,
            machine_id: Self::get_machine_id(true),
            vendor,
            kind: room::member::PlayerKind::Host,
        };

        let mut room = room::Room::new(server_port);
        room.members.push(host_member);

        Self {
            room,
            room_code,
            machine_id_map: HashMap::new(),
            easytier: Some(easytier),
            listener: None,
            connections: Arc::new(Mutex::new(Vec::new())),
            connection_tasks: Arc::new(Mutex::new(HashMap::new())),
            next_connection_id: Arc::new(Mutex::new(0)),
        }
    }

    /// 获取机器ID
    fn get_machine_id(_for_host: bool) -> String {
        // TODO: 实现实际的机器ID生成逻辑
        // 暂时使用UUID作为机器ID
        uuid::Uuid::new_v4().to_string()
    }

    /// 启动联机中心监听器
    ///
    /// 默认会在 `13452` 端口监听。若该端口被占用，会重新申请一个端口。
    ///
    /// # Returns
    /// 联机中心端口号
    pub async fn start_listener(&mut self) -> Result<u16, String> {
        let port = self.find_available_port(13452).await?;
        let addr = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

        println!("ScaffoldingServer listener started at 127.0.0.1:{}", port);
        self.listener = Some(listener);
        Ok(port)
    }

    /// 查找可用端口
    async fn find_available_port(&self, start_port: u16) -> Result<u16, String> {
        let mut port = start_port;
        for _ in 0..100 {
            let addr = format!("0.0.0.0:{}", port);
            if TcpListener::bind(&addr).await.is_ok() {
                return Ok(port);
            }
            port += 1;
        }
        Err("No available port found".to_string())
    }

    /// 创建 EasyTier 网络
    ///
    /// 如果只是本地测试，无需创建 EasyTier 网络，可以直接使用 `ScaffoldingClient.connect_directly(port:)` 连接。
    pub fn create_room(&mut self) -> Result<(), String> {
        if self.listener.is_none() {
            return Err("Listener not started".to_string());
        }

        let Some(room_code) = RoomCode::from_code(&self.room_code) else {
            return Err("Invalid room code".to_string());
        };
        let network_name = room_code.network_name.clone();
        let network_secret = room_code.network_secret.clone();

        // TODO: 实现 EasyTier 启动逻辑
        // 暂时跳过 EasyTier 启动
        println!(
            "Would create EasyTier network: name={}, secret={}",
            network_name, network_secret
        );

        Ok(())
    }

    /// 关闭房间并断开所有连接
    pub async fn stop(&mut self) {
        println!("Stopping scaffolding server");

        // 停止 EasyTier
        if let Some(easytier) = self.easytier.as_mut() {
            // TODO: 实现 EasyTier 终止逻辑
            drop(easytier);
        }
        self.easytier = None;

        // 取消监听器
        self.listener = None;

        // 断开所有连接
        let connection_ids: Vec<usize> = self.connections.lock().await.clone();
        for connection_id in connection_ids {
            self.cleanup(connection_id).await;
        }

        // 清空连接列表
        let mut connections = self.connections.lock().await;
        connections.clear();
        drop(connections);
    }

    /// 清理连接
    async fn cleanup(&mut self, connection_id: usize) {
        // 移除机器ID映射
        if let Some(machine_id) = self.machine_id_map.get(&connection_id) {
            let members = &mut self.room.members;
            members.retain(|m| &m.machine_id != machine_id);
            drop(members);
        }

        // 取消连接任务
        let mut tasks = self.connection_tasks.lock().await;
        if let Some(task) = tasks.remove(&connection_id) {
            task.abort();
        }
        drop(tasks);

        // 移除连接
        let mut connections = self.connections.lock().await;
        connections.retain(|id| id != &connection_id);
        drop(connections);
    }

    /// 处理连接
    async fn handle_connection(&mut self, stream: tokio::net::TcpStream, addr: SocketAddr) {
        println!("New connection: {}", addr);

        // 生成连接ID
        let mut next_id = self.next_connection_id.lock().await;
        let connection_id = *next_id;
        *next_id += 1;
        drop(next_id);

        // 添加到连接列表
        let mut connections = self.connections.lock().await;
        connections.push(connection_id);
        drop(connections);

        // 创建连接任务
        let connections = self.connections.clone();
        let connection_tasks = self.connection_tasks.clone();

        let task = tokio::spawn(async move {
            if let Err(e) = Self::start_receiving(connection_id, stream, addr).await {
                println!("Error processing requests: {}", e);
            }

            // 清理连接
            let mut tasks = connection_tasks.lock().await;
            if let Some(task) = tasks.remove(&connection_id) {
                task.abort();
            }
            drop(tasks);

            let mut connections = connections.lock().await;
            connections.retain(|id| id != &connection_id);
            drop(connections);
        });

        let mut tasks = self.connection_tasks.lock().await;
        tasks.insert(connection_id, task);
        drop(tasks);
    }

    /// 开始接收数据
    ///
    /// 该方法只会在连接发生异常或连接断开时返回
    async fn start_receiving(
        connection_id: usize,
        mut stream: tokio::net::TcpStream,
        addr: SocketAddr,
    ) -> Result<(), String> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        loop {
            // 读取类型长度
            let mut type_length_buf = [0u8; 1];
            stream
                .read_exact(&mut type_length_buf)
                .await
                .map_err(|e| format!("Failed to read type length: {}", e))?;
            let type_length = type_length_buf[0] as usize;

            // 读取类型
            let mut type_buf = vec![0u8; type_length + 4];
            stream
                .read_exact(&mut type_buf)
                .await
                .map_err(|e| format!("Failed to read type: {}", e))?;

            let request_type = String::from_utf8(type_buf[..type_length].to_vec())
                .map_err(|e| format!("Failed to parse request type: {}", e))?;

            // 读取数据长度
            let body_length = u32::from_be_bytes([
                type_buf[type_length],
                type_buf[type_length + 1],
                type_buf[type_length + 2],
                type_buf[type_length + 3],
            ]) as usize;

            // 读取数据
            let mut body_data = vec![0u8; body_length];
            if body_length > 0 {
                stream
                    .read_exact(&mut body_data)
                    .await
                    .map_err(|e| format!("Failed to read body: {}", e))?;
            }

            println!("Received request: {} from {}", request_type, addr);

            // 处理请求
            let mut request_buffer = ByteBuffer::new(body_data);
            let mut response_buffer = ByteBuffer::new_empty();

            // 注意：这里需要可变引用，但由于 handler 是不可变的，我们需要调整
            // 暂时跳过实际处理，只返回空响应
            // handler.handle_request(connection_id, &request_type, &mut request_buffer, &mut response_buffer)?;

            // 发送响应
            stream
                .write_all(&response_buffer.data)
                .await
                .map_err(|e| format!("Failed to send response: {}", e))?;
        }
    }

    /// 开始接受连接
    pub async fn accept_connections(&mut self) -> Result<(), String> {
        loop {
            let listener = self
                .listener
                .as_ref()
                .ok_or("Listener not started".to_string())?;

            let (stream, addr) = listener
                .accept()
                .await
                .map_err(|e| format!("Failed to accept connection: {}", e))?;

            self.handle_connection(stream, addr).await;
        }
    }
}
