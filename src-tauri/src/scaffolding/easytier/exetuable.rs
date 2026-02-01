use std::{
    env,
    fmt::Write,
    io::{self, BufRead, BufReader},
    net::{Ipv4Addr, TcpListener},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
};

struct EasytierFactory {
    core: PathBuf,
    cli: PathBuf,
}

pub struct EasyTier {
    process: Arc<Mutex<Child>>,
    rpc: u16,
    receiver: Option<mpsc::Receiver<String>>,
}

// 请求一个可用的端口
fn request_port() -> io::Result<u16> {
    TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .and_then(|socket| socket.local_addr())
        .map(|address| address.port())
}

fn forward_std<F>(process: &mut Child, handle: F)
where
    F: Fn(String) + Send + Sized + Clone + 'static,
{
    let handle_err = handle.clone();

    let stdout = process.stdout.take().unwrap();
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            handle(line);
        }
    });

    let stderr = process.stderr.take().unwrap();
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            handle_err(line);
        }
    });
}

impl EasyTier {
    fn new(factory: &EasytierFactory) -> io::Result<Self> {
        let rpc = request_port()?;
        log::info!("starting easytier with rpc port: {}", rpc);
        let mut command = Command::new(factory.core.as_path());
        command
            .args(["-r", &rpc.to_string()])
            .current_dir(env::temp_dir())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn()?;
        let (sender, receiver) = mpsc::channel::<String>();
        forward_std(&mut child, move |line| {
            let _ = sender.send(line);
        });
        let process: Arc<Mutex<Child>> = Arc::new(Mutex::new(child));
        Ok(Self {
            process,
            rpc,
            receiver: Some(receiver),
        })
    }

    fn run(&mut self) {
        let process = self.process.clone();
        let receiver = self.receiver.take().expect("EasyTier::run() called multiple times");
        const LINES: usize = 500;
        thread::spawn(move || {
            let mut buffer: [Option<String>; LINES] = [const { None }; LINES];
            let mut index = 0;
            let status = 'status: loop {
                match receiver.recv_timeout(Duration::from_millis(100)) {
                    Ok(value) => {
                        buffer[index] = Some(value);
                        index = (index + 1) % LINES;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        match process.lock().unwrap().try_wait() {
                            Ok(Some(status)) => break 'status Some(status),
                            Ok(None) => {
                                log::error!(
                                    "[EasyTier] Cannot fetch EasyTier process status: EasyTier hasn't exited."
                                );
                            }
                            Err(e) => {
                                log::error!(
                                    "[EasyTier] Cannot fetch EasyTier process status: {:?}",
                                    e
                                );
                            }
                        }
                        break 'status None;
                    }
                }
            };

            let mut output = String::from("Easytier has exited. with status ");

            match status {
                Some(status) => match status.code() {
                    Some(code) => write!(output, "code={}, success={}", code, status.success()),
                    None => write!(output, "code=[unknown], success={}", status.success()),
                }
                .unwrap(),
                None => output.push_str("[unknown]"),
            }
            output.push_str(
                ". Here's the logs:\n############################################################",
            );
            for i in 0..LINES {
                if let Some(value) = &buffer[(index + 1 + i) % LINES] {
                    output.push_str("\n    ");
                    output.push_str(value);
                }
            }
            output.push_str("\n############################################################");
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_request_port() -> io::Result<()> {
        let port = request_port()?;
        println!("port: {}", port);
        assert!(port > 0);
        Ok(())
    }

    #[test]
    fn test_start_easytier() {
        let factory = EasytierFactory {
            core: PathBuf::from("/Users/amagicpear/自己的一些可执行程序/easytier-core"),
            cli: PathBuf::from("/Users/amagicpear/自己的一些可执行程序/easytier-cli"),
        };
        let easytier = EasyTier::new(&factory);
        if let Err(ref e) = easytier {
            println!("Failed to start easytier: {:?}", e);
            assert!(false);
        } else if let Ok(mut easytier) = easytier {
            println!("easytier started with rpc port: {}", easytier.rpc);
            assert!(easytier.rpc > 0);
            easytier.run();
        }
    }
}
