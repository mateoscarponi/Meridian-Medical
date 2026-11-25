use crate::components::icons::Icon;
use crate::components::shell::ToolHeader;
use crate::components::ui::*;
use crate::domain::util::to_fixed;
use leptos::prelude::*;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Trigger,
    Step,
    Branch,
    Human,
}

struct Node {
    kind: Kind,
    title: &'static str,
    system: &'static str,
    ms: u64,
    log: &'static str,
}

struct Recipe {
    id: &'static str,
    name: &'static str,
    desc: &'static str,
    runs_week: u32,
    min_saved: u32,
    success: u32,
    enabled: bool,
    nodes: &'static [Node],
}

use Kind::*;

const fn n(kind: Kind, title: &'static str, system: &'static str, ms: u64, log: &'static str) -> Node {
    Node { kind, title, system, ms, log }
}

static RECIPES: [Recipe; 4] = [
    Recipe {
        id: "fax-intake",
        name: "Fax referral → booked appointment",
        desc: "From an inbound fax to a scheduled new-patient visit without retyping.",
        runs_week: 86,
        min_saved: 14,
        success: 96,
        enabled: true,
        nodes: &[
            n(Trigger, "Fax received", "eFax", 300, "DOC-5521 received from (516) 555-0172, 2 pages"),
            n(Step, "Read & classify", "Document AI", 900, "Referral (98%) · 7 fields extracted"),
            n(Step, "Match or create patient", "MPI · EMR FHIR", 500, "Matched MOS-205518 (0.97)"),
            n(Step, "Verify eligibility", "Clearinghouse 270/271", 800, "Empire BCBS active · specialist copay $40 · referral not required"),
            n(Branch, "Prior auth needed for first visit?", "Payer rules", 300, "No → continue"),
            n(Step, "Offer 3 slots by SMS", "Scheduling · SMS", 700, "Patient picked Tue 10:45 AM Garden City"),
            n(Step, "Send forms & imaging link", "Portal", 400, "Intake (ES) + HIPAA + imaging upload sent"),
            n(Step, "Notify referring office", "Direct messaging", 400, "Appointment confirmation sent to Patel Family Medicine"),
        ],
    },
    Recipe {
        id: "auth-chaser",
        name: "Authorization status chaser",
        desc: "Polls payers, updates orders, nudges humans only when needed.",
        runs_week: 140,
        min_saved: 9,
        success: 99,
        enabled: true,
        nodes: &[
            n(Trigger, "Every hour, 7am–7pm", "Scheduler", 300, "22 pending authorizations"),
            n(Step, "Check status", "X12 276/278 · payer APIs", 1000, "4 approved · 1 pended for info · 17 unchanged"),
            n(Step, "Write auth # to orders & claims", "EMR · Billing", 500, "4 orders updated (REF*G1)"),
            n(Branch, "Pended for more information?", "Rules", 300, "Yes (1) → Copilot assembles missing records"),
            n(Human, "Auth specialist approves resubmission", "Work queue", 800, "Approved by J. Alvarez"),
            n(Step, "Offer procedure slots to patients", "Scheduling · SMS", 500, "4 patients texted · 3 booked"),
        ],
    },
    Recipe {
        id: "denial-appeal",
        name: "Denial → appeal draft",
        desc: "Reads the 835, finds the root cause, drafts the appeal with chart evidence.",
        runs_week: 31,
        min_saved: 25,
        success: 88,
        enabled: true,
        nodes: &[
            n(Trigger, "835 remittance with denial", "Clearinghouse", 300, "CLM-90122 · CARC 197 (precert absent)"),
            n(Step, "Root-cause analysis", "Rules + LLM", 900, "Auth existed (HUM-AUTH-8841207) but was missing on the claim"),
            n(Branch, "Correctable without appeal?", "Rules", 300, "Yes → corrected claim (frequency code 7)"),
            n(Step, "Submit corrected claim", "837P", 600, "Resubmitted with REF*G1"),
            n(Step, "Add scrubber rule", "Claim Scrubber", 400, "Payer-specific auth check strengthened"),
        ],
    },
    Recipe {
        id: "post-op",
        name: "Post-op check-in",
        desc: "Day 1 and day 7 after surgery or injection, with red-flag escalation.",
        runs_week: 44,
        min_saved: 11,
        success: 97,
        enabled: false,
        nodes: &[
            n(Trigger, "Procedure completed", "EMR (HL7 SIU/DFT)", 300, "ACDF C5-C6 · Thomas Russo"),
            n(Step, "Day-1 SMS questionnaire", "SMS", 700, "Pain 4/10 · no fever · eating & drinking"),
            n(Branch, "Any red flag?", "NLP triage", 400, "No → schedule day-7 check"),
            n(Step, "Day-7 check & PT referral", "SMS · Meridian Motion PT", 600, "PT referral sent; first visit booked"),
        ],
    },
];

