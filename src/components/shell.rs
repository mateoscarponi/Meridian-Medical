//! Care-operations platform chrome: sidebar navigation, top bar, idle auto-lock and tool headers.

use super::brand::{BrandMark, DemoPill};
use super::browser::use_idle;
use super::icons::Icon;
use super::ui::cx;
use leptos::prelude::*;
use leptos_router::components::Outlet;
use leptos_router::hooks::use_location;

pub struct NavItem {
    pub href: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
}

pub const NAV_GROUPS: [(&str, &[NavItem]); 6] = [
    (
        "Overview",
        &[NavItem { href: "/platform", label: "Command Center", icon: "LayoutDashboard" }, NavItem { href: "/platform/impact", label: "Impact & ROI", icon: "Gauge" }],
    ),
    (
        "Revenue cycle",
        &[
            NavItem { href: "/platform/prior-auth", label: "Prior Auth Copilot", icon: "ClipboardCheck" },
            NavItem { href: "/platform/eligibility", label: "Eligibility", icon: "BadgeCheck" },
            NavItem { href: "/platform/claims", label: "Claim Scrubber", icon: "Receipt" },
            NavItem { href: "/platform/wc-nofault", label: "WC & No-Fault Desk", icon: "Briefcase" },
        ],
    ),
    (
        "Front office",
        &[
            NavItem { href: "/platform/scheduling", label: "Smart Scheduling", icon: "CalendarClock" },
            NavItem { href: "/platform/referrals", label: "Referrals", icon: "Share2" },
            NavItem { href: "/platform/messages", label: "Patient Messages", icon: "MessagesSquare" },
            NavItem { href: "/platform/agent", label: "AI Front-Desk Agent", icon: "Bot" },
        ],
    ),
    (
        "Clinical AI",
        &[
            NavItem { href: "/platform/scribe", label: "Clinical Scribe", icon: "FileText" },
            NavItem { href: "/platform/chart-nlp", label: "Chart NLP", icon: "ScanText" },
            NavItem { href: "/platform/documents", label: "Document AI", icon: "FileScan" },
        ],
    ),
    (
        "Data & integration",
        &[
            NavItem { href: "/platform/interop", label: "HL7 / FHIR Lab", icon: "Network" },
            NavItem { href: "/platform/pipeline", label: "Data Pipeline", icon: "Database" },
            NavItem { href: "/platform/automations", label: "Automations", icon: "Workflow" },
        ],
    ),
    (
        "Trust & build",
        &[
            NavItem { href: "/platform/compliance", label: "HIPAA & Security", icon: "ShieldCheck" },
            NavItem { href: "/platform/architecture", label: "Architecture", icon: "Layers" },
        ],
    ),
];

#[component]
fn NavList(#[prop(optional)] on_navigate: Option<Callback<()>>) -> impl IntoView {
    let location = use_location();
    view! {
        <nav class="space-y-6" aria-label="Platform">
            {NAV_GROUPS
                .iter()
                .map(|(label, items)| {
                    view! {
                        <div>
                            <div class="px-3 text-[10px] font-bold uppercase tracking-[0.16em] text-white/35">{*label}</div>
                            <ul class="mt-2 space-y-0.5">
                                {items
                                    .iter()
                                    .map(|it| {
                                        let href = it.href;
                                        let active = move || {
                                            let path = location.pathname.get();
                                            if href == "/platform" { path == "/platform" } else { path.starts_with(href) }
                                        };
                                        view! {
                                            <li>
                                                <a
                                                    href=href
                                                    on:click=move |_| {
                                                        if let Some(cb) = on_navigate {
                                                            cb.run(());
                                                        }
                                                    }
                                                    class=move || {
                                                        cx(&[
                                                            "flex items-center gap-2.5 rounded-lg px-3 py-2 text-[13.5px] font-medium transition-all",
                                                            if active() {
                                                                "bg-white/10 text-white shadow-[inset_2px_0_0_var(--color-gold-2)]"
                                                            } else {
                                                                "text-white/60 hover:bg-white/5 hover:text-white"
                                                            },
                                                        ])
                                                    }
                                                >
                                                    <Icon name=it.icon class=Signal::derive(move || cx(&["size-4", if active() { "text-gold-2" } else { "" }]))/>
                                                    {it.label}
                                                </a>
                                            </li>
                                        }
                                    })
                                    .collect_view()}
                            </ul>
                        </div>
                    }
                })
                .collect_view()}
        </nav>
    }
}

