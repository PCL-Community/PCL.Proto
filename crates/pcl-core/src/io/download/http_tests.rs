use super::*;
use std::sync::{
    Mutex,
    atomic::{AtomicBool, AtomicUsize},
};
use tokio::{io::AsyncReadExt, net::TcpListener, task::JoinSet};

#[derive(Clone, Copy)]
enum Behavior {
    Range,
    Ignore,
    BadRange,
    BadSegment,
    RejectRange,
    DisconnectOnce,
    Corrupt,
}
/// 每次请求记录的 `(起始, 结束)` 字节区间；`None` 表示一次无效请求。
type RequestLog = Arc<Mutex<Vec<Option<(u64, u64)>>>>;
struct Server {
    url: String,
    requests: RequestLog,
    peak: Arc<AtomicUsize>,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl Server {
    async fn new(body: Arc<Vec<u8>>, behavior: Behavior, delay: Duration) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/file.jar", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let peak = Arc::new(AtomicUsize::new(0));
        let active = Arc::new(AtomicUsize::new(0));
        let disconnected = Arc::new(AtomicBool::new(false));
        let recorded = requests.clone();
        let maximum = peak.clone();
        let task = tokio::spawn(async move {
            let mut connections = JoinSet::new();
            loop {
                tokio::select! {
                    accepted = listener.accept() => {
                        let (mut socket, _) = accepted.unwrap();
                        let body = body.clone(); let recorded = recorded.clone();
                        let active = active.clone(); let maximum = maximum.clone();
                        let disconnected = disconnected.clone();
                        connections.spawn(async move {
                            let mut request = Vec::new();
                            loop {
                                let byte = socket.read_u8().await?;
                                request.push(byte);
                                if request.ends_with(b"\r\n\r\n") { break }
                                assert!(request.len() < 8192);
                            }
                            let request = String::from_utf8(request).unwrap().to_ascii_lowercase();
                            let range = request.lines().find_map(|line| line.strip_prefix("range: bytes="))
                                .map(|value| { let (a, b) = value.split_once('-').unwrap(); (a.parse::<u64>().unwrap(), b.parse::<u64>().unwrap()) });
                            recorded.lock().unwrap().push(range);
                            let now = active.fetch_add(1, Ordering::SeqCst) + 1;
                            maximum.fetch_max(now, Ordering::SeqCst);
                            struct Active(Arc<AtomicUsize>);
                            impl Drop for Active { fn drop(&mut self) { self.0.fetch_sub(1, Ordering::SeqCst); } }
                            let _active = Active(active);
                            if range.is_some() && matches!(behavior, Behavior::RejectRange) {
                                socket.write_all(b"HTTP/1.1 416 Range Not Satisfiable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").await?;
                                return std::io::Result::Ok(());
                            }
                            let (start, end, partial) = match range {
                                Some((start, end)) if !matches!(behavior, Behavior::Ignore) => (start as usize, end as usize, true),
                                _ => (0, body.len() - 1, false),
                            };
                            let content_range = if partial { format!("Content-Range: bytes {}-{end}/{}\r\n",
                                    if matches!(behavior, Behavior::BadRange) || (matches!(behavior, Behavior::BadSegment) && end > start) { start + 1 } else { start }, body.len()) } else { String::new() };
                            let headers = format!("HTTP/1.1 {}\r\nContent-Length: {}\r\n{content_range}Connection: close\r\n\r\n",
                                if partial { "206 Partial Content" } else { "200 OK" }, end - start + 1);
                            socket.write_all(headers.as_bytes()).await?;
                            let disconnect = matches!(behavior, Behavior::DisconnectOnce) && end > start && start == 0
                                && !disconnected.swap(true, Ordering::SeqCst);
                            let end = if disconnect { start + 256 * 1024 - 1 } else { end };
                            for chunk in body[start..=end].chunks(64 * 1024) {
                                if !delay.is_zero() { tokio::time::sleep(delay).await; }
                                if matches!(behavior, Behavior::Corrupt) {
                                    socket.write_all(&vec![0; chunk.len()]).await?;
                                } else { socket.write_all(chunk).await?; }
                            }
                            std::io::Result::Ok(())
                        });
                    }
                    Some(_) = connections.join_next(), if !connections.is_empty() => {}
                }
            }
        });
        Self {
            url,
            requests,
            peak,
            task,
        }
    }
}
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("pcl-range-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn file(&self) -> PathBuf {
        self.0.join("client.jar")
    }
    fn assert_clean(&self) {
        assert!(std::fs::read_dir(&self.0).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".part")
        }));
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn payload(size: usize) -> Arc<Vec<u8>> {
    Arc::new((0..size).map(|i| (i % 251) as u8).collect())
}

