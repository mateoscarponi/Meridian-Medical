use crate::components::icons::Icon;
use crate::components::shell::ToolHeader;
use crate::components::ui::*;
use leptos::prelude::*;

const STAGES: [&str; 5] = ["Received", "Insurance verified", "Scheduled", "Seen", "Loop closed"];

#[derive(Clone, PartialEq)]
struct Ref {
    id: &'static str,
    patient: &'static str,
    from: &'static str,
    via: &'static str,
    reason: &'static str,
    stage: usize,
    days: u32,
    auto: Option<&'static str>,
}

fn initial() -> Vec<Ref> {
    let r = |id, patient, from, via, reason, stage, days| Ref { id, patient, from, via, reason, stage, days, auto: None };
    vec![
        r("R-3301", "Carmen Diaz", "Patel Family Medicine", "Fax", "Right sciatica, 5 wk", 0, 0),
        r("R-3302", "Grace Kim", "Northwell PCP – Garden City", "FHIR", "Neck pain w/ arm numbness", 0, 1),
        r("R-3303", "Luis Moreno", "Queens Medical Assoc.", "Direct", "Lumbar stenosis", 1, 1),
        r("R-3304", "Ana Pereira", "Meridian Motion PT", "Portal", "Failed PT, consider ESI", 1, 3),
        r("R-3305", "Priya Natarajan", "Garden City Pediatrics & Adult", "Fax", "Adult scoliosis", 2, 2),
        r("R-3306", "Thomas Russo", "Dr. Feld – Internal Medicine", "Direct", "C5-C6 radiculopathy", 3, 6),
        r("R-3307", "Linda Chen", "Flushing Family Care", "FHIR", "Claudication", 3, 2),
        r("R-3308", "Harold Becker", "Babylon Medical Group", "Fax", "Lumbar spondylosis", 4, 0),
    ]
}

fn via_icon(via: &str) -> &'static str {
    match via {
        "Fax" => "Printer",
        "Direct" => "Mail",
        "FHIR" => "Network",
        _ => "Share2",
    }
}

/// What runs at each stage (the "Auto: " prefix of the original is dropped where it was stripped).
fn next_action(stage: usize) -> &'static str {
    match stage {
        0 => "match patient → check eligibility (270/271)",
        1 => "text patient 3 slot options",
        2 => "reminders + pre-visit checklist",
        3 => "send signed consult note to referrer",
        _ => "Done",
    }
}

const LAST: usize = 4;

