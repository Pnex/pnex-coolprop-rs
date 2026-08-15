//! Shared helpers for the integration tests.

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use coolprop_server::router;

/// Issue a request with a JSON body; returns `(status, parsed body)`.
pub async fn json_req(
    method: Method,
    uri: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let response = router()
        .oneshot(
            Request::builder()
                .method(method.clone())
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let parsed = if bytes.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or_else(|e| {
            panic!(
                "non-JSON response for {method} {uri}: {e}: {}",
                String::from_utf8_lossy(&bytes)
            )
        })
    };
    (status, parsed)
}

/// Issue a body-less request; returns `(status, parsed JSON body)`.
pub async fn req(method: Method, uri: &str) -> (StatusCode, serde_json::Value) {
    let response = router()
        .oneshot(
            Request::builder()
                .method(method.clone())
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let parsed = if bytes.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or_else(|e| {
            panic!(
                "non-JSON response for {method} {uri}: {e}: {}",
                String::from_utf8_lossy(&bytes)
            )
        })
    };
    (status, parsed)
}

/// Issue a body-less request; returns `(status, raw text body)`.
/// (Only used by some of the test binaries sharing this module.)
#[allow(dead_code)]
pub async fn text_req(method: Method, uri: &str) -> (StatusCode, String) {
    let response = router()
        .oneshot(
            Request::builder()
                .method(method.clone())
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8_lossy(&bytes).into_owned())
}
