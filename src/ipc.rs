use std::env;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
pub fn socket_path() -> PathBuf {
    let runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(runtime_dir).join("island_launcher.sock")
}

pub fn notify_running_instance(path: &PathBuf) -> bool {
    match UnixStream::connect(path) {
        Ok(mut stream) => {
            if let Err(e) = stream.write_all(b"T") {
                eprintln!(
                    "Warning: Connected to daemon but failed to send toggle signal: {}",
                    e
                );
            }
            true
        }
        Err(_) => false,
    }
}

pub fn bind_listener(path: &PathBuf) -> std::io::Result<UnixListener> {
    if path.exists()
            && let Err(e) = fs::remove_file(path)
        {
            eprintln!("Warning: Failed to remove old socket file: {}", e);
        }

    UnixListener::bind(path)
}

pub fn serve<F>(listener: UnixListener, mut on_toggle: F)
where
    F: FnMut() + Send + 'static,
{
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf = [0; 1];
                // Read 1 byte from the stream
                if let Ok(1) = stream.read(&mut buf)
                    && buf[0] == b'T'
                {
                    on_toggle();
                }
            }
            Err(e) => {
                eprintln!("IPC connection failed: {}", e);
            }
        }
    }
}
