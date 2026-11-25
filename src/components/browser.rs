//! Browser-only helpers (scroll, IntersectionObserver, media queries, idle timer, storage).
//! Everything here runs inside effects, which only execute in the browser after hydration,
//! so the server render never touches `window`.

use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub fn window() -> web_sys::Window {
    web_sys::window().expect("window")
}

pub fn prefers_reduced_motion() -> bool {
    window().match_media("(prefers-reduced-motion: reduce)").ok().flatten().map(|m| m.matches()).unwrap_or(false)
}

/// Adds a passive window listener; returns a guard closure that removes it.
pub fn on_window(event: &'static str, f: impl Fn() + 'static) -> impl FnOnce() {
    let cb = Closure::<dyn Fn()>::new(f);
    let opts = web_sys::AddEventListenerOptions::new();
    opts.set_passive(true);
    let _ = window().add_event_listener_with_callback_and_add_event_listener_options(event, cb.as_ref().unchecked_ref(), &opts);
    move || {
        let _ = window().remove_event_listener_with_callback(event, cb.as_ref().unchecked_ref());
        drop(cb);
    }
}

/// Observes `el`; calls `f(is_intersecting)` on every change. Returns the observer (disconnect to stop).
pub fn observe(el: &web_sys::Element, threshold: f64, root_margin: &str, f: impl Fn(bool, &web_sys::IntersectionObserver) + 'static) -> web_sys::IntersectionObserver {
    let cb = Closure::<dyn Fn(js_sys::Array, web_sys::IntersectionObserver)>::new(move |entries: js_sys::Array, io: web_sys::IntersectionObserver| {
        if let Some(e) = entries.get(0).dyn_ref::<web_sys::IntersectionObserverEntry>() {
            f(e.is_intersecting(), &io);
        }
    });
    let init = web_sys::IntersectionObserverInit::new();
    init.set_threshold(&JsValue::from_f64(threshold));
    init.set_root_margin(root_margin);
    let io = web_sys::IntersectionObserver::new_with_options(cb.as_ref().unchecked_ref(), &init).expect("IntersectionObserver");
    cb.forget(); // lives as long as the observer
    io.observe(el);
    io
}

pub fn session_get(key: &str) -> Option<String> {
    window().session_storage().ok().flatten()?.get_item(key).ok().flatten()
}
pub fn session_set(key: &str, value: &str) {
    if let Some(s) = window().session_storage().ok().flatten() {
        let _ = s.set_item(key, value);
    }
}
pub fn session_remove(key: &str) {
    if let Some(s) = window().session_storage().ok().flatten() {
        let _ = s.remove_item(key);
    }
}

/// Automatic logoff (HIPAA Security Rule §164.312(a)(2)(iii)).
/// `idle` becomes true once the user has been inactive for `ms`; set it back to false to reset.
pub fn use_idle(ms: i32) -> RwSignal<bool> {
    let idle = RwSignal::new(false);
    Effect::new(move |_| {
        let timer: std::rc::Rc<std::cell::Cell<i32>> = Default::default();
        let fire = Closure::<dyn Fn()>::new(move || idle.set(true));
        let fire_fn: js_sys::Function = fire.as_ref().unchecked_ref::<js_sys::Function>().clone();
        fire.forget();
        let arm = {
            let timer = timer.clone();
            move || {
                let w = window();
                w.clear_timeout_with_handle(timer.get());
                timer.set(w.set_timeout_with_callback_and_timeout_and_arguments_0(&fire_fn, ms).unwrap_or(0));
            }
        };
        arm();
        let guards: Vec<Box<dyn FnOnce()>> = ["mousemove", "keydown", "pointerdown", "scroll", "touchstart"]
            .into_iter()
            .map(|e| {
                let arm = arm.clone();
                Box::new(on_window(e, move || arm())) as Box<dyn FnOnce()>
            })
            .collect();
        let timer2 = timer.clone();
        cleanup(move || {
            for g in guards {
                g();
            }
            window().clear_timeout_with_handle(timer2.get());
        });
    });
    idle
}

/// `on_cleanup` for browser-only (non-`Send`) resources such as JS closures and timers.
pub fn cleanup(f: impl FnOnce() + 'static) {
    let f = send_wrapper::SendWrapper::new(Some(Box::new(f) as Box<dyn FnOnce()>));
    let f = std::sync::Mutex::new(f);
    on_cleanup(move || {
        if let Ok(mut g) = f.lock() {
            if let Some(f) = Option::take(&mut **g) {
                f();
            }
        }
    });
}

/// `navigator.clipboard.writeText(text)` (best effort).
pub fn copy_text(text: &str) {
    let nav = window().navigator();
    if let Ok(clip) = js_sys::Reflect::get(&nav, &"clipboard".into()) {
        if let Ok(f) = js_sys::Reflect::get(&clip, &"writeText".into()) {
            if let Some(f) = f.dyn_ref::<js_sys::Function>() {
                let _ = f.call1(&clip, &JsValue::from_str(text));
            }
        }
    }
}

/// `fetch(url, { method, headers: { "Content-Type": content_type }, body })` -> (status, body text).
/// Same-origin only (the CSP `connect-src 'self'` enforces it).
#[cfg(feature = "hydrate")]
pub async fn fetch_text(method: &str, url: &str, content_type: Option<&str>, body: Option<String>) -> Result<(u16, String), String> {
    use wasm_bindgen_futures::JsFuture;
    let init = web_sys::RequestInit::new();
    init.set_method(method);
    if let Some(b) = body {
        init.set_body(&JsValue::from_str(&b));
    }
    let req = web_sys::Request::new_with_str_and_init(url, &init).map_err(|e| format!("{e:?}"))?;
    if let Some(ct) = content_type {
        let _ = req.headers().set("Content-Type", ct);
    }
    let res: web_sys::Response = JsFuture::from(window().fetch_with_request(&req)).await.map_err(|e| format!("{e:?}"))?.dyn_into().map_err(|_| "bad response".to_string())?;
    let status = res.status();
    let text = JsFuture::from(res.text().map_err(|e| format!("{e:?}"))?).await.map_err(|e| format!("{e:?}"))?;
    Ok((status, text.as_string().unwrap_or_default()))
}

/// Server build: never called (event handlers only run in the browser).
#[cfg(not(feature = "hydrate"))]
pub async fn fetch_text(_method: &str, _url: &str, _content_type: Option<&str>, _body: Option<String>) -> Result<(u16, String), String> {
    Err("browser only".into())
}

/// `await new Promise((r) => setTimeout(r, ms))`
#[cfg(feature = "hydrate")]
pub async fn sleep_ms(ms: i32) {
    let p = js_sys::Promise::new(&mut |resolve, _| {
        let _ = window().set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
    });
    let _ = wasm_bindgen_futures::JsFuture::from(p).await;
}
#[cfg(not(feature = "hydrate"))]
pub async fn sleep_ms(_ms: i32) {}