async fn custom_transfer(
    url: &str,
    destination: &Path,
    on_progress: impl Fn(u64, Option<u64>, f64),
) -> anyhow::Result<()> {
    custom_transfer_checked(&Downloader::default(), url, destination, 0, on_progress).await
}

#[tokio::test]
async fn installer_minimum_size_does_not_publish_short_files() {
    let server = Server::new(payload(1024), Behavior::Ignore, Duration::ZERO).await;
    let temp = Temp::new();
    assert!(
        custom_transfer_checked(
            &Downloader::default(),
            &server.url,
            &temp.file(),
            64 * 1024,
            |_, _, _| {}
        )
        .await
        .is_err()
    );
    assert!(!temp.file().exists());
    temp.assert_clean();
}

#[tokio::test]
async fn custom_download_saves_known_length_response() {
    let body = payload(4096);
    let server = Server::new(body.clone(), Behavior::Ignore, Duration::ZERO).await;
    let temp = Temp::new();
    custom_transfer(&server.url, &temp.file(), |_, _, _| {})
        .await
        .unwrap();
    assert_eq!(std::fs::read(temp.file()).unwrap(), *body);
    temp.assert_clean();
}

#[tokio::test]
async fn custom_download_parallelizes_and_recovers_interrupted_ranges() {
    let body = payload(24 * 1024 * 1024);
    let server = Server::new(
        body.clone(),
        Behavior::DisconnectOnce,
        Duration::from_millis(1),
    )
    .await;
    let temp = Temp::new();
    let downloader = downloader(2, 3);
    download_unknown(&downloader, &server.url, &temp.file())
        .await
        .unwrap();
    assert_eq!(std::fs::read(temp.file()).unwrap(), *body);
    assert_eq!(server.peak.load(Ordering::SeqCst), 2);
    assert!(
        server
            .requests
            .lock()
            .unwrap()
            .iter()
            .any(|range| range.is_some_and(|(start, _)| start > 0 && start < 8 * 1024 * 1024))
    );
    assert_eq!(downloader.resources.stats().active_connections, 0);
}

#[tokio::test]
async fn custom_download_falls_back_when_ranges_are_unreliable() {
    let body = payload(4 * 1024 * 1024);
    for behavior in [
        Behavior::Ignore,
        Behavior::BadRange,
        Behavior::RejectRange,
        Behavior::BadSegment,
    ] {
        let server = Server::new(body.clone(), behavior, Duration::ZERO).await;
        let temp = Temp::new();
        let downloader = downloader(2, 2);
        download_unknown(&downloader, &server.url, &temp.file())
            .await
            .unwrap();
        assert_eq!(std::fs::read(temp.file()).unwrap(), *body);
        assert_eq!(server.requests.lock().unwrap().last(), Some(&None));
    }
}

#[tokio::test]
async fn custom_download_handles_unknown_length_and_preserves_existing_files() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/file", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        for _ in 0..2 {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0; 4096];
            let _ = socket.read(&mut request).await.unwrap();
            let _ = socket
                .write_all(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\nhello")
                .await;
        }
    });
    let temp = Temp::new();
    custom_transfer(&url, &temp.file(), |_, _, _| {})
        .await
        .unwrap();
    server.await.unwrap();
    assert_eq!(std::fs::read(temp.file()).unwrap(), b"hello");
    assert!(
        custom_transfer(&url, &temp.file(), |_, _, _| {})
            .await
            .is_err()
    );
    assert_eq!(std::fs::read(temp.file()).unwrap(), b"hello");
    temp.assert_clean();
}

#[tokio::test]
async fn custom_download_cleans_up_after_truncated_response() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/file", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        for _ in 0..2 {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0; 4096];
            let _ = socket.read(&mut request).await.unwrap();
            let _ = socket
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 20\r\nConnection: close\r\n\r\nshort",
                )
                .await;
        }
    });
    let temp = Temp::new();
    assert!(
        custom_transfer(&url, &temp.file(), |_, _, _| {})
            .await
            .is_err()
    );
    server.await.unwrap();
    assert!(!temp.file().exists());
    temp.assert_clean();
}

#[tokio::test]
async fn custom_download_retries_transient_http_failures() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/file", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        for attempt in 0..4 {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0; 4096];
            let _ = socket.read(&mut request).await.unwrap();
            let response: &[u8] = if attempt < 2 {
                b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            } else {
                b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nhello"
            };
            let _ = socket.write_all(response).await;
        }
    });
    let temp = Temp::new();
    custom_transfer(&url, &temp.file(), |_, _, _| {})
        .await
        .unwrap();
    server.await.unwrap();
    assert_eq!(std::fs::read(temp.file()).unwrap(), b"hello");
    temp.assert_clean();
}

