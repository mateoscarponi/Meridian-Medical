use leptos::prelude::*;
use leptos_meta::Title;

/// 404 page (same look as the Next.js default "This page could not be found.").
#[component]
pub fn NotFound() -> impl IntoView {
    #[cfg(feature = "ssr")]
    if let Some(res) = use_context::<leptos_axum::ResponseOptions>() {
        res.set_status(axum::http::StatusCode::NOT_FOUND);
    }
    view! {
        <Title text="404: This page could not be found."/>
        <div style="font-family:system-ui,\"Segoe UI\",Roboto,Helvetica,Arial,sans-serif,\"Apple Color Emoji\",\"Segoe UI Emoji\";height:100vh;text-align:center;display:flex;flex-direction:column;align-items:center;justify-content:center">
            <div style="line-height:48px">
                <h1 style="display:inline-block;margin:0 20px 0 0;padding-right:23px;font-size:24px;font-weight:500;vertical-align:top;border-right:1px solid rgba(0,0,0,.3)">"404"</h1>
                <div style="display:inline-block">
                    <h2 style="font-size:14px;font-weight:400;line-height:28px;margin:0">"This page could not be found."</h2>
                </div>
            </div>
        </div>
    }
}
