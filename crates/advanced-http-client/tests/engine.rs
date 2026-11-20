use std::time::Duration;

use advanced_http_client::{HttpEngine, RequestSpec};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const CREATED: &[u8] = b"HTTP/1.1 201 Created\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
const SERVER_ERROR: &[u8] =
    b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";

async fn read_request(socket: &mut TcpStream) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        let read = socket.read(&mut chunk).await.expect("read request");
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if let Some(end) = header_end(&buffer) {
            let head = String::from_utf8_lossy(&buffer[..end]);
            let body = content_length(&head);
            if buffer.len() >= end + 4 + body {
                break;
            }
        }
    }
    buffer
}

fn header_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

fn content_length(head: &str) -> usize {
    head.lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse().ok())
                .flatten()
        })
        .unwrap_or(0)
}

async fn serve_once(response: &'static [u8]) -> (String, tokio::task::JoinHandle<Vec<u8>>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("bind");
    let address = listener.local_addr().expect("address");
    let handle = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("accept");
        let request = read_request(&mut socket).await;
        socket.write_all(response).await.expect("write response");
        let _ = socket.shutdown().await;
        request
    });
    (format!("http://{address}/path"), handle)
}

async fn serve_hang() -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("bind");
    let address = listener.local_addr().expect("address");
    let handle = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("accept");
        let mut chunk = [0u8; 1024];
        let _ = socket.read(&mut chunk).await;
        tokio::time::sleep(Duration::from_secs(10)).await;
    });
    (format!("http://{address}/hang"), handle)
}

fn engine() -> HttpEngine {
    HttpEngine::with_timeout(Duration::from_secs(5)).expect("engine")
}

#[tokio::test]
async fn post_sends_body_and_basic_authorization() {
    let (url, server) = serve_once(CREATED).await;
    let spec = RequestSpec::new("POST", &url)
        .expect("spec")
        .with_body("{\"a\":1}")
        .with_basic_auth("user", "pass");

    let outcome = engine().execute(&spec).await.expect("request");
    assert_eq!(outcome.status.as_u16(), 201);

    let request = String::from_utf8_lossy(&server.await.expect("server")).to_ascii_lowercase();
    assert!(request.starts_with("post /path http/1.1"), "{request}");
    assert!(
        request.contains("authorization: basic dxnlcjpwyxnz"),
        "{request}"
    );
    assert!(request.contains("{\"a\":1}"), "{request}");
}

#[tokio::test]
async fn authorization_header_is_sent_verbatim() {
    let (url, server) = serve_once(CREATED).await;
    let spec = RequestSpec::new("GET", &url)
        .expect("spec")
        .with_authorization("Bearer token-123")
        .expect("header");

    engine().execute(&spec).await.expect("request");

    let response = server.await.expect("server");
    let request = String::from_utf8_lossy(&response);
    assert!(
        request
            .to_ascii_lowercase()
            .contains("authorization: bearer token-123"),
        "{request}"
    );
}

#[tokio::test]
async fn error_statuses_are_outcomes_not_transport_errors() {
    let (url, server) = serve_once(SERVER_ERROR).await;
    let spec = RequestSpec::new("GET", &url).expect("spec");

    let outcome = engine().execute(&spec).await.expect("request");
    assert_eq!(outcome.status.as_u16(), 500);
    let _ = server.await.expect("server");
}

#[tokio::test]
async fn custom_query_method_is_sent() {
    let (url, server) = serve_once(CREATED).await;
    let spec = RequestSpec::new("QUERY", &url).expect("spec");

    engine().execute(&spec).await.expect("request");

    let response = server.await.expect("server");
    let request = String::from_utf8_lossy(&response);
    assert!(request.starts_with("QUERY /path HTTP/1.1"), "{request}");
}

#[tokio::test]
async fn a_hung_request_times_out() {
    let (url, server) = serve_hang().await;
    let engine = HttpEngine::with_timeout(Duration::from_millis(100)).expect("engine");
    let spec = RequestSpec::new("GET", &url).expect("spec");

    let result = engine.execute(&spec).await;
    assert!(result.is_err(), "expected a timeout, got {result:?}");
    server.abort();
}
