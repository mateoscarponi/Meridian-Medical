//! The complete Axum application (pages + APIs + security headers), shared by the local
//! server (`src/main.rs`) and the Vercel function (`api/vercel.rs`).

use super::{api, security};
use crate::app::{shell, App};
use axum::{middleware, Router};
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use std::net::SocketAddr;
use tower_http::compression::CompressionLayer;

/// Builds the router. `site_root` is the folder holding the built static site
/// (`pkg/` WASM bundle, `main.css`, fonts, icon), served by the fallback handler.
pub fn router(site_root: &str, addr: SocketAddr) -> Router {
    let leptos_options = LeptosOptions::builder().output_name("meridian_medical").site_root(site_root).site_pkg_dir("pkg").site_addr(addr).build();
    let routes = generate_route_list(App);
    Router::new()
        .merge(api::routes())
        .leptos_routes(&leptos_options, routes, {
            let o = leptos_options.clone();
            move || shell(o.clone())
        })
        // Static files from the site root (WASM bundle, CSS, fonts, icon); 404 page otherwise.
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options)
        .layer(middleware::from_fn(security::headers))
        .layer(CompressionLayer::new())
}
