use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::thread;

const BIN: &str = env!("CARGO_BIN_EXE_filefile");

/// Spawn a one-shot HTTP server that serves `body` to the next request.
/// Returns the URL clients should hit.
fn serve_once(body: &str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let body = body.to_string();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0u8; 4096];
        let _ = stream.read(&mut buf);
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = stream.write_all(resp.as_bytes());
    });
    format!("http://{}/Filefile.yaml", addr)
}

#[test]
fn ff_applies_a_remote_filefile() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let url = serve_once("greet:\n  hello: \"world\"\n");

    let status = Command::new(BIN)
        .current_dir(root)
        .arg(&url)
        .status()
        .unwrap();
    assert!(status.success(), "ff <url> exited non-zero");

    assert!(root.join("greet").is_dir());
    assert_eq!(fs::read_to_string(root.join("greet/hello")).unwrap(), "world");
}
