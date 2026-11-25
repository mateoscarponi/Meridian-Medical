//! Vercel Function entry point (Vercel's native Rust runtime).
//!
//! Every request is rewritten to this function by `vercel.json`; it runs the exact same Axum
//! router as the local server (`src/main.rs`). Static assets (WASM bundle, CSS, fonts) are
//! prebuilt into `public/` and served by Vercel's CDN.
#![recursion_limit = "1024"]

use meridian_medical::server::router::router;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use vercel_runtime::axum::VercelLayer;
use vercel_runtime::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = router("public", SocketAddr::from(([0, 0, 0, 0], 3000)));
    let app = ServiceBuilder::new().layer(VercelLayer::new()).service(app);
    vercel_runtime::run(app).await
}
