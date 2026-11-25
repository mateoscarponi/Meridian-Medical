#![recursion_limit = "1024"]
//! Meridian Orthopedics & Spine Care — Rust server.
//!
//! Axum serves: server-rendered Leptos pages, the WebAssembly bundle and static assets,
//! and the JSON / FHIR / HL7 APIs. Security headers and cache rules wrap every response.
//!
//! Environment:
//!   MERIDIAN_ADDR       listen address (default 127.0.0.1:3018)
//!   MERIDIAN_SITE_ROOT  directory with the built site (default target/site)
//!   MERIDIAN_ENV        "production" adds upgrade-insecure-requests to the CSP
//!   ANTHROPIC_API_KEY   optional; enables Claude for the Scribe / Messages tools

use meridian_medical::server::router::router;
use std::net::SocketAddr;

/// `target/site` next to the working directory, or the one beside the executable
/// (`target/release/meridian_medical.exe` -> `target/site`), so the server runs from anywhere.
fn default_site_root() -> String {
    if std::path::Path::new("target/site/pkg").exists() {
        return "target/site".into();
    }
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().and_then(|d| d.parent()).map(|t| t.join("site")))
        .filter(|p| p.exists())
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "target/site".into())
}

#[tokio::main]
async fn main() {
    let addr: SocketAddr = std::env::var("MERIDIAN_ADDR").ok().and_then(|a| a.parse().ok()).unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 3018)));
    let site_root = std::env::var("MERIDIAN_SITE_ROOT").unwrap_or_else(|_| default_site_root());
    let app = router(&site_root, addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap_or_else(|e| panic!("cannot bind {addr}: {e}"));
    println!("Meridian Orthopedics & Spine Care (Rust) listening on http://{addr}");

    // "localhost" resolves to ::1 first on Windows; when using the default loopback address, also
    // listen on IPv6 loopback so clients don't wait ~2 s for the IPv4 fallback.
    let v6 = if std::env::var("MERIDIAN_ADDR").is_err() {
        tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], addr.port()))).await.ok()
    } else {
        None
    };
    let main = axum::serve(listener, app.clone().into_make_service());
    match v6 {
        Some(l6) => {
            println!("also listening on http://[::1]:{}  (open http://localhost:{})", addr.port(), addr.port());
            let (a, b) = tokio::join!(main, axum::serve(l6, app.into_make_service()));
            a.and(b).expect("server error");
        }
        None => main.await.expect("server error"),
    }
}
