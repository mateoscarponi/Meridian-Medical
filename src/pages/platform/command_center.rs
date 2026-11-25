use crate::components::charts::{PairedBars, TrendLine};
use crate::components::icons::Icon;
use crate::components::shell::ToolHeader;
use crate::components::ui::*;
use leptos::prelude::*;

struct Score {
    goal: &'static str,
    kpi: &'static str,
    base: &'static str,
    now: &'static str,
    target: &'static str,
    pct: f64,
    href: &'static str,
    icon: &'static str,
}

const SCORECARD: [Score; 5] = [
    Score { goal: "Reduced administrative workload", kpi: "Staff hours on manual admin / week", base: "318 h", now: "188 h", target: "170 h", pct: 88.0, href: "/platform/impact", icon: "Timer" },
    Score { goal: "Faster authorizations & documentation", kpi: "Median auth turnaround · note close time", base: "10 d · 2.1 d", now: "2.6 d · same day", target: "2 d · same day", pct: 92.0, href: "/platform/prior-auth", icon: "ClipboardCheck" },
    Score { goal: "Improved interoperability", kpi: "Systems connected via HL7 / FHIR / X12", base: "2", now: "9", target: "10", pct: 90.0, href: "/platform/interop", icon: "Network" },
    Score { goal: "Operational efficiency through automation", kpi: "Share of repeatable tasks automated", base: "8%", now: "71%", target: "75%", pct: 95.0, href: "/platform/automations", icon: "Workflow" },
    Score { goal: "AI tools that measurably improve workflows", kpi: "AI tools in production with a tracked KPI", base: "0", now: "7 of 7", target: "7", pct: 100.0, href: "/platform/impact", icon: "Brain" },
];

struct Action {
    id: u32,
    icon: &'static str,
    title: &'static str,
    detail: &'static str,
    cta: &'static str,
    href: &'static str,
    danger: bool,
}

const ACTIONS: [Action; 4] = [
    Action { id: 1, icon: "ClipboardCheck", title: "3 Aetna authorizations pending > 5 business days", detail: "Copilot drafted peer-to-peer requests with the chart evidence attached.", cta: "Send P2P requests", href: "/platform/prior-auth", danger: false },
    Action { id: 2, icon: "CalendarClock", title: "Garden City, Thu 2–4 PM: 4 open slots, 6 waitlist matches", detail: "Matched by visit type, insurance and distance. Expected fill rate 83%.", cta: "Text offers", href: "/platform/scheduling", danger: false },
    Action { id: 3, icon: "Receipt", title: "2 claims at high denial risk ($2,480)", detail: "Bundled 77003 with 64483 and an invalid M54.5 code. Fixes are pre-filled.", cta: "Apply fixes", href: "/platform/claims", danger: true },
    Action { id: 4, icon: "Share2", title: "Referral loop open 6 days: Dr. Patel's office", detail: "Consult note signed 11/12. Auto-send via Direct secure messaging.", cta: "Close the loop", href: "/platform/referrals", danger: false },
];

const QUEUES: [(&str, u32, u32, &str); 4] = [("Front desk", 14, 96, "18 min"), ("Authorizations", 22, 91, "2.1 d"), ("Billing", 37, 88, "3 d"), ("Clinical inbox", 9, 99, "41 min")];

const INTEGRATIONS: [(&str, bool, &str); 6] = [
    ("EMR · FHIR R4", true, "118 ms"),
    ("HL7 ADT / SIU feed", true, "1.2 s"),
    ("Clearinghouse X12", true, "4.0 s"),
    ("Fax → Document AI", true, "38 s"),
    ("SMS / IVR gateway", true, "0.6 s"),
    ("NY WCB Medical Portal", false, "retrying"),
];

const TOUR: [(&str, &str, &str); 7] = [
    ("/#request", "Smart intake", "patient-side triage & routing"),
    ("/platform/prior-auth", "Prior Auth Copilot", "run it on PA-24812"),
    ("/platform/documents", "Document AI", "press Process all"),
    ("/platform/claims", "Claim Scrubber", "fix all safe edits"),
    ("/platform/agent", "AI Agent", "play the Spanish scenario"),
    ("/platform/interop", "HL7 / FHIR Lab", "POST a real message"),
    ("/platform/impact", "Impact & ROI", "plug in real volumes"),
];

