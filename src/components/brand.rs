//! Brand mark: a side-view human spine (cervical, thoracic and lumbar curves, gold discs, sacrum)
//! on a deep navy tile. Pure SVG so it stays sharp at every size; same artwork as public/icon.svg.

use super::ui::cx;
use leptos::prelude::*;

const SPINE_PROCESSES: &str = r#"<path transform="translate(26.7 8.06) rotate(-0.78)" d="M1.64 -0.21L4.69 0.57" stroke-width="1.11"/><path transform="translate(27.28 11.19) rotate(-9.06)" d="M1.75 -0.23L5 0.61" stroke-width="1.17"/><path transform="translate(28.69 14.44) rotate(-13.54)" d="M1.86 -0.24L5.3 0.65" stroke-width="1.24"/><path transform="translate(30.45 17.83) rotate(-13.02)" d="M1.96 -0.25L5.87 1.85" stroke-width="1.31"/><path transform="translate(31.79 21.35) rotate(-6.89)" d="M2.07 -0.27L6.16 1.92" stroke-width="1.38"/><path transform="translate(32.02 25.01) rotate(3.55)" d="M2.18 -0.28L6.45 1.99" stroke-width="1.45"/><path transform="translate(30.9 28.79) rotate(11.89)" d="M2.29 -0.29L6.73 2.06" stroke-width="1.52"/><path transform="translate(28.92 32.71) rotate(13.77)" d="M2.4 -0.31L7.02 2.13" stroke-width="1.59"/><path transform="translate(27.16 36.77) rotate(8.28)" d="M2.51 -0.32L7.02 1.13" stroke-width="1.66"/><path transform="translate(26.77 40.95) rotate(-3.39)" d="M2.62 -0.33L7.32 1.18" stroke-width="1.73"/><path transform="translate(28.11 45.27) rotate(-12.5)" d="M2.73 -0.35L7.62 1.23" stroke-width="1.8"/>"#;
const SPINE_BONES: &str = r#"<rect transform="translate(26.7 8.06) rotate(-0.78)" x="-2.92" y="-1.06" width="5.85" height="2.13" rx="0.85"/><rect transform="translate(27.28 11.19) rotate(-9.06)" x="-3.12" y="-1.13" width="6.24" height="2.26" rx="0.9"/><rect transform="translate(28.69 14.44) rotate(-13.54)" x="-3.31" y="-1.2" width="6.63" height="2.39" rx="0.96"/><rect transform="translate(30.45 17.83) rotate(-13.02)" x="-3.51" y="-1.26" width="7.02" height="2.52" rx="1.01"/><rect transform="translate(31.79 21.35) rotate(-6.89)" x="-3.7" y="-1.33" width="7.41" height="2.66" rx="1.06"/><rect transform="translate(32.02 25.01) rotate(3.55)" x="-3.9" y="-1.4" width="7.8" height="2.79" rx="1.12"/><rect transform="translate(30.9 28.79) rotate(11.89)" x="-4.09" y="-1.46" width="8.19" height="2.92" rx="1.17"/><rect transform="translate(28.92 32.71) rotate(13.77)" x="-4.29" y="-1.53" width="8.58" height="3.06" rx="1.22"/><rect transform="translate(27.16 36.77) rotate(8.28)" x="-4.48" y="-1.59" width="8.97" height="3.19" rx="1.28"/><rect transform="translate(26.77 40.95) rotate(-3.39)" x="-4.68" y="-1.66" width="9.36" height="3.32" rx="1.33"/><rect transform="translate(28.11 45.27) rotate(-12.5)" x="-4.87" y="-1.73" width="9.75" height="3.46" rx="1.38"/><path transform="translate(29.38 48.1) rotate(-24)" d="M-4.87 0 H4.87 C4.97 3.4 1.75 7 0.19 8.8 C-0.78 9.9 -2.53 10.1 -3.22 9.2 C-4.39 7.8 -5.02 3.6 -4.87 0 Z"/>"#;
const SPINE_DISCS: &str = r#"<rect transform="translate(26.87 9.59) rotate(-5.18)" x="-1.87" y="-0.45" width="3.74" height="0.9" rx="0.45"/><rect transform="translate(27.89 12.78) rotate(-11.87)" x="-2" y="-0.45" width="3.99" height="0.9" rx="0.45"/><rect transform="translate(29.56 16.1) rotate(-13.94)" x="-2.12" y="-0.45" width="4.24" height="0.9" rx="0.45"/><rect transform="translate(31.22 19.56) rotate(-10.71)" x="-2.25" y="-0.45" width="4.49" height="0.9" rx="0.45"/><rect transform="translate(32.08 23.15) rotate(-1.94)" x="-2.37" y="-0.45" width="4.74" height="0.9" rx="0.45"/><rect transform="translate(31.63 26.87) rotate(8.36)" x="-2.49" y="-0.45" width="4.99" height="0.9" rx="0.45"/><rect transform="translate(29.97 30.72) rotate(13.7)" x="-2.62" y="-0.45" width="5.24" height="0.9" rx="0.45"/><rect transform="translate(27.94 34.71) rotate(12.02)" x="-2.74" y="-0.45" width="5.49" height="0.9" rx="0.45"/><rect transform="translate(26.75 38.83) rotate(2.87)" x="-2.87" y="-0.45" width="5.74" height="0.9" rx="0.45"/><rect transform="translate(27.24 43.08) rotate(-8.83)" x="-2.99" y="-0.45" width="5.99" height="0.9" rx="0.45"/>"#;

