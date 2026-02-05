use crate::{scaffolding::{
    self, room::member::Member, server::ScaffoldingServer,
}, util::byte_buffer::ByteBuffer};

type Sender = Option<Member>;

/// 已注册的协议列表
pub enum RequestType {
    Ping,               // c:ping
    Protocols,          // c:protocols
    ServerPort,         // c:server_port
    PlayerPing,         // c:player_ping
    PlayerProfilesList, // c:player_profiles_list
}

impl RequestType {
    fn to_string(&self) -> &'static str {
        match self {
            RequestType::Ping => "c:ping",
            RequestType::Protocols => "c:protocols",
            RequestType::ServerPort => "c:server_port",
            RequestType::PlayerPing => "c:player_ping",
            RequestType::PlayerProfilesList => "c:player_profiles_list",
        }
    }

    pub fn protocols() -> Vec<&'static str> {
        vec![
            "c:ping",
            "c:protocols",
            "c:server_port",
            "c:player_ping",
            "c:player_profiles_list",
        ]
    }
}

/// 处理请求
pub fn handle_request(
    server: &mut ScaffoldingServer,
    connection_id: usize,
    request_type: &RequestType,
    request_body: &mut ByteBuffer,
    response_buffer: &mut ByteBuffer,
) -> Result<(), scaffolding::ScaffoldingError> {
    // 构建Sender
    let sender = build_sender(server, connection_id);

    // 处理请求
    let response = match request_type {
        RequestType::Ping => handle_ping(&sender, request_body),
        RequestType::Protocols => handle_protocols(&sender, request_body),
        RequestType::ServerPort => handle_server_port(server, &sender, request_body),
        RequestType::PlayerPing => handle_player_ping(server, connection_id, &sender, request_body),
        RequestType::PlayerProfilesList => {
            handle_player_profiles_list(server, &sender, request_body)
        }
    }?;

    // 写入响应
    response_buffer.write_u8(response.status);
    response_buffer.write_u32(response.data.len() as u32);
    response_buffer.write_data(&response.data);

    Ok(())
}

/// 构建Sender
fn build_sender(server: &ScaffoldingServer, connection_id: usize) -> Sender {
    let member = server
        .machine_id_map
        .get(&connection_id)
        .and_then(|machine_id| {
            server
                .room
                .members
                .iter()
                .find(|m| m.machine_id == *machine_id)
                .cloned()
        });
    member
}

/// 处理ping请求
fn handle_ping(
    _sender: &Sender,
    request_body: &ByteBuffer,
) -> Result<scaffolding::Response, scaffolding::ScaffoldingError> {
    Ok(scaffolding::Response {
        status: 0,
        data: request_body.data.clone(),
    })
}

/// 处理protocols请求
fn handle_protocols(
    _sender: &Sender,
    _request_body: &ByteBuffer,
) -> Result<scaffolding::Response, scaffolding::ScaffoldingError> {
    let protocols = RequestType::protocols();
    let mut data = Vec::new();
    let mut buffer = ByteBuffer::new_empty();
    buffer.write_string(&protocols.join("\0"));
    data.extend_from_slice(&buffer.data);

    Ok(scaffolding::Response { status: 0, data })
}

/// 处理server_port请求
fn handle_server_port(
    server: &ScaffoldingServer,
    _sender: &Sender,
    _request_body: &ByteBuffer,
) -> Result<scaffolding::Response, scaffolding::ScaffoldingError> {
    let server_port = server.room.server_port;

    let mut data = Vec::new();
    let mut buffer = ByteBuffer::new_empty();
    buffer.write_u16(server_port);
    data.extend_from_slice(&buffer.data);

    Ok(scaffolding::Response { status: 0, data })
}

/// 处理player_ping请求
fn handle_player_ping(
    server: &mut ScaffoldingServer,
    connection_id: usize,
    _sender: &Sender,
    request_body: &ByteBuffer,
) -> Result<scaffolding::Response, scaffolding::ScaffoldingError> {
    // 解码玩家信息
    let member: Member = serde_json::from_slice(&request_body.data)
        .map_err(|e| scaffolding::ScaffoldingError::Other(e.to_string()))?;

    // 检查机器ID冲突
    if server.machine_id_map.get(&connection_id).is_none()
        && server
            .machine_id_map
            .values()
            .any(|id| *id == member.machine_id)
    {
        println!("Detected a machine_id collision");
        return Err(scaffolding::ScaffoldingError::ProtocolError(
            "machine_id collision".to_string(),
        ));
    }

    if let Some(machine_id) = server.machine_id_map.get(&connection_id) {
        if machine_id != &member.machine_id {
            println!("machine_id mismatch detected");
            return Err(scaffolding::ScaffoldingError::ProtocolError(
                "player info mismatch".to_string(),
            ));
        }
    }

    // 更新机器ID映射
    server
        .machine_id_map
        .insert(connection_id, member.machine_id.clone());

    // 检查玩家是否已存在
    if let Some(stored_member) = server
        .room
        .members
        .iter()
        .find(|m| m.machine_id == member.machine_id)
    {
        if stored_member != &member {
            println!("Member info mismatch for {}", stored_member.name);
            return Err(scaffolding::ScaffoldingError::ProtocolError(
                "player info mismatch".to_string(),
            ));
        }
    } else {
        // 添加新玩家
        println!(
            "Received player info: {{ \"name\": \"{}\", \"vendor\": \"{}\", \"machine_id\": \"{}\"}}",
            member.name, member.vendor, member.machine_id
        );
        server.room.members.push(member);
    }

    Ok(scaffolding::Response {
        status: 0,
        data: Vec::new(),
    })
}

/// 处理player_profiles_list请求
fn handle_player_profiles_list(
    server: &ScaffoldingServer,
    _sender: &Sender,
    _request_body: &ByteBuffer,
) -> Result<scaffolding::Response, scaffolding::ScaffoldingError> {
    let data = serde_json::to_vec(&server.room.members)
        .map_err(|e| scaffolding::ScaffoldingError::ProtocolError(e.to_string()))?;

    Ok(scaffolding::Response { status: 0, data })
}