fn kind_icon(k: Kind) -> &'static str {
    match k {
        Trigger => "Bolt",
        Step => "Workflow",
        Branch => "GitBranch",
        Human => "UserCheck",
    }
}

fn recipe(id: &str) -> &'static Recipe {
    RECIPES.iter().find(|r| r.id == id).unwrap_or(&RECIPES[0])
}

#[component]
pub fn AutomationsPage() -> impl IntoView {
    let sel = RwSignal::new(RECIPES[0].id);
    let enabled = RwSignal::new(RECIPES.iter().map(|r| r.enabled).collect::<Vec<bool>>());
    let progress = RwSignal::new(-1i32);
    let running = RwSignal::new(false);
    let r = move || recipe(sel.get());
    let total_ms = move || r().nodes.iter().map(|n| n.ms).sum::<u64>();

    let simulate = move || {
        running.set(true);
        progress.set(0);
        let rr = recipe(sel.get_untracked());
        let mut t = 0u64;
        for (i, node) in rr.nodes.iter().enumerate() {
            t += node.ms;
            let last = i == rr.nodes.len() - 1;
            set_timeout(
                move || {
                    progress.set(i as i32 + 1);
                    if last {
                        running.set(false);
                    }
                },
                Duration::from_millis(t),
            );
        }
    };

    let weekly_hours = move || enabled.with(|e| RECIPES.iter().zip(e.iter()).filter(|(_, on)| **on).map(|(x, _)| (x.runs_week * x.min_saved) as f64 / 60.0).sum::<f64>());
    let on_count = move || enabled.with(|e| e.iter().filter(|x| **x).count());

    view! {
        <div class="mx-auto max-w-7xl">
            {move || {
                view! {
                    <ToolHeader
                        eyebrow="Data & integration · Workflow automation"
                        title="Automations"
                        description="Repeatable workflows built as event-driven recipes that span the EMR, payers, fax, SMS and staff queues. Humans approve the steps that need judgment, and every run is timed, so the savings are measured rather than estimated."
                        tags=&["event-driven", "idempotent steps", "retries + dead-letter", "human-in-the-loop", "run analytics"]
                        dyn_impact=vec![
                            (format!("{} h", to_fixed(weekly_hours(), 0)), "saved / week (enabled)".into()),
                            (format!("{}/{}", on_count(), RECIPES.len()), "recipes on".into()),
                        ]
                    />
                }
            }}

            <div class="grid gap-6 xl:grid-cols-[360px_1fr]">
                <div class="space-y-3">
                    {RECIPES
                        .iter()
                        .enumerate()
                        .map(|(i, x)| {
                            let on = move || enabled.with(|e| e[i]);
                            view! {
                                <div
                                    class=move || {
                                        cx(&[
                                            "rounded-2xl border border-line bg-white shadow-[var(--shadow-soft)]",
                                            "cursor-pointer p-4 transition",
                                            if sel.get() == x.id { "ring-2 ring-ocean" } else { "hover:shadow-[var(--shadow-lift)]" },
                                        ])
                                    }
                                    on:click=move |_| {
                                        sel.set(x.id);
                                        progress.set(-1);
                                    }
                                >
                                    <div class="flex items-start justify-between gap-3">
                                        <div>
                                            <div class="text-[14px] font-semibold text-ink">{x.name}</div>
                                            <div class="mt-0.5 text-[12px] text-muted">{x.desc}</div>
                                        </div>
                                        <button
                                            role="switch"
                                            aria-checked=move || on().to_string()
                                            aria-label=format!("Enable {}", x.name)
                                            on:click=move |ev| {
                                                ev.stop_propagation();
                                                enabled.update(|e| e[i] = !e[i]);
                                            }
                                            class=move || cx(&["relative h-5 w-9 shrink-0 rounded-full transition", if on() { "bg-sage" } else { "bg-line-2" }])
                                        >
                                            <span class=move || {
                                                cx(&["absolute top-0.5 size-4 rounded-full bg-white shadow transition-all", if on() { "left-[18px]" } else { "left-0.5" }])
                                            }></span>
                                        </button>
                                    </div>
                                    <div class="mt-3 grid grid-cols-3 gap-2 text-center">
                                        <div class="rounded-lg bg-cream py-1.5">
                                            <div class="text-[13px] font-semibold text-ink">{x.runs_week}</div>
                                            <div class="text-[10px] text-muted">"runs/wk"</div>
                                        </div>
                                        <div class="rounded-lg bg-cream py-1.5">
                                            <div class="text-[13px] font-semibold text-ink">{x.min_saved} "m"</div>
                                            <div class="text-[10px] text-muted">"saved/run"</div>
                                        </div>
                                        <div class="rounded-lg bg-cream py-1.5">
                                            <div class="text-[13px] font-semibold text-ink">{x.success} "%"</div>
                                            <div class="text-[10px] text-muted">"no-touch"</div>
                                        </div>
                                    </div>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>

                <Card>
                    <CardHeader
                        title=Signal::derive(move || r().name.to_string())
                        subtitle=Signal::derive(move || {
                            format!(
                                "{} steps · simulated run ≈ {} s · manual equivalent ≈ {} min",
                                r().nodes.len(),
                                to_fixed(total_ms() as f64 / 1000.0, 1),
                                r().min_saved
                            )
                        })
                        right=move || {
                            view! {
                                <Button on_click=Callback::new(move |_| simulate()) disabled=Signal::derive(move || running.get())>
                                    {move || {
                                        if running.get() {
                                            view! { <Icon name="Loader2" class="size-4 animate-spin"/> }.into_any()
                                        } else {
                                            view! { <Icon name="Play" class="size-4"/> }.into_any()
                                        }
                                    }}
                                    " Simulate run"
                                </Button>
                            }
                        }
                    />
                    <div class="p-6">
                        <ol class="relative space-y-3">
                            {move || {
                                let rr = r();
                                rr.nodes
                                    .iter()
                                    .enumerate()
                                    .map(|(i, node)| {
                                        let done = move || progress.get() > i as i32;
                                        let active = move || running.get() && progress.get() == i as i32;
                                        let kind = node.kind;
                                        view! {
                                            <li class="relative flex gap-4">
                                                {(i < rr.nodes.len() - 1)
                                                    .then(|| {
                                                        view! {
                                                            <div class=move || {
                                                                cx(&["absolute left-[19px] top-10 h-[calc(100%-16px)] w-0.5 transition-colors duration-500", if done() { "bg-sage" } else { "bg-line" }])
                                                            }></div>
                                                        }
                                                    })}
                                                <div class=move || {
                                                    cx(&[
                                                        "z-10 grid size-10 shrink-0 place-items-center rounded-xl transition-all duration-300",
                                                        if done() {
                                                            "bg-sage text-white"
                                                        } else if active() {
                                                            "bg-ocean text-white shadow-[var(--shadow-lift)]"
                                                        } else {
                                                            match kind {
                                                                Trigger => "bg-gold-soft text-gold",
                                                                Human => "bg-sky-2 text-ocean",
                                                                _ => "bg-sand text-muted",
                                                            }
                                                        },
                                                    ])
                                                }>
                                                    {move || {
                                                        if active() {
                                                            view! { <Icon name="Loader2" class="size-4 animate-spin"/> }.into_any()
                                                        } else if done() {
                                                            view! { <Icon name="CheckCircle2" class="size-4"/> }.into_any()
                                                        } else {
                                                            view! { <Icon name=kind_icon(kind) class="size-4"/> }.into_any()
                                                        }
                                                    }}
                                                </div>
                                                <div class=move || {
                                                    cx(&[
                                                        "flex-1 rounded-xl border px-4 py-3 transition-all duration-300",
                                                        if active() { "border-ocean bg-sky-2" } else if done() { "border-sage/30 bg-white" } else { "border-line bg-cream/40" },
                                                    ])
                                                }>
                                                    <div class="flex flex-wrap items-center justify-between gap-2">
                                                        <div class="flex items-center gap-2">
                                                            <span class="text-[13.5px] font-semibold text-ink">{node.title}</span>
                                                            {(kind == Human).then(|| view! { <Badge tone=Tone::Ocean>"Human approval"</Badge> })}
                                                            {(kind == Branch).then(|| view! { <Badge tone=Tone::Gold>"Condition"</Badge> })}
                                                        </div>
                                                        <span class="font-mono text-[11px] text-faint">{node.system}</span>
                                                    </div>
                                                    {move || {
                                                        if done() {
                                                            view! { <div class="animate-fade-up mt-1 font-mono text-[11.5px] text-sage">"✓ " {node.log}</div> }.into_any()
                                                        } else {
                                                            view! {
                                                                <div class="mt-1 flex items-center gap-1 text-[11.5px] text-faint">
                                                                    <Icon name="CircleDashed" class="size-3"/>
                                                                    " waiting"
                                                                </div>
                                                            }
                                                                .into_any()
                                                        }
                                                    }}
                                                </div>
                                            </li>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </ol>
                        <Show when=move || progress.get() == r().nodes.len() as i32 && !running.get()>
                            {move || {
                                let humans = r().nodes.iter().filter(|n| n.kind == Human).count();
                                view! {
                                    <div class="animate-fade-up mt-5 rounded-xl bg-sage-soft p-4 text-[13px] text-sage">
                                        <b>"Run completed"</b>
                                        " in "
                                        {to_fixed(total_ms() as f64 / 1000.0, 1)}
                                        " s with "
                                        {humans}
                                        " human touch"
                                        {if humans == 1 { "" } else { "es" }}
                                        ". Logged with inputs, outputs and timings for audit and ROI reporting."
                                    </div>
                                }
                            }}
                        </Show>
                    </div>
                </Card>
            </div>
        </div>
    }
}