fn info(server: &Server, body: &[u8]) -> DownloadFile {
    DownloadFile {
        url: server.url.clone(),
        size: body.len() as u64,
        sha1: crate::io::files::sha1_hex(sha1::Sha1::digest(body)),
        path: None,
    }
}
fn downloader(max: usize, file: usize) -> Downloader {
    let downloader = Downloader {
        resources: Arc::new(DownloadResources::default()),
        ..Default::default()
    };
    downloader
        .resources
        .configure(resources::DownloadOptions {
            max_connections: max,
            file_connections: file,
            ..Default::default()
        })
        .unwrap();
    downloader
}

async fn download_unknown(
    downloader: &Downloader,
    url: &str,
    path: &Path,
) -> Result<(), McApiError> {
    downloader
        .download_with_progress(
            DownloadConfig {
                file_index: 0,
                out_path: path.to_owned(),
                task_item_id: 0,
                info: DownloadFile {
                    url: url.into(),
                    size: 0,
                    sha1: String::new(),
                    path: None,
                },
            },
            |_, _, _| {},
        )
        .await
}

#[tokio::test]
async fn resource_without_hash_still_checks_size_and_cleans_failed_transfers() {
    let body = payload(1024);
    let server = Server::new(body.clone(), Behavior::Ignore, Duration::ZERO).await;
    let mut metadata = info(&server, &body);
    metadata.sha1.clear();
    let dir = Temp::new();
    let downloader = downloader(2, 1);
    let (tx, mut rx) = mpsc::channel(100);
    let config = |info| DownloadConfig {
        file_index: 0,
        out_path: dir.file(),
        task_item_id: 0,
        info,
    };
    downloader
        .start_download(config(metadata.clone()), tx)
        .await
        .unwrap();
    while rx.try_recv().is_ok() {}
    assert_eq!(std::fs::read(dir.file()).unwrap(), *body);
    std::fs::remove_file(dir.file()).unwrap();
    metadata.size += 1;
    let (tx, _rx) = mpsc::channel(100);
    assert!(
        downloader
            .start_download(config(metadata), tx)
            .await
            .is_err()
    );
    assert!(!dir.file().exists());
    dir.assert_clean();
}

#[tokio::test]
async fn parallel_ranges_resume_partial_segments_and_share_the_global_quota() {
    let body = payload(24 * 1024 * 1024);
    let server = Server::new(
        body.clone(),
        Behavior::DisconnectOnce,
        Duration::from_millis(1),
    )
    .await;
    let info = info(&server, &body);
    let first = Temp::new();
    let second = Temp::new();
    let downloader = downloader(2, 3);
    let other = downloader.clone();
    let first_path = first.file();
    let second_path = second.file();
    let (a, b) = tokio::join!(
        downloader.download(&info, &first_path),
        other.download(&info, &second_path)
    );
    a.unwrap();
    b.unwrap();
    assert_eq!(std::fs::read(first.file()).unwrap(), *body);
    assert_eq!(std::fs::read(second.file()).unwrap(), *body);
    assert!(
        server
            .requests
            .lock()
            .unwrap()
            .iter()
            .any(|range| range.is_some_and(|(start, _)| start > 0 && start < 8 * 1024 * 1024)),
        "failed segment resumes past bytes already written"
    );
    assert_eq!(server.peak.load(Ordering::SeqCst), 2);
    assert_eq!(downloader.resources.stats().active_connections, 0);
    assert!(downloader.resources.stats().downloaded_bytes >= info.size * 2 + 2);
    first.assert_clean();
    second.assert_clean();
}

