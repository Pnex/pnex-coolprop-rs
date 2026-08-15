//! Binary entry point: serves the CoolProp REST API.

use coolprop_server::router;

#[tokio::main]
async fn main() {
    let addr = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("COOLPROP_SERVER_ADDR").ok())
        .unwrap_or_else(|| "0.0.0.0:8080".to_string());
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("could not bind {addr}: {e}"));
    println!(
        "CoolProp server listening on http://{addr}/  \
         (API under /api/v1, docs at /swagger-ui, spec at /openapi.json)"
    );
    axum::serve(listener, router()).await.unwrap();
}