#[component]
pub fn ReferralsPage() -> impl IntoView {
    let refs = RwSignal::new(initial());
    let auto_on = RwSignal::new(true);
    let advance = move |id: &'static str| {
        let on = auto_on.get_untracked();
        refs.update(|rs| {
            for r in rs.iter_mut().filter(|r| r.id == id && r.stage != LAST) {
                r.auto = on.then(|| next_action(r.stage));
                r.stage += 1;
                r.days = 0;
            }
        })
    };
    let run_all = move || {
        refs.update(|rs| {
            for r in rs.iter_mut().filter(|r| r.stage != LAST) {
                r.auto = Some(next_action(r.stage));
                r.stage += 1;
                r.days = 0;
            }
        })
    };
    let stalled = move || refs.with(|rs| rs.iter().filter(|r| r.days >= 3 && r.stage != LAST).count());

    view! {
        <div class="mx-auto max-w-7xl">
            <ToolHeader
                eyebrow="Front office · Referral management"
                title="Referral Tracker"
                description="Every referral, whether fax, Direct message, FHIR or portal, moves through one pipeline with SLAs. Automations do the repetitive steps and close the loop with the referring physician, which is what keeps referrals coming."
                tags=&["Direct secure messaging", "FHIR ServiceRequest", "eligibility 270/271", "loop closure", "leakage analytics"]
                impact=&[("5.8→1.9 d", "referral to appointment"), ("6%", "leakage (was 22%)")]
            />

            <div class="mb-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                <Stat label="Open referrals" value=Signal::derive(move || refs.with(|rs| rs.iter().filter(|r| r.stage != LAST).count()).to_string()) tone=StatTone::Neutral/>
                {move || {
                    let s = stalled();
                    view! {
                        <Stat
                            label="Stalled ≥ 3 days"
                            value=s.to_string()
                            tone={if s > 0 { StatTone::Danger } else { StatTone::Sage }}
                            delta=Some(if s > 0 { "needs attention".to_string() } else { "none".to_string() })
                        />
                    }
                }}
                <Stat label="Loops closed this month" value="142" delta=Some("98%".to_string()) hint="consult notes returned to referrers"/>
                <Stat label="Top referrer" value="Northwell PCPs" hint="31 referrals · 30 days" tone=StatTone::Neutral/>
            </div>

            <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
                <label class="inline-flex cursor-pointer items-center gap-2 text-[13px] font-semibold text-ink">
                    <button
                        role="switch"
                        aria-checked=move || auto_on.get().to_string()
                        on:click=move |_| auto_on.update(|v| *v = !*v)
                        class=move || cx(&["relative h-6 w-11 rounded-full transition", if auto_on.get() { "bg-sage" } else { "bg-line-2" }])
                    >
                        <span class=move || {
                            cx(&["absolute top-0.5 size-5 rounded-full bg-white shadow transition-all", if auto_on.get() { "left-[22px]" } else { "left-0.5" }])
                        }></span>
                    </button>
                    "Automations "
                    {move || if auto_on.get() { "on" } else { "off" }}
                </label>
                <Button size=BtnSize::Sm on_click=Callback::new(move |_| run_all())>
                    <Icon name="Zap" class="size-3.5"/>
                    " Run automations on all"
                </Button>
            </div>

            <div class="scroll-thin -mx-4 overflow-x-auto px-4 pb-2 sm:-mx-8 sm:px-8">
                <div class="grid min-w-[1100px] grid-cols-5 gap-4">
                    {STAGES
                        .iter()
                        .enumerate()
                        .map(|(si, s)| {
                            let items = move || refs.with(|rs| rs.iter().filter(|r| r.stage == si).cloned().collect::<Vec<_>>());
                            view! {
                                <div class="rounded-2xl bg-sand/70 p-3">
                                    <div class="mb-3 flex items-center justify-between px-1">
                                        <span class="text-[12px] font-bold uppercase tracking-wide text-muted">{*s}</span>
                                        <span class="rounded-full bg-white px-2 text-[11px] font-semibold text-ink">{move || items().len()}</span>
                                    </div>
                                    <div class="space-y-2.5">
                                        <For each=items key=|r| (r.id, r.stage) let:r>
                                            <Card class="animate-fade-up p-3.5">
                                                <div class="flex items-center justify-between">
                                                    <span class="font-mono text-[10.5px] text-faint">{r.id}</span>
                                                    <span class="inline-flex items-center gap-1 text-[11px] text-muted">
                                                        <Icon name=via_icon(r.via) class="size-3"/>
                                                        {r.via}
                                                    </span>
                                                </div>
                                                <div class="mt-1 text-[13.5px] font-semibold text-ink">{r.patient}</div>
                                                <div class="text-[12px] text-muted">{r.reason}</div>
                                                <div class="mt-1 truncate text-[11px] text-faint">"from " {r.from}</div>
                                                {r.auto.map(|a| view! { <div class="mt-2 rounded-md bg-sage-soft px-2 py-1 text-[11px] text-sage">"✓ " {a}</div> })}
                                                <div class="mt-2.5 flex items-center justify-between">
                                                    {if r.days >= 3 && si != LAST {
                                                        view! { <Badge tone=Tone::Danger><Icon name="Clock" class="size-3"/> " " {r.days} " d"</Badge> }.into_any()
                                                    } else {
                                                        view! {
                                                            <span class="text-[11px] text-faint">
                                                                {if r.days == 0 { "today".to_string() } else { format!("{} d in stage", r.days) }}
                                                            </span>
                                                        }
                                                            .into_any()
                                                    }}
                                                    {(si != LAST)
                                                        .then(|| {
                                                            let id = r.id;
                                                            view! {
                                                                <button
                                                                    on:click=move |_| advance(id)
                                                                    class="inline-flex items-center gap-1 rounded-md px-2 py-1 text-[11px] font-semibold text-ocean hover:bg-sky-2"
                                                                >
                                                                    "Advance "
                                                                    <Icon name="ArrowRight" class="size-3"/>
                                                                </button>
                                                            }
                                                        })}
                                                </div>
                                            </Card>
                                        </For>
                                    </div>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </div>

            <Card class="mt-6">
                <CardHeader title="Automation rules" subtitle="What runs at each stage when automations are on"/>
                <div class="grid gap-px bg-line sm:grid-cols-2 lg:grid-cols-4">
                    {STAGES[..4]
                        .iter()
                        .enumerate()
                        .map(|(i, s)| {
                            view! {
                                <div class="bg-white p-4">
                                    <div class="text-[12px] font-bold uppercase tracking-wide text-faint">{*s}</div>
                                    <div class="mt-1.5 text-[13px] text-ink">{next_action(i)}</div>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </Card>
        </div>
    }
}