#[component]
pub fn CommandCenter() -> impl IntoView {
    let done = RwSignal::new(Vec::<u32>::new());
    let tour = RwSignal::new(true);
    view! {
        <div class="mx-auto max-w-7xl">
            <ToolHeader
                eyebrow="Monday, November 16 · All locations"
                title="Good morning. Here's where the practice stands."
                description="One view of the five outcomes this platform is built to move, the work waiting on each team, and AI-recommended next actions."
                tags=&["decision-support", "real-time KPIs", "work queues", "SLA tracking"]
            />

            <Show when=move || tour.get()>
                <Card class="mb-6 overflow-hidden border-ocean/20 bg-gradient-to-r from-sky-2 via-white to-sage-soft/50">
                    <div class="flex flex-wrap items-center justify-between gap-3 px-5 pt-4">
                        <div class="flex items-center gap-2 text-[13px] font-semibold text-ink">
                            <Icon name="Compass" class="size-4 text-ocean"/>
                            " 5-minute guided tour"
                        </div>
                        <button on:click=move |_| tour.set(false) class="text-[12px] font-semibold text-muted hover:text-ink">"Hide"</button>
                    </div>
                    <ol class="scroll-thin flex gap-2 overflow-x-auto px-5 pb-4 pt-3">
                        {TOUR
                            .iter()
                            .enumerate()
                            .map(|(i, (href, label, hint))| {
                                view! {
                                    <li class="shrink-0">
                                        <a href=*href class="group flex items-center gap-2.5 rounded-xl bg-white px-3 py-2 ring-1 ring-line transition hover:ring-ocean">
                                            <span class="grid size-6 place-items-center rounded-full bg-navy text-[11px] font-bold text-white">{i + 1}</span>
                                            <span>
                                                <span class="block text-[12.5px] font-semibold text-ink group-hover:text-ocean">{*label}</span>
                                                <span class="block text-[11px] text-muted">{*hint}</span>
                                            </span>
                                        </a>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ol>
                </Card>
            </Show>

            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                <Stat label="Admin hours saved this week" value="130 h" delta=Some("≈ 3.3 FTE".to_string()) hint="vs. pre-automation baseline"/>
                <Stat label="Median auth turnaround" value="2.6 d" delta=Some("−74%".to_string()) hint="baseline: up to 10 business days"/>
                <Stat label="Clean claim rate" value="97.1%" delta=Some("+9.4 pts".to_string()) hint="first-pass acceptance"/>
                <Stat label="No-show rate" value="8.6%" delta=Some("−4.8 pts".to_string()) hint="model-driven reminders & backfill"/>
            </div>

            <div class="mt-6 grid gap-6 xl:grid-cols-[1.35fr_1fr]">
                <Card>
                    <CardHeader
                        title="What success looks like"
                        subtitle="Baseline at go-live → today → 90-day target"
                        icon=|| view! { <Icon name="Check" class="size-4"/> }
                    />
                    <ul class="divide-y divide-line">
                        {SCORECARD
                            .iter()
                            .map(|s| {
                                view! {
                                    <li>
                                        <a href=s.href class="group grid gap-3 px-5 py-4 transition hover:bg-cream sm:grid-cols-[1.4fr_1fr] sm:items-center">
                                            <div class="flex items-start gap-3">
                                                <Icon name=s.icon class="mt-0.5 size-4 shrink-0 text-ocean"/>
                                                <div>
                                                    <div class="flex items-center gap-1 text-[14px] font-semibold text-ink">
                                                        {s.goal}
                                                        <Icon name="ArrowUpRight" class="size-3.5 text-faint opacity-0 transition group-hover:opacity-100"/>
                                                    </div>
                                                    <div class="text-[12px] text-muted">{s.kpi}</div>
                                                </div>
                                            </div>
                                            <div>
                                                <div class="mb-1.5 flex items-center justify-between text-[12px] tabular-nums">
                                                    <span class="text-faint line-through decoration-faint/50">{s.base}</span>
                                                    <span class="font-semibold text-ink">{s.now}</span>
                                                    <span class="text-muted">"→ " {s.target}</span>
                                                </div>
                                                <Progress value=s.pct tone=ProgressTone::Sage/>
                                            </div>
                                        </a>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                </Card>

                <Card>
                    <CardHeader
                        title="Recommended actions"
                        subtitle="Generated from live queues and payer data"
                        icon=|| view! { <Icon name="Lightbulb" class="size-4"/> }
                    />
                    <ul class="divide-y divide-line">
                        {ACTIONS
                            .iter()
                            .map(|a| {
                                let id = a.id;
                                let is_done = move || done.with(|d| d.contains(&id));
                                view! {
                                    <li class=move || cx(&["px-5 py-4 transition", if is_done() { "bg-sage-soft/40" } else { "" }])>
                                        <div class="flex items-start gap-3">
                                            <div class=move || {
                                                cx(&["mt-0.5 grid size-8 shrink-0 place-items-center rounded-lg", if is_done() { "bg-sage text-white" } else { "bg-sand text-ink" }])
                                            }>
                                                {move || if is_done() { view! { <Icon name="Check" class="size-4"/> } } else { view! { <Icon name=a.icon class="size-4"/> } }}
                                            </div>
                                            <div class="min-w-0 flex-1">
                                                <div class=move || cx(&["text-[14px] font-semibold", if is_done() { "text-sage" } else { "text-ink" }])>{a.title}</div>
                                                <div class="mt-0.5 text-[12px] text-muted">
                                                    {move || if is_done() { "Done: logged to the audit trail and the owning queue." } else { a.detail }}
                                                </div>
                                                <Show when=move || !is_done()>
                                                    <div class="mt-2.5 flex gap-2">
                                                        <Button
                                                            size=BtnSize::Sm
                                                            variant={if a.danger { BtnVariant::Dark } else { BtnVariant::Primary }}
                                                            on_click=Callback::new(move |_| done.update(|d| d.push(id)))
                                                        >
                                                            {a.cta}
                                                        </Button>
                                                        <a href=a.href class="inline-flex items-center rounded-xl px-3 py-1.5 text-[13px] font-semibold text-ocean hover:bg-sky-2">
                                                            "Review"
                                                        </a>
                                                    </div>
                                                </Show>
                                            </div>
                                        </div>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                </Card>
            </div>

            <div class="mt-6 grid gap-6 lg:grid-cols-2">
                <Card>
                    <CardHeader title="Staff hours per week, by workflow" subtitle="Before the platform vs. the last 4 weeks (average)"/>
                    <div class="p-5">
                        <PairedBars
                            a_label="Before"
                            b_label="Now"
                            unit=" h"
                            data=vec![
                                ("Prior authorizations", 96.0, 34.0),
                                ("Fax & document intake", 58.0, 14.0),
                                ("Phone & messages", 72.0, 49.0),
                                ("Claim edits & denials", 52.0, 27.0),
                                ("Referral coordination", 40.0, 18.0),
                            ]
                        />
                    </div>
                </Card>
                <Card>
                    <CardHeader title="Median authorization turnaround" subtitle="Business days from order to payer decision, weekly"/>
                    <div class="p-5">
                        <TrendLine
                            label="Median authorization turnaround by week"
                            unit=" d"
                            target=2.0
                            target_label="Target"
                            points=[9.6, 9.1, 8.8, 7.4, 6.1, 5.2, 4.4, 3.9, 3.4, 3.1, 2.8, 2.6].iter().enumerate().map(|(i, y)| (format!("W{}", i + 1), *y)).collect()
                        />
                    </div>
                </Card>
            </div>

            <div class="mt-6 grid gap-6 lg:grid-cols-[1fr_1fr]">
                <Card>
                    <CardHeader title="Work queues" subtitle="Items waiting · SLA attainment · oldest item"/>
                    <div class="grid gap-px bg-line sm:grid-cols-2">
                        {QUEUES
                            .iter()
                            .map(|(team, items, sla, oldest)| {
                                let tone = if *sla >= 95 { Tone::Sage } else if *sla >= 90 { Tone::Ocean } else { Tone::Warn };
                                view! {
                                    <div class="bg-white p-5">
                                        <div class="flex items-center justify-between">
                                            <div class="text-[13px] font-semibold text-ink">{*team}</div>
                                            <Badge tone=tone>{*sla} "% SLA"</Badge>
                                        </div>
                                        <div class="mt-2 font-display text-3xl font-semibold text-ink tabular-nums">{*items}</div>
                                        <div class="text-[12px] text-muted">"oldest: " {*oldest}</div>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </div>
                </Card>
                <Card>
                    <CardHeader title="Integration health" subtitle="Interfaces monitored every 60 seconds" icon=|| view! { <Icon name="Plug" class="size-4"/> }/>
                    <ul class="divide-y divide-line">
                        {INTEGRATIONS
                            .iter()
                            .map(|(name, ok, latency)| {
                                view! {
                                    <li class="flex items-center justify-between px-5 py-3 text-[13px]">
                                        <span class="flex items-center gap-2.5 font-medium text-ink">
                                            <span class=cx(&["size-2 rounded-full", if *ok { "bg-sage" } else { "animate-pulse-soft bg-warn" }])></span>
                                            {*name}
                                        </span>
                                        <span class="flex items-center gap-3">
                                            <span class="font-mono text-[12px] text-muted">{*latency}</span>
                                            {if *ok {
                                                view! { <Badge tone=Tone::Sage>"Healthy"</Badge> }.into_any()
                                            } else {
                                                view! { <Badge tone=Tone::Warn><Icon name="CircleAlert" class="size-3"/> " Degraded · queued"</Badge> }.into_any()
                                            }}
                                        </span>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                </Card>
            </div>
        </div>
    }
}
