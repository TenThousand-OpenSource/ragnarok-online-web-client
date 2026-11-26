//! The Adventurer's Agency: the client's requests to the game's web service, handed on to it.
//!
//! The Windows client posts its agency's requests (`/party/list`, `/party/add`… of rodecomp's
//! `CAdventurerAgencyMgr::ServiceUrl`) to the account's web service, which with rAthena is its `web-server`
//! (`src/web/partybooking_controller.cpp`). A page cannot post to another origin, so the browser client posts
//! them here, and each is handed on as it came: the same form, the same answer. Nothing is answered in the
//! service's place: while it is not running, the answer is 502.

use std::time::Duration;

use axum::Router;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// The requests the client makes (`CAdventurerAgencyMgr::ServiceUrl`). rAthena answers all but `change`.
const REQUESTS: [&str; 7] = ["add", "list", "search", "del", "get", "change", "info"];
/// How long the service is waited for: to be reached, and then to answer.
const REACHED_IN: Duration = Duration::from_secs(3);
const ANSWERED_IN: Duration = Duration::from_secs(10);
/// The longest answer taken.
const MOST: u64 = 1 << 20;

/// `POST /party/<request>`, handed on to the web service at that address (`host:port`).
pub fn router(web_server: String) -> Router {
    Router::new().route("/party/{request}", post(forward)).with_state(std::sync::Arc::new(web_server))
}

async fn forward(State(server): State<std::sync::Arc<String>>, Path(request): Path<String>, headers: HeaderMap, body: Bytes) -> Response {
    if !REQUESTS.contains(&request.as_str()) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let kind = headers.get(header::CONTENT_TYPE).and_then(|kind| kind.to_str().ok()).unwrap_or("application/octet-stream");
    match asked(&server, &request, kind, &body).await {
        Ok((status, answer)) => (
            StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY),
            [(header::CONTENT_TYPE, "application/json"), (header::CACHE_CONTROL, "no-store")],
            answer,
        )
            .into_response(),
        Err(error) => (StatusCode::BAD_GATEWAY, format!("the web service at {server} did not answer: {error}")).into_response(),
    }
}

/// Posts a body to the service and gives its answer's status and body. The service is asked to close the
/// connection after its answer, so the answer is everything it sends.
async fn asked(server: &str, request: &str, kind: &str, body: &[u8]) -> Result<(u16, Vec<u8>), String> {
    let connect = tokio::time::timeout(REACHED_IN, TcpStream::connect(server));
    let mut stream = connect.await.map_err(|_| "not reached in time".to_owned())?.map_err(|error| error.to_string())?;
    let head = format!(
        "POST /party/{request} HTTP/1.1\r\nHost: {server}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let talk = async {
        stream.write_all(head.as_bytes()).await?;
        stream.write_all(body).await?;
        let mut answer = Vec::new();
        (&mut stream).take(MOST).read_to_end(&mut answer).await?;
        Ok::<_, std::io::Error>(answer)
    };
    let answer = tokio::time::timeout(ANSWERED_IN, talk).await.map_err(|_| "no answer in time".to_owned())?.map_err(|error| error.to_string())?;
    answer_of(&answer).ok_or_else(|| "an answer that is no HTTP".to_owned())
}

/// The status and the body of an HTTP answer as it came off the wire; a body sent in chunks is put together.
fn answer_of(answer: &[u8]) -> Option<(u16, Vec<u8>)> {
    let end = answer.windows(4).position(|bytes| bytes == b"\r\n\r\n")?;
    let head = std::str::from_utf8(&answer[..end]).ok()?;
    let status = head.lines().next()?.split(' ').nth(1)?.parse().ok()?;
    let body = &answer[end + 4..];
    let chunked = head.lines().any(|line| {
        let line = line.to_ascii_lowercase();
        line.starts_with("transfer-encoding:") && line.contains("chunked")
    });
    Some((status, if chunked { unchunked(body)? } else { body.to_vec() }))
}

/// A body sent in chunks, each after its length in sixteens, put together.
fn unchunked(mut body: &[u8]) -> Option<Vec<u8>> {
    let mut whole = Vec::new();
    loop {
        let end = body.windows(2).position(|bytes| bytes == b"\r\n")?;
        let size = usize::from_str_radix(std::str::from_utf8(&body[..end]).ok()?.split(';').next()?.trim(), 16).ok()?;
        if size == 0 {
            return Some(whole);
        }
        whole.extend_from_slice(body.get(end + 2..end + 2 + size)?);
        body = body.get(end + 2 + size + 2..)?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_answer_is_its_status_and_its_body() {
        let plain = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 13\r\n\r\n{ \"Type\": 1 }";
        assert_eq!(answer_of(plain), Some((200, b"{ \"Type\": 1 }".to_vec())));
        let refused = b"HTTP/1.1 400 Bad Request\r\n\r\nError";
        assert_eq!(answer_of(refused), Some((400, b"Error".to_vec())));
        let chunked = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\n{ \"T\r\n9\r\nype\": 1 }\r\n0\r\n\r\n";
        assert_eq!(answer_of(chunked), Some((200, b"{ \"Type\": 1 }".to_vec())));
        assert_eq!(answer_of(b"nonsense"), None);
    }

    /// A stand-in for the web service: takes one request, keeps it, answers.
    async fn service(answer: &'static str) -> (String, tokio::task::JoinHandle<String>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let taken = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            let mut piece = [0u8; 1024];
            // (The request ends with its form's last line.)
            while !request.ends_with(b"--end--\r\n") {
                let read = socket.read(&mut piece).await.unwrap();
                request.extend_from_slice(&piece[..read]);
            }
            socket.write_all(answer.as_bytes()).await.unwrap();
            String::from_utf8(request).unwrap()
        });
        (address, taken)
    }

    #[tokio::test]
    async fn a_request_is_handed_on_as_it_came_and_its_answer_given_back() {
        let (address, taken) = service("HTTP/1.1 200 OK\r\nContent-Length: 28\r\n\r\n{ \"Type\": 1, \"totalPage\": 0}").await;
        let form = b"--end\r\nContent-Disposition: form-data; name=\"AID\"\r\n\r\n2000006\r\n--end--\r\n";
        let (status, answer) = asked(&address, "list", "multipart/form-data; boundary=end", form).await.unwrap();
        assert_eq!((status, answer.as_slice()), (200, &b"{ \"Type\": 1, \"totalPage\": 0}"[..]));
        let request = taken.await.unwrap();
        assert!(request.starts_with("POST /party/list HTTP/1.1\r\n"), "{request}");
        assert!(request.contains("Content-Type: multipart/form-data; boundary=end\r\n"), "{request}");
        assert!(request.contains(&format!("Content-Length: {}\r\n", form.len())), "{request}");
        assert!(request.ends_with("name=\"AID\"\r\n\r\n2000006\r\n--end--\r\n"), "{request}");
    }

    #[tokio::test]
    async fn a_service_that_is_not_running_is_an_error_and_no_answer() {
        // A port nothing listens on: one that was just let go.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap().to_string();
        drop(listener);
        assert!(asked(&address, "list", "multipart/form-data; boundary=end", b"").await.is_err());
    }
}
