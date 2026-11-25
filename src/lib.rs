//! Meridian Orthopedics & Spine Care — the whole site in Rust.
//!
//! One crate, two compile targets:
//! - `ssr` (native): Axum server, API routes, server-side rendering of every page.
//! - `hydrate` (wasm32): the same components compiled to WebAssembly to make pages interactive.
#![recursion_limit = "1024"]
#![allow(unexpected_cfgs)]

// The server's HTML and the WASM bundle must agree on Leptos type erasure (it changes the
// hydration markers). `.cargo/config.toml` sets it for every target; fail loudly if a build
// environment (e.g. a RUSTFLAGS override) drops it, instead of shipping pages that can't hydrate.
#[cfg(not(erase_components))]
compile_error!("build with `--cfg erase_components` (see .cargo/config.toml)");

pub mod app;
pub mod components;
pub mod domain;
pub mod pages;
#[cfg(feature = "ssr")]
pub mod server;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
}