fn markup(id: &str, light: bool) -> String {
    let mut s = String::with_capacity(4096);
    s.push_str(&format!(
        r##"<defs><linearGradient id="{id}bg" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#1e4d7a"/><stop offset="1" stop-color="#0f2a47"/></linearGradient><linearGradient id="{id}bn" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#ffffff"/><stop offset="1" stop-color="#ecd9b6"/></linearGradient><radialGradient id="{id}gl" cx=".5" cy=".42" r=".55"><stop offset="0" stop-color="#5f93c2" stop-opacity=".45"/><stop offset="1" stop-color="#5f93c2" stop-opacity="0"/></radialGradient></defs><rect width="64" height="64" rx="15" fill="url(#{id}bg)"/><rect width="64" height="64" rx="15" fill="url(#{id}gl)"/>"##
    ));
    if light {
        s.push_str(r##"<rect x=".75" y=".75" width="62.5" height="62.5" rx="14.25" fill="none" stroke="rgb(255 255 255 / .22)" stroke-width="1.5"/>"##);
    }
    s.push_str(&format!(r##"<g stroke="url(#{id}bn)" stroke-linecap="round" fill="none">{SPINE_PROCESSES}</g><g fill="url(#{id}bn)">{SPINE_BONES}</g><g fill="#d4a94f">{SPINE_DISCS}</g>"##));
    s
}

/// `uid` keeps the gradient ids unique when several marks are on one page (React used `useId`).
#[component]
pub fn BrandMark(uid: &'static str, #[prop(optional, into)] class: String, #[prop(optional)] light: bool) -> impl IntoView {
    // Static, build-time artwork (no user input), so injecting it as markup is safe.
    view! { <svg viewBox="0 0 64 64" class=cx(&["shrink-0", &class]) aria-hidden="true" focusable="false" inner_html=markup(uid, light)></svg> }
}

#[component]
pub fn Logo(uid: &'static str, #[prop(optional)] light: bool, #[prop(optional)] compact: bool) -> impl IntoView {
    view! {
        <a href="/" class="group flex items-center gap-3" aria-label="Meridian Orthopedics & Spine Care, home">
            <BrandMark uid=uid light=light class="size-10 transition-transform duration-300 group-hover:-rotate-3"/>
            {(!compact)
                .then(|| {
                    view! {
                        <span class="leading-tight">
                            <span class=cx(&["block font-display text-[17px] font-semibold", if light { "text-white" } else { "text-ink" }])>
                                "Meridian Orthopedics"
                            </span>
                            <span class=cx(&[
                                "block text-[11px] font-semibold uppercase tracking-[0.18em]",
                                if light { "text-white/60" } else { "text-muted" },
                            ])>"& Spine Care"</span>
                        </span>
                    }
                })}
        </a>
    }
}

#[component]
pub fn DemoPill(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <span
            class=cx(&[
                "inline-flex items-center gap-1 rounded-full bg-gold/15 px-2 py-0.5 text-[10px] font-bold uppercase tracking-[0.14em] text-gold-2 ring-1 ring-inset ring-gold/30",
                &class,
            ])
            title="Concept demo — not a real medical practice. All data is synthetic."
        >
            "Demo"
        </span>
    }
}
