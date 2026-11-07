//! Minecraft server status queries.

pub mod model;

use anyhow::{bail, ensure};
use hickory_resolver::{
    TokioResolver,
    proto::rr::{RData, rdata::SRV},
};
use model::McPingResult;
use std::{
    io::Cursor,
    net::{IpAddr, SocketAddr},
    time::{Duration, Instant},
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};

const DEFAULT_PORT: u16 = 25565;
const TIMEOUT: Duration = Duration::from_secs(10);
const CONNECT_TIMEOUT: Duration = Duration::from_millis(2500);
const MAX_PACKET: usize = 1024 * 1024;

pub(crate) fn parse_address(input: &str) -> anyhow::Result<(String, Option<u16>)> {
    let input = input.trim();
    let input = input
        .split_once("://")
        .map_or(input, |(_, address)| address)
        .trim_end_matches('/');
    if let Ok(addr) = input.parse::<SocketAddr>() {
        ensure!(addr.port() != 0, "Invalid port");
        return Ok((addr.ip().to_string(), Some(addr.port())));
    }
    let input = input
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(input);
    if input.parse::<IpAddr>().is_ok() {
        return Ok((input.to_owned(), None));
    }
    let (host, port) = match input.split_once(':') {
        Some((host, port)) => (host, Some(port.parse::<u16>()?)),
        None => (input, None),
    };
    ensure!(
        !host.is_empty() && host.len() <= 255 && !host.contains(['/', '[', ']']) && port != Some(0),
        "Invalid server address"
    );
    Ok((host.to_owned(), port))
}

fn order_srv(mut records: Vec<SRV>, mut draw: impl FnMut(usize) -> usize) -> Vec<SRV> {
    records.retain(|srv| srv.port != 0 && !srv.target.is_root());
    records.sort_by_key(|srv| srv.priority);
    let mut ordered = Vec::with_capacity(records.len());
    while !records.is_empty() {
        let group_len = records.partition_point(|srv| srv.priority == records[0].priority);
        let total: usize = records[..group_len]
            .iter()
            .map(|srv| usize::from(srv.weight))
            .sum();
        let index = if total == 0 {
            draw(group_len)
        } else {
            let mut ticket = draw(total);
            records[..group_len]
                .iter()
                .position(|srv| {
                    if ticket < usize::from(srv.weight) {
                        true
                    } else {
                        ticket -= usize::from(srv.weight);
                        false
                    }
                })
                .unwrap()
        };
        ordered.push(records.remove(index));
    }
    ordered
}

