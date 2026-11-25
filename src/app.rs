//! HTML shell and router. Every page is rendered on the server (Axum + Leptos SSR) and then
//! hydrated in the browser by the same Rust code compiled to WebAssembly.

use crate::components::shell::PlatformShell;
use crate::pages::{home::HomePage, not_found::NotFound, platform, portal::PortalPage, privacy::PrivacyPage};
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Meta, MetaTags, Title};
use leptos_router::{
    components::{ParentRoute, Route, Router, Routes},
    path,
};

pub const DEFAULT_TITLE: &str = "Meridian Orthopedics & Spine Care — Demo";

/// `"%s · Meridian Orthopedics & Spine (Demo)"`
pub fn page_title(t: &str) -> String {
    format!("{t} · Meridian Orthopedics & Spine (Demo)")
}

/// Fonts that are needed above the fold (same set next/font preloaded).
const PRELOAD_FONTS: [&str; 3] = ["/fonts/636a5ac981f94f8b-s.p.woff2", "/fonts/6e8c7cb283336a9d-s.p.woff2", "/fonts/bb3ef058b751a6ad-s.p.woff2"];

pub fn shell(options: LeptosOptions) -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        // Per-request nonce-based Content-Security-Policy for this HTML response.
        if let (Some(nonce), Some(res)) = (use_nonce(), use_context::<leptos_axum::ResponseOptions>()) {
            let csp = crate::server::security::csp(Some(&nonce.to_string()));
            if let Ok(v) = axum::http::HeaderValue::from_str(&csp) {
                res.insert_header(axum::http::header::CONTENT_SECURITY_POLICY, v);
            }
        }
    }
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                {PRELOAD_FONTS.map(|f| view! { <link rel="preload" href=f r#as="font" crossorigin="" r#type="font/woff2"/> })}
                <link rel="stylesheet" href="/main.css"/>
                <link rel="icon" href="/icon.svg" r#type="image/svg+xml" sizes="any"/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body class="min-h-screen">
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Meta
            name="description"
            content="Concept demo of an orthopedic & spine practice website paired with an AI-driven care operations platform: prior authorization, document AI, claims, scheduling, interoperability (HL7 / FHIR) and more."
        />
        <Meta name="robots" content="noindex, nofollow"/>
        <Router>
            <Routes fallback=NotFound>
                <Route path=path!("/") view=|| view! { <Title text=DEFAULT_TITLE/> <HomePage/> }/>
                <Route path=path!("/privacy") view=PrivacyPage/>
                <Route path=path!("/portal") view=|| view! { <Title text=DEFAULT_TITLE/> <PortalPage/> }/>
                <ParentRoute path=path!("/platform") view=|| view! { <Title text=page_title("Care Operations Platform")/> <PlatformShell/> }>
                    <Route path=path!("") view=platform::command_center::CommandCenter/>
                    <Route path=path!("impact") view=platform::impact::ImpactPage/>
                    <Route path=path!("prior-auth") view=platform::prior_auth::PriorAuthPage/>
                    <Route path=path!("eligibility") view=platform::eligibility::EligibilityPage/>
                    <Route path=path!("claims") view=platform::claims::ClaimsPage/>
                    <Route path=path!("wc-nofault") view=platform::wc_nofault::WcNoFaultPage/>
                    <Route path=path!("scheduling") view=platform::scheduling::SchedulingPage/>
                    <Route path=path!("referrals") view=platform::referrals::ReferralsPage/>
                    <Route path=path!("messages") view=platform::messages::MessagesPage/>
                    <Route path=path!("agent") view=platform::agent::AgentPage/>
                    <Route path=path!("scribe") view=platform::scribe::ScribePage/>
                    <Route path=path!("chart-nlp") view=platform::chart_nlp::ChartNlpPage/>
                    <Route path=path!("documents") view=platform::documents::DocumentsPage/>
                    <Route path=path!("interop") view=platform::interop::InteropPage/>
                    <Route path=path!("pipeline") view=platform::pipeline::PipelinePage/>
                    <Route path=path!("automations") view=platform::automations::AutomationsPage/>
                    <Route path=path!("compliance") view=platform::compliance::CompliancePage/>
                    <Route path=path!("architecture") view=platform::architecture::ArchitecturePage/>
                </ParentRoute>
            </Routes>
        </Router>
    }
}