#[component]
pub fn PlatformShell() -> impl IntoView {
    let open = RwSignal::new(false);
    let manual_lock = RwSignal::new(false);
    let idle = use_idle(15 * 60 * 1000);
    let locked = move || idle.get() || manual_lock.get();
    view! {
        <div class="min-h-screen bg-cream">
            <Show when=locked>
                <div class="fixed inset-0 z-[100] grid place-items-center bg-ink/75 p-4 backdrop-blur-xl" role="dialog" aria-modal="true" aria-label="Session locked">
                    <div class="animate-fade-up w-full max-w-sm rounded-3xl bg-white p-7 text-center shadow-[var(--shadow-lift)]">
                        <div class="mx-auto grid size-12 place-items-center rounded-full bg-navy text-white">
                            <Icon name="Lock" class="size-5"/>
                        </div>
                        <h2 class="mt-4 font-display text-xl font-semibold text-ink">"Session locked"</h2>
                        <p class="mt-1.5 text-[13px] text-muted">
                            {move || if idle.get() { "Locked automatically after 15 minutes of inactivity" } else { "Locked by user" }}
                            " to protect patient information on unattended screens."
                        </p>
                        <button
                            on:click=move |_| {
                                idle.set(false);
                                manual_lock.set(false);
                            }
                            class="mt-5 w-full rounded-xl bg-navy px-4 py-2.5 text-[14px] font-semibold text-white transition hover:bg-ocean"
                        >
                            "Unlock"
                        </button>
                        <p class="mt-2 text-[11px] text-faint">"Demo unlock. Production re-authenticates with SSO + MFA."</p>
                    </div>
                </div>
            </Show>
            <aside class="scroll-thin fixed inset-y-0 left-0 z-40 hidden w-64 flex-col overflow-y-auto bg-ink px-3 py-5 lg:flex">
                <a href="/" class="flex items-center gap-2.5 px-3">
                    <BrandMark uid="side" light=true class="size-9"/>
                    <span class="leading-tight">
                        <span class="block font-display text-[15px] font-semibold text-white">"Meridian Orthopedics"</span>
                        <span class="block text-[10px] font-semibold uppercase tracking-[0.16em] text-white/45">"Care Operations"</span>
                    </span>
                </a>
                <div class="mt-8 flex-1">
                    <NavList/>
                </div>
                <div class="mt-8 rounded-xl border border-white/10 bg-white/[0.04] p-3 text-[11px] leading-relaxed text-white/55">
                    <div class="mb-1 flex items-center gap-2">
                        <DemoPill/>
                        <span class="font-semibold text-white/80">"Synthetic data"</span>
                    </div>
                    "No real patients. Built to show how AI can streamline a multi-site medical practice."
                    <div class="mt-2 flex items-center gap-1.5 text-white/70">
                        <Icon name="ShieldCheck" class="size-3 text-sage-2"/>
                        " HIPAA safeguards demonstrated"
                    </div>
                </div>
            </aside>

            <Show when=move || open.get()>
                <div class="fixed inset-0 z-50 lg:hidden">
                    <div class="absolute inset-0 bg-ink/50 backdrop-blur-sm" on:click=move |_| open.set(false)></div>
                    <div class="scroll-thin absolute inset-y-0 left-0 w-72 overflow-y-auto bg-ink px-3 py-5">
                        <div class="mb-6 flex items-center justify-between px-3">
                            <span class="font-display text-[15px] font-semibold text-white">"Meridian Orthopedics"</span>
                            <button on:click=move |_| open.set(false) class="text-white/70" aria-label="Close navigation">
                                <Icon name="X" class="size-5"/>
                            </button>
                        </div>
                        <NavList on_navigate=Callback::new(move |_| open.set(false))/>
                    </div>
                </div>
            </Show>

            <div class="lg:pl-64">
                <header class="sticky top-0 z-30 border-b border-line bg-cream/85 backdrop-blur-xl">
                    <div class="flex items-center justify-between gap-4 px-4 py-3 sm:px-8">
                        <div class="flex items-center gap-3">
                            <button on:click=move |_| open.set(true) class="rounded-lg p-1.5 text-ink lg:hidden" aria-label="Open navigation">
                                <Icon name="Menu" class="size-5"/>
                            </button>
                            <a href="/" class="hidden items-center gap-1.5 text-[13px] font-medium text-muted hover:text-ink sm:inline-flex">
                                <Icon name="ArrowLeft" class="size-3.5"/>
                                " Practice website"
                            </a>
                        </div>
                        <div class="flex items-center gap-3">
                            <span class="hidden items-center gap-1.5 rounded-full bg-sage-soft px-2.5 py-1 text-[11px] font-semibold text-sage xl:inline-flex">
                                <span class="size-1.5 animate-pulse-soft rounded-full bg-sage"></span>
                                " All integrations healthy"
                            </span>
                            <a
                                href="/platform/compliance"
                                class="hidden items-center gap-1.5 rounded-full bg-sky-2 px-2.5 py-1 text-[11px] font-semibold text-ocean hover:bg-sky md:inline-flex"
                                title="HIPAA safeguards active: audit logging, RBAC, auto-lock after 15 minutes idle"
                            >
                                <Icon name="ShieldCheck" class="size-3"/>
                                " HIPAA safeguards on · auto-lock 15 min"
                            </a>
                            <button
                                on:click=move |_| manual_lock.set(true)
                                class="grid size-8 place-items-center rounded-full text-muted ring-1 ring-line-2 transition hover:bg-sand hover:text-ink"
                                aria-label="Lock screen"
                                title="Lock screen"
                            >
                                <Icon name="Lock" class="size-3.5"/>
                            </button>
                            <DemoPill class="!text-gold"/>
                            <div class="flex items-center gap-2">
                                <div class="grid size-8 place-items-center rounded-full bg-navy text-[12px] font-semibold text-white">"OM"</div>
                                <div class="hidden leading-tight sm:block">
                                    <div class="text-[13px] font-semibold text-ink">"Operations Manager"</div>
                                    <div class="text-[11px] text-muted">"All locations"</div>
                                </div>
                            </div>
                        </div>
                    </div>
                </header>
                <main class="px-4 py-8 sm:px-8">
                    <Outlet/>
                </main>
            </div>
        </div>
    }
}

