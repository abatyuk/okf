#![cfg(feature = "url-sources")]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use okf_core::ports::net::{Net, RealNet};
use okf_core::query::artifact::fetch_artifact;

fn serve(status: &str, body: &str) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/artifact", listener.local_addr().unwrap());
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nETag: \"v1\"\r\nLast-Modified: Wed, 01 Jan 2025 00:00:00 GMT\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut request = Vec::new();
        while !request.ends_with(b"\r\n\r\n") {
            let mut byte = [0];
            stream.read_exact(&mut byte).unwrap();
            request.push(byte[0]);
        }
        stream.write_all(response.as_bytes()).unwrap();
    });
    (url, handle)
}

#[test]
fn fetch_preserves_validators_and_body() {
    let (url, server) = serve("200 OK", "abc");
    let response = RealNet.fetch(&url).unwrap();
    server.join().unwrap();
    assert_eq!(response.etag.as_deref(), Some("\"v1\""));
    assert_eq!(
        response.last_modified.as_deref(),
        Some("Wed, 01 Jan 2025 00:00:00 GMT")
    );
    assert_eq!(response.body, b"abc");
}

#[test]
fn artifact_fetch_preserves_limits_and_digest() {
    for (body, truncated) in [("abc", false), ("abcdef", true)] {
        let (url, server) = serve("200 OK", body);
        let artifact = fetch_artifact(&url, 3).unwrap();
        server.join().unwrap();
        assert_eq!(artifact.text.as_deref(), Some("abc"));
        assert_eq!(artifact.truncated, truncated);
        assert_eq!(
            artifact.sha256,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}

#[test]
fn fetch_rejects_http_errors() {
    let (url, server) = serve("404 Not Found", "missing");
    assert!(RealNet.fetch(&url).is_err());
    server.join().unwrap();
    let (url, server) = serve("404 Not Found", "missing");
    assert!(fetch_artifact(&url, 100).is_err());
    server.join().unwrap();
}
