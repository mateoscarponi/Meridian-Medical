//! Security headers applied to every route (HIPAA Security Rule: transmission security,
//! integrity, and defense against clickjacking / content injection).
//! No third-party origins are allowed: fonts are self-hosted, and the site loads no analytics,
//! ad pixels or chat widgets.
//!
//! Upgrade over the original: HTML pages get a per-request nonce-based CSP for scripts
//! (`'nonce-…'` instead of `'unsafe-inline'`), set from the Leptos shell.

use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};

pub fn production() -> bool {
    std::env::var("MERIDIAN_ENV").map(|v| v == "production").unwrap_or(false)
}

/// CSP. `nonce`: Some for server-rendered pages, None for static files / API responses.
pub fn csp(nonce: Option<&str>) -> String {
    let script = match nonce {
        // 'wasm-unsafe-eval' lets the browser compile our own WebAssembly module (nothing else).
        Some(n) => format!("script-src 'self' 'nonce-{n}' 'wasm-unsafe-eval'"),
        None => "script-src 'self' 'wasm-unsafe-eval'".to_string(),
    };
    let mut parts = vec![
        "default-src 'self'".to_string(),
        script,
        "style-src 'self' 'unsafe-inline'".into(),
        "img-src 'self' data: blob:".into(),
        "font-src 'self'".into(),
        "connect-src 'self'".into(),
        "frame-ancestors 'none'".into(),
        "form-action 'self'".into(),
        "base-uri 'self'".into(),
        "object-src 'none'".into(),
    ];
    if production() {
        parts.push("upgrade-insecure-requests".into());
    }
    parts.join("; ")
}

const STATIC_HEADERS: [(&str, &str); 6] = [
    ("strict-transport-security", "max-age=63072000; includeSubDomains; preload"),
    ("x-frame-options", "DENY"),
    ("x-content-type-options", "nosniff"),
    ("referrer-policy", "no-referrer"),
    ("permissions-policy", "camera=(), microphone=(), geolocation=(), payment=(), interest-cohort=(), browsing-topics=()"),
    ("cross-origin-opener-policy", "same-origin"),
];

/// Adds the security headers, plus `Cache-Control` rules:
/// PHI-bearing surfaces (/platform, /portal, /api) are never cached; fonts are immutable.
pub async fn headers(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_string();
    let mut res = next.run(req).await;
    let h = res.headers_mut();
    for (k, v) in STATIC_HEADERS {
        h.insert(k, HeaderValue::from_static(v));
    }
    if !h.contains_key("content-security-policy") {
        if let Ok(v) = HeaderValue::from_str(&csp(None)) {
            h.insert("content-security-policy", v);
        }
    }
    let no_store = ["/platform", "/portal", "/api"].iter().any(|p| path == *p || path.starts_with(&format!("{p}/")));
    if no_store {
        h.insert("cache-control", HeaderValue::from_static("no-store"));
    } else if path.starts_with("/fonts/") {
        h.insert("cache-control", HeaderValue::from_static("public, max-age=31536000, immutable"));
    } else if !h.contains_key("cache-control") {
        h.insert("cache-control", HeaderValue::from_static("no-cache"));
    }
    res
}