#[tokio::test]
async fn unreliable_range_servers_fall_back_to_one_sequential_request() {
    let body = payload(4 * 1024 * 1024);
    for behavior in [
        Behavior::Ignore,
        Behavior::BadRange,
        Behavior::RejectRange,
        Behavior::BadSegment,
    ] {
        let server = Server::new(body.clone(), behavior, Duration::ZERO).await;
        let info = info(&server, &body);
        let output = Temp::new();
        let downloader = downloader(1, 8);
        tokio::time::timeout(
            Duration::from_secs(5),
            downloader.download(&info, &output.file()),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(std::fs::read(output.file()).unwrap(), *body);
        let expected = if matches!(behavior, Behavior::BadSegment) {
            vec![Some((0, 0)), Some((0, info.size - 1)), None]
        } else {
            vec![Some((0, 0)), None]
        };
        assert_eq!(*server.requests.lock().unwrap(), expected);
        output.assert_clean();
    }
}

#[tokio::test]
async fn per_file_limit_bounds_workers_even_with_a_larger_global_quota() {
    let body = payload(24 * 1024 * 1024);
    let server = Server::new(body.clone(), Behavior::Range, Duration::from_millis(1)).await;
    let info = info(&server, &body);
    let output = Temp::new();
    let downloader = downloader(64, 2);
    downloader.download(&info, &output.file()).await.unwrap();
    assert_eq!(std::fs::read(output.file()).unwrap(), *body);
    assert_eq!(server.peak.load(Ordering::SeqCst), 2);
}

#[tokio::test]
#[ignore = "Downloads the official 1.21.1 client into a disposable directory"]
async fn official_large_client_range_smoke() {
    use crate::io::net::http::api_client::{ApiProvider, MinecraftApiClient};
    let client = MinecraftApiClient::new(
        crate::io::download::Downloader::default(),
        &ApiProvider::Official,
    );
    let metadata = client.get_version_metadata("1.21.1").await.unwrap();
    let info: DownloadFile =
        serde_json::from_value(metadata["downloads"]["client"].clone()).unwrap();
    let output = Temp::new();
    let downloader = downloader(64, 8);
    let started = std::time::Instant::now();
    downloader.download(&info, &output.file()).await.unwrap();
    assert!(crate::io::files::check_sha1(&output.file(), &info.sha1).unwrap());
    eprintln!(
        "Official 1.21.1 client: size={}, network_bytes={}, elapsed={:.3}s",
        info.size,
        downloader.resources.stats().downloaded_bytes,
        started.elapsed().as_secs_f64()
    );
    output.assert_clean();
}

#[tokio::test]
async fn corruption_and_cancellation_preserve_existing_files_and_release_connections() {
    let body = payload(4 * 1024 * 1024);
    for (behavior, delay) in [
        (Behavior::Corrupt, Duration::ZERO),
        (Behavior::Range, Duration::from_millis(20)),
    ] {
        let server = Server::new(body.clone(), behavior, delay).await;
        let info = info(&server, &body);
        let output = Temp::new();
        std::fs::write(output.file(), b"existing file").unwrap();
        let downloader = downloader(8, 8);
        let result = tokio::time::timeout(
            Duration::from_millis(if delay.is_zero() { 5000 } else { 150 }),
            downloader.download(&info, &output.file()),
        )
        .await;
        if delay.is_zero() {
            assert!(matches!(result.unwrap(), Err(McApiError::Sha1Mismatch(_))));
        } else {
            assert!(result.is_err());
        }
        assert_eq!(std::fs::read(output.file()).unwrap(), b"existing file");
        assert_eq!(downloader.resources.stats().active_connections, 0);
        output.assert_clean();
    }
}

#[tokio::test]
async fn small_files_do_not_probe_and_cache_hits_do_not_add_traffic() {
    let body = payload(1024);
    let server = Server::new(body.clone(), Behavior::Range, Duration::ZERO).await;
    let info = info(&server, &body);
    let output = Temp::new();
    let downloader = downloader(64, 8);
    downloader.download(&info, &output.file()).await.unwrap();
    downloader.download(&info, &output.file()).await.unwrap();
    assert_eq!(*server.requests.lock().unwrap(), vec![None]);
    assert_eq!(downloader.resources.stats().downloaded_bytes, info.size);
}

#[tokio::test]
#[ignore = "Controlled per-request bandwidth benchmark, not a real CE benchmark"]
async fn controlled_range_benchmark() {
    let body = payload(32 * 1024 * 1024);
    let server = Server::new(body.clone(), Behavior::Range, Duration::from_millis(4)).await;
    let info = info(&server, &body);
    for connections in [1, 8] {
        let output = Temp::new();
        let downloader = downloader(64, connections);
        let started = std::time::Instant::now();
        downloader.download(&info, &output.file()).await.unwrap();
        assert_eq!(std::fs::read(output.file()).unwrap(), *body);
        eprintln!(
            "32 MiB per-request-limited server: file_connections={connections}, elapsed={:.3}s, network_bytes={}",
            started.elapsed().as_secs_f64(),
            downloader.resources.stats().downloaded_bytes
        );
    }
}

#[tokio::test]
#[ignore = "Controlled custom-download bandwidth benchmark"]
async fn controlled_custom_download_benchmark() {
    let body = payload(32 * 1024 * 1024);
    let server = Server::new(body.clone(), Behavior::Range, Duration::from_millis(4)).await;
    for connections in [1, 8] {
        let output = Temp::new();
        let downloader = downloader(64, connections);
        let started = std::time::Instant::now();
        download_unknown(&downloader, &server.url, &output.file())
            .await
            .unwrap();
        let elapsed = started.elapsed();
        assert_eq!(std::fs::read(output.file()).unwrap(), *body);
        eprintln!(
            "Custom 32 MiB: connections={connections}, elapsed={:.3}s",
            elapsed.as_secs_f64()
        );
    }
}
