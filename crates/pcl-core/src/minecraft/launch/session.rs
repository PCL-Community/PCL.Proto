use std::{
    collections::VecDeque,
    io::{self, Read},
    path::PathBuf,
    process::Child,
    sync::{
        Arc, LazyLock, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};

const MAX_LINES: usize = 1000;
const MAX_LINE_BYTES: usize = 8192;
const MAX_LOG_BYTES: usize = 1024 * 1024;
const MAX_FINISHED: usize = 8;
const LOADER_SOLUTION_MARKERS: [&str; 3] = [
    "A potential solution has been determined, this may resolve your problem:",
    "A potential solution has been determined:",
    "确定了一种可能的解决方法，这样做可能会解决你的问题：",
];

#[derive(Clone, serde::Serialize)]
pub struct GameSession {
    id: String,
    instance_directory: PathBuf,
    pid: u32,
    running: bool,
    phase: SessionPhase,
    exit_code: Option<i32>,
    error: Option<String>,
    lines: VecDeque<String>,
}

#[derive(Clone, Copy, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum SessionPhase {
    Starting,
    Running,
    Exited,
    Failed,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct LaunchFailure {
    pub exit_code: Option<i32>,
    pub details: Option<String>,
    pub solution: Option<String>,
}

fn loader_failure(lines: &VecDeque<String>, exit_code: Option<i32>) -> LaunchFailure {
    let output = lines
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("\n");
    let details = output
        .find("Incompatible mods found!")
        .and_then(|start| output[start..].split_once(": ").map(|(_, detail)| detail))
        .and_then(|detail| detail.split("\n\tat ").next())
        .map(|detail| {
            LOADER_SOLUTION_MARKERS
                .iter()
                .find_map(|marker| detail.split_once(marker).map(|(value, _)| value))
                .unwrap_or(detail)
                .trim()
        })
        .filter(|detail| !detail.is_empty())
        .map(ToOwned::to_owned);
    let solution = LOADER_SOLUTION_MARKERS
        .iter()
        .find_map(|marker| output.split_once(marker).map(|(_, value)| value))
        .map(|value| {
            value
                .lines()
                .map(str::trim)
                .skip_while(|line| line.is_empty())
                .take_while(|line| line.starts_with('-'))
                .map(|line| line.trim_start_matches('-').trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .filter(|solution| !solution.is_empty());
    LaunchFailure {
        exit_code,
        details,
        solution,
    }
}

#[derive(Clone, Debug)]
pub enum SessionEvent {
    Spawned,
    Ready,
    Finished {
        safe_exit: bool,
        failure: Option<LaunchFailure>,
    },
}

#[derive(Clone, Copy)]
enum Signal {
    Ready,
    Crash,
    ReaderDone(bool),
}

struct Entry {
    snapshot: GameSession,
    stop: Arc<AtomicBool>,
    log_bytes: usize,
}

static SESSIONS: LazyLock<Mutex<VecDeque<Entry>>> = LazyLock::new(|| Mutex::new(VecDeque::new()));

fn append(id: &str, line: String) {
    let mut sessions = SESSIONS.lock().unwrap();
    if let Some(entry) = sessions.iter_mut().find(|entry| entry.snapshot.id == id) {
        while !entry.snapshot.lines.is_empty()
            && (entry.snapshot.lines.len() >= MAX_LINES
                || entry.log_bytes + line.len() > MAX_LOG_BYTES)
        {
            entry.log_bytes -= entry.snapshot.lines.pop_front().unwrap().len();
        }
        entry.log_bytes += line.len();
        entry.snapshot.lines.push_back(line);
    }
}

fn read_output(
    mut input: impl Read,
    id: &str,
    secret: Option<&str>,
    signals: Option<&mpsc::Sender<Signal>>,
) -> io::Result<()> {
    let mut buffer = [0; 4096];
    let mut line = Vec::new();
    let mut oversized = false;
    let emit = |line: &mut Vec<u8>, oversized: &mut bool| {
        let value = if *oversized {
            "[Log line exceeded size limit]".into()
        } else {
            let value = String::from_utf8_lossy(line)
                .trim_end_matches('\r')
                .to_owned();
            match secret.filter(|s| !s.is_empty()) {
                Some(secret) => value.replace(secret, "[redacted]"),
                None => value,
            }
        };
        if !value.contains("[CHAT]") {
            let signal = if value.contains("OpenAL initialized")
                || value.contains("Starting up SoundSystem")
                || (value.contains("Created")
                    && value.contains("textures")
                    && value.contains("-atlas"))
                || value.contains("Found animation info")
            {
                Some(Signal::Ready)
            } else if value.contains("Crash report saved to")
                || value.contains("This crash report has been saved to:")
                || value.contains("Could not save crash report to")
                || value.contains("/ERROR]: Unable to launch")
                || value.contains(
                    "An exception was thrown, the game will display an error screen and halt.",
                )
            {
                Some(Signal::Crash)
            } else {
                None
            };
            if let (Some(signals), Some(signal)) = (signals, signal) {
                let _ = signals.send(signal);
            }
        }
        append(id, value);
        line.clear();
        *oversized = false;
    };
    loop {
        let read = input.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        for byte in &buffer[..read] {
            if *byte == b'\n' {
                emit(&mut line, &mut oversized);
            } else if !oversized {
                if line.len() == MAX_LINE_BYTES {
                    line.clear();
                    oversized = true;
                } else {
                    line.push(*byte);
                }
            }
        }
    }
    if !line.is_empty() || oversized {
        emit(&mut line, &mut oversized);
    }
    Ok(())
}

pub fn track(
    mut child: Child,
    instance_directory: PathBuf,
    secret: Option<String>,
    cleanup: impl FnOnce() + Send + 'static,
    mut on_event: impl FnMut(&str, SessionEvent) + Send + 'static,
) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let stop = Arc::new(AtomicBool::new(false));
    {
        let mut sessions = SESSIONS.lock().unwrap();
        while sessions
            .iter()
            .filter(|entry| !entry.snapshot.running)
            .count()
            >= MAX_FINISHED
        {
            if let Some(index) = sessions.iter().position(|entry| !entry.snapshot.running) {
                sessions.remove(index);
            }
        }
        sessions.push_back(Entry {
            snapshot: GameSession {
                id: id.clone(),
                instance_directory,
                pid: child.id(),
                running: true,
                phase: SessionPhase::Starting,
                exit_code: None,
                error: None,
                lines: VecDeque::new(),
            },
            stop: stop.clone(),
            log_bytes: 0,
        });
    }
    on_event(&id, SessionEvent::Spawned);
    let (sender, receiver) = mpsc::channel();
    let pipes: Vec<_> = [
        child
            .stdout
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
        child
            .stderr
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    ]
    .into_iter()
    .flatten()
    .collect();
    let reader_count = pipes.len();
    for pipe in pipes {
        let id = id.clone();
        let secret = secret.clone();
        let sender = sender.clone();
        std::thread::spawn(move || {
            let result = read_output(pipe, &id, secret.as_deref(), Some(&sender));
            if let Err(error) = &result {
                log::warn!("Cannot read game output: {error}");
            }
            let _ = sender.send(Signal::ReaderDone(result.is_ok()));
        });
    }
    drop(sender);
    let worker_id = id.clone();
    std::thread::spawn(move || {
        let id = worker_id;
        let mut ready = false;
        let mut crashed = false;
        let mut stopped = false;
        let mut output_incomplete = false;
        let mut readers_done = 0;
        let result = loop {
            if stop.swap(false, Ordering::Relaxed) {
                stopped = true;
                if let Err(error) = child.kill() {
                    append(&id, format!("Cannot stop game: {error}"));
                }
            }
            match child.try_wait() {
                Ok(Some(status)) => break Ok(status.code()),
                Ok(None) => match receiver.recv_timeout(Duration::from_millis(100)) {
                    Ok(Signal::Ready) if !ready && !crashed && !stopped => {
                        if matches!(child.try_wait(), Ok(None)) {
                            ready = true;
                            if let Some(entry) = SESSIONS
                                .lock()
                                .unwrap()
                                .iter_mut()
                                .find(|entry| entry.snapshot.id == id)
                            {
                                entry.snapshot.phase = SessionPhase::Running;
                            }
                            on_event(&id, SessionEvent::Ready);
                        }
                    }
                    Ok(Signal::Crash) => crashed = true,
                    Ok(Signal::ReaderDone(ok)) => {
                        readers_done += 1;
                        output_incomplete |= !ok;
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        std::thread::sleep(Duration::from_millis(100))
                    }
                    _ => {}
                },
                Err(error) => break Err(error.to_string()),
            }
        };
        let deadline = std::time::Instant::now() + Duration::from_secs(1);
        while readers_done < reader_count {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            match receiver.recv_timeout(remaining) {
                Ok(Signal::Crash) => crashed = true,
                Ok(Signal::ReaderDone(ok)) => {
                    readers_done += 1;
                    output_incomplete |= !ok;
                }
                Ok(Signal::Ready) => {}
                Err(_) => break,
            }
        }
        cleanup();
        let exit_code = result.as_ref().ok().copied().flatten();
        let safe_exit = matches!(exit_code, Some(0))
            && ready
            && !crashed
            && !stopped
            && !output_incomplete
            && readers_done == reader_count;
        let failure = (!ready && !stopped && !safe_exit).then(|| {
            let sessions = SESSIONS.lock().unwrap();
            let lines = sessions
                .iter()
                .find(|entry| entry.snapshot.id == id)
                .map(|entry| &entry.snapshot.lines)
                .cloned()
                .unwrap_or_default();
            loader_failure(&lines, exit_code)
        });
        let mut sessions = SESSIONS.lock().unwrap();
        if let Some(entry) = sessions.iter_mut().find(|entry| entry.snapshot.id == id) {
            entry.snapshot.running = false;
            entry.snapshot.phase =
                if matches!(exit_code, Some(0)) && !crashed && !stopped && failure.is_none() {
                    SessionPhase::Exited
                } else {
                    SessionPhase::Failed
                };
            match result {
                Ok(code) => entry.snapshot.exit_code = code,
                Err(error) => entry.snapshot.error = Some(error),
            }
        }
        while sessions
            .iter()
            .filter(|entry| !entry.snapshot.running)
            .count()
            > MAX_FINISHED
        {
            if let Some(index) = sessions.iter().position(|entry| !entry.snapshot.running) {
                sessions.remove(index);
            }
        }
        drop(sessions);
        on_event(&id, SessionEvent::Finished { safe_exit, failure });
    });
    id
}

pub fn get_game_sessions(id: Option<String>) -> Vec<GameSession> {
    let sessions = SESSIONS.lock().unwrap();
    let selected = id
        .as_deref()
        .or_else(|| sessions.back().map(|entry| entry.snapshot.id.as_str()));
    sessions
        .iter()
        .rev()
        .map(|entry| {
            let mut snapshot = entry.snapshot.clone();
            if Some(snapshot.id.as_str()) != selected {
                snapshot.lines.clear();
            }
            snapshot
        })
        .collect()
}

pub fn stop_game_session(id: String) -> Result<(), String> {
    let sessions = SESSIONS.lock().map_err(|e| e.to_string())?;
    let entry = sessions
        .iter()
        .find(|entry| entry.snapshot.id == id)
        .ok_or("Game session not found")?;
    if !entry.snapshot.running {
        return Err("Game process has already exited".into());
    }
    entry.stop.store(true, Ordering::Relaxed);
    Ok(())
}

pub fn clear_game_session_log(id: String) -> Result<(), String> {
    let mut sessions = SESSIONS.lock().map_err(|e| e.to_string())?;
    let entry = sessions
        .iter_mut()
        .find(|entry| entry.snapshot.id == id)
        .ok_or("Game session not found")?;
    entry.snapshot.lines.clear();
    entry.log_bytes = 0;
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::{
        process::{Command, Stdio},
        sync::atomic::AtomicUsize,
        time::Instant,
    };

    fn wait_for(id: &str, condition: impl Fn(&GameSession) -> bool) -> GameSession {
        let start = Instant::now();
        loop {
            if let Some(session) = get_game_sessions(Some(id.into()))
                .into_iter()
                .find(|session| session.id == id)
                && condition(&session)
            {
                return session;
            }
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "session did not reach expected state"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn child(script: &str) -> Child {
        Command::new("/bin/sh")
            .args(["-c", script])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap()
    }

    #[test]
    fn captures_bounded_output_redacts_token_and_reports_exit() {
        let cleaned = Arc::new(AtomicUsize::new(0));
        let completed = cleaned.clone();
        let id = track(
            child("printf 'value secret-token\\n'; printf 'error line\\n' >&2; exit 7"),
            PathBuf::from("/test/instance"),
            Some("secret-token".into()),
            move || {
                completed.fetch_add(1, Ordering::SeqCst);
            },
            |_, _| {},
        );
        let session = wait_for(&id, |session| !session.running && session.lines.len() >= 2);
        assert_eq!(session.exit_code, Some(7));
        assert!(session.lines.iter().any(|line| line == "value [redacted]"));
        assert!(session.lines.iter().any(|line| line == "error line"));
        assert_eq!(cleaned.load(Ordering::SeqCst), 1);
        assert!(stop_game_session(id.clone()).is_err());
        read_output(
            format!(
                "{}\n{}\n",
                "x".repeat(MAX_LINE_BYTES + 1),
                "line\n".repeat(MAX_LINES + 1)
            )
            .as_bytes(),
            &id,
            None,
            None,
        )
        .unwrap();
        assert_eq!(
            wait_for(&id, |session| session.lines.len() == MAX_LINES)
                .lines
                .len(),
            MAX_LINES
        );
        clear_game_session_log(id.clone()).unwrap();
        assert!(
            wait_for(&id, |session| session.lines.is_empty())
                .lines
                .is_empty()
        );
    }

    #[test]
    fn stop_targets_only_the_selected_child() {
        let first = track(
            child("exec sleep 30"),
            PathBuf::from("/test/first"),
            None,
            || {},
            |_, _| {},
        );
        let second = track(
            child("exec sleep 30"),
            PathBuf::from("/test/second"),
            None,
            || {},
            |_, _| {},
        );
        stop_game_session(first.clone()).unwrap();
        wait_for(&first, |session| !session.running);
        assert!(wait_for(&second, |session| session.running).running);
        stop_game_session(second.clone()).unwrap();
        wait_for(&second, |session| !session.running);
    }

    #[test]
    fn emits_ready_then_safe_exit_for_a_successful_game() {
        let (sender, receiver) = mpsc::channel();
        let id = track(
            child("printf 'OpenAL initialized\\n'; sleep 0.3; exit 0"),
            PathBuf::from("/test/ready"),
            None,
            || {},
            move |_, event| {
                sender.send(event).unwrap();
            },
        );
        assert!(matches!(
            receiver.recv_timeout(Duration::from_secs(5)).unwrap(),
            SessionEvent::Spawned
        ));
        assert!(matches!(
            receiver.recv_timeout(Duration::from_secs(5)).unwrap(),
            SessionEvent::Ready
        ));
        assert!(matches!(
            receiver.recv_timeout(Duration::from_secs(5)).unwrap(),
            SessionEvent::Finished {
                safe_exit: true,
                failure: None
            }
        ));
        wait_for(&id, |session| !session.running);
    }

    #[test]
    fn reports_loader_solution_when_game_exits_before_ready() {
        let (sender, receiver) = mpsc::channel();
        let id = track(
            child(
                "printf '有不兼容的模组！ (Incompatible mods found!)\\nnet.fabricmc.loader.impl.FormattedException: Some of your mods are incompatible with the game or each other!\\n确定了一种可能的解决方法，这样做可能会解决你的问题：\\n - 安装 fabric-api，任意版本。\\n更多信息：\\n - 模组需要 fabric-api。\\n\\tat net.fabricmc.loader.impl.FabricLoaderImpl.load(FabricLoaderImpl.java:207)\\n' >&2; exit 1",
            ),
            PathBuf::from("/test/fabric"),
            None,
            || {},
            move |_, event| {
                sender.send(event).unwrap();
            },
        );
        assert!(matches!(
            receiver.recv_timeout(Duration::from_secs(5)).unwrap(),
            SessionEvent::Spawned
        ));
        let SessionEvent::Finished { failure, .. } =
            receiver.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("expected failed game session");
        };
        let failure = failure.expect("expected an early-exit failure");
        assert_eq!(failure.exit_code, Some(1));
        assert_eq!(
            failure.details.as_deref(),
            Some("Some of your mods are incompatible with the game or each other!")
        );
        assert_eq!(
            failure.solution.as_deref(),
            Some("安装 fabric-api，任意版本。")
        );
        wait_for(&id, |session| !session.running);
    }
}
