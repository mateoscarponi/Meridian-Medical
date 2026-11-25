//! Meridian Orthopedics & Spine Care — the whole site in Rust.
//!
//! One crate, two compile targets:
//! - `ssr` (native): Axum server, API routes, server-side rendering of every page.
//! - `hydrate` (wasm32): the same components compiled to WebAssembly to make pages interactive.
#![recursion_limit = "1024"]

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