#[component]
pub fn ToolHeader(
    eyebrow: &'static str,
    #[prop(into)] title: String,
    #[prop(into)] description: String,
    #[prop(optional)] tags: &'static [&'static str],
    #[prop(optional)] impact: &'static [(&'static str, &'static str)],
    /// Tags computed at runtime (rendered before `tags`).
    #[prop(optional)] dyn_tags: Vec<String>,
    /// Impact tiles computed at runtime (rendered before `impact`).
    #[prop(optional)] dyn_impact: Vec<(String, String)>,
) -> impl IntoView {
    let tags: Vec<String> = dyn_tags.into_iter().chain(tags.iter().map(|t| t.to_string())).collect();
    let impact: Vec<(String, String)> = dyn_impact.into_iter().chain(impact.iter().map(|(a, b)| (a.to_string(), b.to_string()))).collect();
    view! {
        <div class="mb-8 flex flex-wrap items-end justify-between gap-6">
            <div class="max-w-3xl">
                <div class="text-[12px] font-semibold uppercase tracking-[0.16em] text-gold">{eyebrow}</div>
                <h1 class="mt-2 font-display text-[34px] leading-tight font-semibold tracking-tight text-ink">{title}</h1>
                <p class="mt-2 text-[15px] leading-relaxed text-muted">{description}</p>
                {(!tags.is_empty())
                    .then(|| {
                        view! {
                            <div class="mt-4 flex flex-wrap gap-1.5">
                                {tags
                                    .into_iter()
                                    .map(|t| {
                                        view! {
                                            <span class="rounded-md bg-white px-2 py-1 font-mono text-[11px] text-muted ring-1 ring-inset ring-line">{t}</span>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        }
                    })}
            </div>
            {(!impact.is_empty())
                .then(|| {
                    view! {
                        <div class="flex gap-3">
                            {impact
                                .into_iter()
                                .map(|(value, label)| {
                                    view! {
                                        <div class="rounded-2xl border border-line bg-white px-4 py-3 shadow-[var(--shadow-soft)]">
                                            <div class="font-display text-2xl font-semibold text-sage tabular-nums">{value}</div>
                                            <div class="text-[11px] font-medium text-muted">{label}</div>
                                        </div>
                                    }
                                })
                                .collect_view()}
                        </div>
                    }
                })}
        </div>
    }
}