async fn query_resolved(
    resolver: &TokioResolver,
    host: &str,
    port: Option<u16>,
    deadline: tokio::time::Instant,
) -> anyhow::Result<(McPingResult, u128)> {
    let mut targets = Vec::new();
    if port.is_none() {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if let Ok(Ok(records)) = tokio::time::timeout(
            (remaining / 2).min(CONNECT_TIMEOUT),
            resolver.srv_lookup(format!("_minecraft._tcp.{}", host.trim_end_matches('.'))),
        )
        .await
        {
            let records: Vec<_> = records
                .answers()
                .iter()
                .filter_map(|record| match &record.data {
                    RData::SRV(srv) => Some(srv.clone()),
                    _ => None,
                })
                .collect();
            for srv in order_srv(records, |upper| fastrand::usize(..upper)) {
                targets.push((srv.target.to_string(), srv.port));
            }
        }
    }
    targets.push((host.to_string(), port.unwrap_or(DEFAULT_PORT)));
    let mut last_error = None;
    for (index, (target, port)) in targets.iter().enumerate() {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        let count = u32::try_from(targets.len() - index).unwrap_or(u32::MAX);
        let target_deadline = tokio::time::Instant::now() + remaining / count;
        let dns_budget = (remaining / count / 2).min(CONNECT_TIMEOUT);
        let addresses = match tokio::time::timeout(
            dns_budget,
            resolver.lookup_ip(target.trim_end_matches('.')),
        )
        .await
        {
            Ok(Ok(ips)) => ips
                .iter()
                .map(|ip| SocketAddr::new(ip, *port))
                .collect::<Vec<_>>(),
            Ok(Err(error)) => {
                last_error = Some(anyhow::anyhow!("DNS {target}: {error}"));
                continue;
            }
            Err(error) => {
                last_error = Some(anyhow::anyhow!("DNS {target}: {error}"));
                continue;
            }
        };
        match query_endpoints(host, &addresses, target_deadline).await {
            Ok(result) => return Ok(result),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error
        .unwrap_or_else(|| anyhow::anyhow!("No server endpoints responded before the deadline")))
}

async fn query(host: &str, endpoint: SocketAddr) -> anyhow::Result<(McPingResult, u128)> {
    let mut stream =
        tokio::time::timeout(CONNECT_TIMEOUT, tokio::net::TcpStream::connect(endpoint)).await??;
    let mut handshake = vec![0];
    encode(772, &mut handshake);
    encode(host.len(), &mut handshake);
    handshake.extend_from_slice(host.as_bytes());
    handshake.extend_from_slice(&endpoint.port().to_be_bytes());
    handshake.push(1);
    let mut request = Vec::new();
    encode(handshake.len(), &mut request);
    request.extend_from_slice(&handshake);
    request.extend_from_slice(&[1, 0, 9, 1]);
    request.extend_from_slice(&0i64.to_be_bytes());
    stream.write_all(&request).await?;
    let start = Instant::now();
    let mut status = None;
    let mut latency = None;
    while status.is_none() || latency.is_none() {
        let length = read_varint(&mut stream).await?;
        ensure!((1..=MAX_PACKET).contains(&length), "Invalid packet length");
        let mut packet = vec![0; length];
        stream.read_exact(&mut packet).await?;
        let mut packet = Cursor::new(packet);
        match read_varint(&mut packet).await? {
            0 => {
                let length = read_varint(&mut packet).await?;
                let offset = packet.position() as usize;
                ensure!(
                    length == packet.get_ref().len() - offset,
                    "Invalid status length"
                );
                status = Some(serde_json::from_slice::<McPingResult>(
                    &packet.get_ref()[offset..],
                )?);
            }
            1 => {
                let offset = packet.position() as usize;
                ensure!(
                    packet.get_ref()[offset..] == 0i64.to_be_bytes(),
                    "Invalid pong"
                );
                latency = Some(start.elapsed().as_millis());
            }
            _ => {}
        }
    }
    let latency = latency.unwrap();
    let mut status = status.unwrap();
    status.latency = latency as u64;
    Ok((status, latency))
}

async fn query_endpoints(
    host: &str,
    endpoints: &[SocketAddr],
    deadline: tokio::time::Instant,
) -> anyhow::Result<(McPingResult, u128)> {
    let mut last_error = None;
    for (index, endpoint) in endpoints.iter().enumerate() {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        let count = u32::try_from(endpoints.len() - index).unwrap_or(u32::MAX);
        let budget = if count == 1 {
            remaining
        } else {
            (remaining / count).min(CONNECT_TIMEOUT)
        };
        match tokio::time::timeout(budget, query(host, *endpoint)).await {
            Ok(Ok(result)) => return Ok(result),
            Ok(Err(error)) => last_error = Some(anyhow::anyhow!("Server {endpoint}: {error:#}")),
            Err(error) => last_error = Some(anyhow::anyhow!("Server {endpoint}: {error}")),
        }
    }
    Err(last_error
        .unwrap_or_else(|| anyhow::anyhow!("No server endpoints responded before the deadline")))
}

fn encode(mut value: usize, output: &mut Vec<u8>) {
    while value > 0x7f {
        output.push((value as u8) | 0x80);
        value >>= 7;
    }
    output.push(value as u8);
}

async fn read_varint<R: AsyncRead + Unpin>(stream: &mut R) -> anyhow::Result<usize> {
    let mut result = 0usize;
    for index in 0..5 {
        let byte = stream.read_u8().await?;
        ensure!(index != 4 || byte <= 7, "Invalid VarInt");
        result |= usize::from(byte & 0x7f) << (7 * index);
        if byte & 0x80 == 0 {
            return Ok(result);
        }
    }
    bail!("Invalid VarInt")
}

pub async fn server_query(addr_str: &str) -> Result<(McPingResult, u128), String> {
    let deadline = tokio::time::Instant::now() + TIMEOUT;
    let operation = async {
        let (host, port) = parse_address(addr_str)?;
        if let Ok(ip) = host.parse::<IpAddr>() {
            return query_endpoints(
                &host,
                &[SocketAddr::new(ip, port.unwrap_or(DEFAULT_PORT))],
                deadline,
            )
            .await;
        }
        let resolver = TokioResolver::builder_tokio()?.build()?;
        query_resolved(&resolver, &host, port, deadline).await
    };
    tokio::time::timeout_at(deadline, operation)
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e: anyhow::Error| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hickory_resolver::proto::rr::Name;

    fn srv(priority: u16, weight: u16, port: u16, target: &str) -> SRV {
        SRV::new(priority, weight, port, Name::from_ascii(target).unwrap())
    }

    #[test]
    fn srv_order_respects_priority_weight_and_zero_weight() {
        let records = vec![
            srv(20, 50, 20, "later.example."),
            srv(10, 1, 1, "one.example."),
            srv(10, 3, 2, "three.example."),
            srv(10, 0, 3, "zero.example."),
            srv(0, 1, 4, "."),
            srv(0, 1, 0, "invalid.example."),
        ];
        let ports = |draw: fn(usize) -> usize| {
            order_srv(records.clone(), draw)
                .into_iter()
                .map(|record| record.port)
                .collect::<Vec<_>>()
        };
        assert_eq!(ports(|_| 0), vec![1, 2, 3, 20]);
        assert_eq!(ports(|upper| upper - 1), vec![2, 1, 3, 20]);

        let zeroes = vec![srv(1, 0, 1, "a.example."), srv(1, 0, 2, "b.example.")];
        assert_eq!(order_srv(zeroes, |upper| upper - 1)[0].port, 2);
    }

    #[tokio::test]
    async fn resolves_srv_and_explicit_port_with_local_dns() {
        use hickory_resolver::{
            Resolver,
            config::{LookupIpStrategy, NameServerConfig, ResolverConfig},
            net::runtime::TokioRuntimeProvider,
            proto::{
                op::Message,
                rr::{Record, RecordType, rdata::A},
            },
        };
        use std::sync::{Arc, Mutex};

        async fn status_server(listener: tokio::net::TcpListener, name: &'static str) {
            let (mut socket, _) = listener.accept().await.unwrap();
            let length = read_varint(&mut socket).await.unwrap();
            let mut request = vec![0; length + 12];
            socket.read_exact(&mut request).await.unwrap();
            let json = format!(r#"{{"version":{{"name":"{name}"}}}}"#);
            let mut packet = vec![0];
            encode(json.len(), &mut packet);
            packet.extend_from_slice(json.as_bytes());
            let mut response = Vec::new();
            encode(packet.len(), &mut response);
            response.extend_from_slice(&packet);
            response.extend_from_slice(&[9, 1]);
            response.extend_from_slice(&0i64.to_be_bytes());
            socket.write_all(&response).await.unwrap();
        }

        let srv_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let srv_port = srv_listener.local_addr().unwrap().port();
        let explicit_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let explicit_port = explicit_listener.local_addr().unwrap().port();
        let fallback_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let fallback_port = fallback_listener.local_addr().unwrap().port();
        let srv_server = tokio::spawn(status_server(srv_listener, "srv"));
        let explicit_server = tokio::spawn(status_server(explicit_listener, "explicit"));
        let fallback_server = tokio::spawn(status_server(fallback_listener, "fallback"));
        let socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let address = socket.local_addr().unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let observed = Arc::clone(&requests);
        let server = tokio::spawn(async move {
            let mut buf = [0; 2048];
            loop {
                let (len, peer) = socket.recv_from(&mut buf).await.unwrap();
                let request = Message::from_vec(&buf[..len]).unwrap();
                let query = request.queries[0].clone();
                observed
                    .lock()
                    .unwrap()
                    .push((query.name.to_string(), query.query_type));
                if query.name.to_string() == "slow.example." {
                    continue;
                }
                let mut response = Message::response(request.metadata.id, request.metadata.op_code);
                response.add_query(query.clone());
                if query.query_type == RecordType::SRV
                    && query.name.to_string() == "_minecraft._tcp.game.example."
                {
                    response.add_answer(Record::from_rdata(
                        query.name,
                        60,
                        RData::SRV(srv(0, 1, srv_port, "target.example.")),
                    ));
                } else if query.query_type == RecordType::SRV
                    && query.name.to_string() == "_minecraft._tcp.fallback.example."
                {
                    response.add_answer(Record::from_rdata(
                        query.name.clone(),
                        60,
                        RData::SRV(srv(0, 1, 1, "slow.example.")),
                    ));
                    response.add_answer(Record::from_rdata(
                        query.name,
                        60,
                        RData::SRV(srv(1, 1, fallback_port, "target.example.")),
                    ));
                } else if query.query_type == RecordType::A {
                    response.add_answer(Record::from_rdata(
                        query.name,
                        60,
                        RData::A(A::new(127, 0, 0, 1)),
                    ));
                }
                socket
                    .send_to(&response.to_vec().unwrap(), peer)
                    .await
                    .unwrap();
            }
        });
        let mut nameserver = NameServerConfig::udp(address.ip());
        nameserver.connections[0].port = address.port();
        let mut builder = Resolver::builder_with_config(
            ResolverConfig::from_name_servers(vec![nameserver]),
            TokioRuntimeProvider::default(),
        );
        builder.options_mut().ip_strategy = LookupIpStrategy::Ipv4Only;
        let resolver = builder.build().unwrap();

        let deadline = || tokio::time::Instant::now() + Duration::from_secs(2);
        let result = query_resolved(&resolver, "game.example.", None, deadline())
            .await
            .unwrap();
        assert_eq!(result.0.version.name, "srv");
        let result = query_resolved(
            &resolver,
            "explicit.example",
            Some(explicit_port),
            deadline(),
        )
        .await
        .unwrap();
        assert_eq!(result.0.version.name, "explicit");
        let result = query_resolved(&resolver, "fallback.example", None, deadline())
            .await
            .unwrap();
        assert_eq!(result.0.version.name, "fallback");
        srv_server.await.unwrap();
        explicit_server.await.unwrap();
        fallback_server.await.unwrap();
        let requests = requests.lock().unwrap();
        assert!(
            !requests
                .iter()
                .any(|(name, kind)| name.contains("explicit.example") && *kind == RecordType::SRV)
        );
        assert!(
            requests
                .iter()
                .any(|(name, kind)| name == "_minecraft._tcp.fallback.example."
                    && *kind == RecordType::SRV)
        );
        assert!(requests.iter().any(|(name, _)| name == "slow.example."));
        assert!(
            !requests
                .iter()
                .any(|(name, kind)| name == "game.example." && *kind == RecordType::A)
        );
        server.abort();
    }

    #[test]
    fn parses_addresses() {
        assert_eq!(
            parse_address("minecraft://example.org/").unwrap(),
            ("example.org".into(), None)
        );
        assert_eq!(
            parse_address("[::1]:1234").unwrap(),
            ("::1".into(), Some(1234))
        );
        assert!(parse_address("example.org:0").is_err());
        assert!(parse_address("example.org/path").is_err());
    }

    #[tokio::test]
    async fn queries_status_and_checks_pong() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let length = read_varint(&mut socket).await.unwrap();
            let mut handshake = vec![0; length];
            socket.read_exact(&mut handshake).await.unwrap();
            assert!(handshake.windows(11).any(|part| part == b"example.org"));
            let mut request = [0; 12];
            socket.read_exact(&mut request).await.unwrap();
            assert_eq!(&request[..4], &[1, 0, 9, 1]);
            let json = br#"{"version":{"name":"test"},"description":"hello"}"#;
            let mut packet = vec![0];
            encode(json.len(), &mut packet);
            packet.extend_from_slice(json);
            let mut response = Vec::new();
            encode(packet.len(), &mut response);
            response.extend_from_slice(&packet);
            response.extend_from_slice(&[9, 1]);
            response.extend_from_slice(&0i64.to_be_bytes());
            socket.write_all(&response).await.unwrap();
        });
        let (result, _) = query("example.org", endpoint).await.unwrap();
        assert_eq!(result.version.name, "test");
        server.await.unwrap();
    }

    #[tokio::test]
    async fn tries_next_endpoint_after_timeout_and_invalid_response() {
        let stalled = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let invalid = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let valid = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoints = [
            stalled.local_addr().unwrap(),
            invalid.local_addr().unwrap(),
            valid.local_addr().unwrap(),
        ];
        let first = tokio::spawn(async move {
            let (mut socket, _) = stalled.accept().await.unwrap();
            let mut bytes = Vec::new();
            socket.read_to_end(&mut bytes).await.unwrap();
            assert!(!bytes.is_empty());
        });
        let second = tokio::spawn(async move {
            let (mut socket, _) = invalid.accept().await.unwrap();
            let mut packet = Vec::new();
            encode(MAX_PACKET + 1, &mut packet);
            socket.write_all(&packet).await.unwrap();
        });
        let third = tokio::spawn(async move {
            let (mut socket, _) = valid.accept().await.unwrap();
            let length = read_varint(&mut socket).await.unwrap();
            let mut request = vec![0; length + 12];
            socket.read_exact(&mut request).await.unwrap();
            let json = br#"{"version":{"name":"fallback"}}"#;
            let mut packet = vec![0];
            encode(json.len(), &mut packet);
            packet.extend_from_slice(json);
            let mut response = Vec::new();
            encode(packet.len(), &mut response);
            response.extend_from_slice(&packet);
            response.extend_from_slice(&[9, 1]);
            response.extend_from_slice(&0i64.to_be_bytes());
            socket.write_all(&response).await.unwrap();
        });

        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        let result = tokio::time::timeout_at(
            deadline,
            query_endpoints("example.org", &endpoints, deadline),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(result.0.version.name, "fallback");
        first.await.unwrap();
        second.await.unwrap();
        third.await.unwrap();
    }

    #[tokio::test]
    async fn endpoint_timeouts_remain_within_the_overall_deadline() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut bytes = Vec::new();
            socket.read_to_end(&mut bytes).await.unwrap();
        });
        let deadline = tokio::time::Instant::now() + Duration::from_millis(100);
        let error = query_endpoints("localhost", &[endpoint], deadline)
            .await
            .unwrap_err()
            .to_string();
        assert!(error.contains(&endpoint.to_string()));
        assert!(error.contains("deadline"));
        server.await.unwrap();
    }

    #[tokio::test]
    async fn rejects_invalid_packet_lengths() {
        let mut invalid = Cursor::new(vec![0xff, 0xff, 0xff, 0xff, 0x0f]);
        assert!(read_varint(&mut invalid).await.is_err());
    }

    #[tokio::test]
    async fn rejects_oversized_server_packet() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut length = Vec::new();
            encode(MAX_PACKET + 1, &mut length);
            socket.write_all(&length).await.unwrap();
        });
        assert!(query("localhost", endpoint).await.is_err());
        server.await.unwrap();
    }
}
