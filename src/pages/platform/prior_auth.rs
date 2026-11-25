use crate::components::browser::copy_text;
use crate::components::highlight::HighlightedText;
use crate::components::icons::Icon;
use crate::components::shell::ToolHeader;
use crate::components::ui::*;
use crate::domain::nlp::extract_entities;
use crate::domain::prior_auth::*;
use crate::domain::util::num;
use leptos::prelude::*;
use std::time::Duration;

const STEPS: [(&str, &str); 7] = [
    ("Eligibility & benefits", "X12 270/271 · active coverage, plan type, in-network"),
    ("Is authorization required?", "Payer rules / Da Vinci CRD lookup for CPT + plan"),
    ("Extract clinical evidence", "NLP over notes, imaging and PT records"),
    ("Match payer criteria", "Criterion-by-criterion with cited evidence"),
    ("Assemble packet", "Questionnaire auto-filled (DTR), attachments linked"),
    ("Draft medical-necessity letter", "Provider reviews and signs"),
    ("Submit & track", "Portal / X12 278 / FHIR PAS · status polled hourly"),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Idle,
    Running,
    Done,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StepState {
    Pending,
    Active,
    Done,
    Skipped,
}

fn case(id: &str) -> &'static AuthCase {
    AUTH_CASES.iter().find(|c| c.id == id).unwrap_or(&AUTH_CASES[0])
}

#[component]
pub fn PriorAuthPage() -> impl IntoView {
    let selected = RwSignal::new(AUTH_CASES[0].id);
    let phase = RwSignal::new(Phase::Idle);
    let step_idx = RwSignal::new(-1i32);
    let submitted = RwSignal::new(Vec::<&'static str>::new());
    let copied = RwSignal::new(false);
    let timer = StoredValue::new(None::<TimeoutHandle>);
    let evals: Vec<(f64, bool)> = AUTH_CASES.iter().map(|a| { let e = evaluate_case(a); (e.readiness_after_fix, !e.blocking.is_empty()) }).collect();

    let c = move || case(selected.get());
    let ev = Memo::new(move |_| {
        let e = evaluate_case(c());
        (e.results, e.readiness, e.readiness_after_fix, e.blocking)
    });
    let entities = Signal::derive(move || extract_entities(c().note));
    let not_required = move || c().pathway == "NY WCB Medical Portal";
    let blocked = move || ev.with(|e| !e.3.is_empty());
    let is_submitted = move || submitted.with(|s| s.contains(&selected.get()));

    let cancel = move || {
        if let Some(h) = timer.get_value() {
            h.clear();
        }
    };
    let select = move |id: &'static str| {
        selected.set(id);
        phase.set(Phase::Idle);
        step_idx.set(-1);
        cancel();
    };

    fn tick(i: i32, last_step: i32, step_idx: RwSignal<i32>, phase: RwSignal<Phase>, timer: StoredValue<Option<TimeoutHandle>>) {
        let h = set_timeout_with_handle(
            move || {
                step_idx.set(i + 1);
                if i >= last_step {
                    phase.set(Phase::Done);
                } else {
                    tick(i + 1, last_step, step_idx, phase, timer);
                }
            },
            Duration::from_millis(520),
        )
        .ok();
        timer.set_value(h);
    }
    let run = move || {
        phase.set(Phase::Running);
        step_idx.set(0);
        let last_step = if blocked() { 3 } else { STEPS.len() as i32 - 2 };
        tick(0, last_step, step_idx, phase, timer);
    };
    on_cleanup(move || {
        if let Some(h) = timer.try_get_value().flatten() {
            h.clear();
        }
    });

    let step_state = move |i: i32| {
        let ph = phase.get();
        let si = step_idx.get();
        if ph == Phase::Idle {
            return StepState::Pending;
        }
        if blocked() && i > 3 && ph == Phase::Done {
            return StepState::Skipped;
        }
        if i < si {
            return StepState::Done;
        }
        if i == si && ph == Phase::Running {
            return StepState::Active;
        }
        if ph == Phase::Done && i == STEPS.len() as i32 - 1 {
            return if is_submitted() { StepState::Done } else { StepState::Pending };
        }
        if i <= si {
            StepState::Done
        } else {
            StepState::Pending
        }
    };
    let submit = move || {
        let id = selected.get_untracked();
        submitted.update(|s| s.push(id));
    };

    view! {
        <div class="mx-auto max-w-7xl">
            <ToolHeader
                eyebrow="Revenue cycle · Insurance authorization support"
                title="Prior Auth Copilot"
                description="Reads the chart, checks payer criteria with cited evidence, fills the questionnaire, drafts the letter of medical necessity and submits. It also tells staff when a request will be denied, before anyone sends it."
                tags=&["NLP evidence extraction", "X12 270/271 · 278", "Da Vinci CRD · DTR · PAS", "NY WCB MTG", "CMS-0057-F"]
                impact=&[("4 min", "per request (was 45)"), ("2.6 d", "median decision")]
            />

            <div class="grid gap-6 xl:grid-cols-[300px_1fr]">
                <Card class="h-fit overflow-hidden">
                    <CardHeader title="Queue" subtitle=format!("{} requests · 2 urgent today", AUTH_CASES.len())/>
                    <ul class="divide-y divide-line">
                        {AUTH_CASES
                            .iter()
                            .zip(evals.into_iter())
                            .map(|(a, (after_fix, has_blocking))| {
                                let id = a.id;
                                let is_sel = move || selected.get() == id;
                                view! {
                                    <li>
                                        <button
                                            on:click=move |_| select(id)
                                            class=move || {
                                                cx(&["w-full px-4 py-3.5 text-left transition", if is_sel() { "bg-sky-2 shadow-[inset_3px_0_0_var(--color-ocean)]" } else { "hover:bg-cream" }])
                                            }
                                        >
                                            <div class="flex items-center justify-between gap-2">
                                                <span class="font-mono text-[11px] text-faint">{a.id}</span>
                                                {move || {
                                                    if submitted.with(|s| s.contains(&id)) {
                                                        view! { <Badge tone=Tone::Sage>"Submitted"</Badge> }.into_any()
                                                    } else if a.pathway == "NY WCB Medical Portal" {
                                                        view! { <Badge tone=Tone::Ocean>"Within MTG"</Badge> }.into_any()
                                                    } else if has_blocking {
                                                        view! { <Badge tone=Tone::Warn>"Not ready"</Badge> }.into_any()
                                                    } else {
                                                        view! { <Badge tone=Tone::Sage>"Ready"</Badge> }.into_any()
                                                    }
                                                }}
                                            </div>
                                            <div class="mt-1 text-[14px] font-semibold text-ink">{a.patient}</div>
                                            <div class="text-[12px] text-muted">"CPT " {a.cpt} " · " {a.payer}</div>
                                            <div class="mt-2 flex items-center gap-2">
                                                <Progress value=after_fix tone={if has_blocking { ProgressTone::Warn } else { ProgressTone::Sage }}/>
                                                <span class="text-[11px] font-semibold tabular-nums text-muted">{num(after_fix)} "%"</span>
                                            </div>
                                        </button>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                </Card>

                <div class="space-y-6">
                    <Card>
                        <div class="flex flex-wrap items-start justify-between gap-4 p-5">
                            <div>
                                <div class="flex flex-wrap items-center gap-2">
                                    <h2 class="font-display text-2xl font-semibold text-ink">{move || c().patient}</h2>
                                    <Badge tone=Tone::Neutral>{move || c().payer_type}</Badge>
                                    <Badge tone=Tone::Ocean>{move || c().pathway}</Badge>
                                </div>
                                <p class="mt-1 text-[14px] text-ink">
                                    <span class="font-mono font-semibold">"CPT " {move || c().cpt}</span>
                                    " · "
                                    {move || c().service}
                                </p>
                                <p class="mt-0.5 text-[13px] text-muted">
                                    "Dx " {move || c().icd10.join(", ")} " · Ordering: " {move || c().provider} " · Received "
                                    {move || if c().received_days_ago == 0 { "today".to_string() } else { format!("{} d ago", c().received_days_ago) }}
                                </p>
                            </div>
                            <div class="flex items-center gap-2">
                                {move || {
                                    if phase.get() == Phase::Running {
                                        view! {
                                            <Button disabled=true>
                                                <Icon name="Loader2" class="size-4 animate-spin"/>
                                                " Working…"
                                            </Button>
                                        }
                                            .into_any()
                                    } else {
                                        let done = phase.get() == Phase::Done;
                                        view! {
                                            <Button on_click=Callback::new(move |_| run()) variant={if done { BtnVariant::Secondary } else { BtnVariant::Primary }}>
                                                <Icon name="Sparkles" class="size-4"/>
                                                " "
                                                {if done { "Run again" } else { "Run Copilot" }}
                                            </Button>
                                        }
                                            .into_any()
                                    }
                                }}
                            </div>
                        </div>
                        <div class="grid gap-px border-t border-line bg-line md:grid-cols-7">
                            {STEPS
                                .iter()
                                .enumerate()
                                .map(|(i, (label, detail))| {
                                    let st = move || step_state(i as i32);
                                    view! {
                                        <div class=move || cx(&["bg-white p-3 transition-colors", if st() == StepState::Active { "bg-sky-2" } else { "" }])>
                                            <div class="flex items-center gap-1.5">
                                                {move || match st() {
                                                    StepState::Done => view! { <Icon name="CheckCircle2" class="size-4 text-sage"/> }.into_any(),
                                                    StepState::Active => view! { <Icon name="Loader2" class="size-4 animate-spin text-ocean"/> }.into_any(),
                                                    StepState::Pending => view! { <Icon name="CircleDashed" class="size-4 text-line-2"/> }.into_any(),
                                                    StepState::Skipped => view! { <Icon name="MinusCircle" class="size-4 text-warn"/> }.into_any(),
                                                }}
                                                <span class="text-[11px] font-bold text-faint">{i + 1}</span>
                                            </div>
                                            <div class="mt-1.5 text-[12px] font-semibold leading-snug text-ink">{*label}</div>
                                            <div class="mt-0.5 hidden text-[11px] leading-snug text-muted xl:block">{*detail}</div>
                                        </div>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </Card>

                    <div class="grid gap-6 lg:grid-cols-2">
                        <Card>
                            <CardHeader
                                title="Chart evidence"
                                subtitle="Entities found by the NLP layer are highlighted"
                                icon=|| view! { <Icon name="FileText" class="size-4"/> }
                            />
                            <div class="p-5 text-[14px] leading-7 text-ink">
                                <HighlightedText text=Signal::derive(move || c().note.to_string()) entities=entities/>
                            </div>
                            <div class="flex flex-wrap gap-2 border-t border-line px-5 py-3">
                                {move || {
                                    c()
                                        .attachments
                                        .iter()
                                        .map(|a| {
                                            view! {
                                                <span class="inline-flex items-center gap-1 rounded-lg bg-sand px-2 py-1 text-[11px] font-medium text-muted">
                                                    <Icon name="Paperclip" class="size-3"/>
                                                    {*a}
                                                </span>
                                            }
                                        })
                                        .collect_view()
                                }}
                            </div>
                        </Card>

                        <Card>
                            <CardHeader
                                title="Payer criteria"
                                subtitle=Signal::derive(move || {
                                    if not_required() { "NY Workers' Compensation Medical Treatment Guidelines".to_string() } else { format!("{} policy for CPT {}", c().payer, c().cpt) }
                                })
                                right=move || {
                                    view! {
                                        <div class="text-right">
                                            <div class="font-display text-2xl font-semibold tabular-nums text-ink">
                                                {move || num(if phase.get() == Phase::Done { ev.with(|e| e.2) } else { ev.with(|e| e.1) })}
                                                "%"
                                            </div>
                                            <div class="text-[11px] text-muted">"readiness"</div>
                                        </div>
                                    }
                                }
                            />
                            <ul class="divide-y divide-line">
                                {move || {
                                    let done = phase.get() == Phase::Done;
                                    ev.with(|e| e.0.clone())
                                        .into_iter()
                                        .map(|r| {
                                            let auto_fixed = done && r.status == CritStatus::Missing && r.fix.as_deref().is_some_and(|f| f.starts_with("Auto-pulled"));
                                            let status = if auto_fixed { CritStatus::Met } else { r.status };
                                            view! {
                                                <li class="px-5 py-3.5">
                                                    <div class="flex items-start gap-2.5">
                                                        {match status {
                                                            CritStatus::Met => view! { <Icon name="Check" class="mt-0.5 size-4 shrink-0 text-sage"/> }.into_any(),
                                                            CritStatus::Missing => view! { <Icon name="XCircle" class="mt-0.5 size-4 shrink-0 text-danger"/> }.into_any(),
                                                            CritStatus::Na => view! { <Icon name="MinusCircle" class="mt-0.5 size-4 shrink-0 text-faint"/> }.into_any(),
                                                        }}
                                                        <div class="min-w-0">
                                                            <div class="text-[13px] font-semibold text-ink">{r.label.clone()}</div>
                                                            <div class="mt-0.5 text-[12px] leading-relaxed text-muted">
                                                                {if r.evidence.is_empty() { "—".to_string() } else { r.evidence.clone() }}
                                                            </div>
                                                            {(r.status == CritStatus::Missing)
                                                                .then(|| r.fix.clone())
                                                                .flatten()
                                                                .map(|fix| {
                                                                    view! {
                                                                        <div class=cx(&[
                                                                            "mt-2 rounded-lg px-2.5 py-1.5 text-[12px]",
                                                                            if auto_fixed { "bg-sage-soft text-sage" } else { "bg-warn-soft text-warn" },
                                                                        ])>{if auto_fixed { "✓ " } else { "" }} {fix}</div>
                                                                    }
                                                                })}
                                                        </div>
                                                    </div>
                                                </li>
                                            }
                                        })
                                        .collect_view()
                                }}
                            </ul>
                        </Card>
                    </div>

                    {move || {
                        (phase.get() == Phase::Done)
                            .then(|| {
                                let cc = c();
                                let inner = if not_required() {
                                    view! {
                                        <Card class="border-ocean/30 bg-sky-2 p-5">
                                            <div class="flex items-start gap-3">
                                                <Icon name="CheckCircle2" class="mt-0.5 size-5 text-ocean"/>
                                                <div>
                                                    <div class="text-[15px] font-semibold text-ink">
                                                        "No authorization request needed: treatment is within the NY Medical Treatment Guidelines."
                                                    </div>
                                                    <p class="mt-1 text-[13px] text-muted">
                                                        "Copilot documented MTG compliance in the chart, scheduled the procedure and set the bill to include the WCB case number. A variance PAR in the WCB Medical Portal is only drafted when a request falls outside the guidelines."
                                                    </p>
                                                    <Button class="mt-3" size=BtnSize::Sm on_click=Callback::new(move |_| submit()) disabled=Signal::derive(is_submitted)>
                                                        {move || if is_submitted() { "Scheduled ✓" } else { "Schedule procedure" }}
                                                    </Button>
                                                </div>
                                            </div>
                                        </Card>
                                    }
                                        .into_any()
                                } else if blocked() {
                                    let blocking = ev.with(|e| e.3.clone());
                                    view! {
                                        <Card class="border-warn/30 bg-warn-soft p-5">
                                            <div class="flex items-start gap-3">
                                                <Icon name="AlertTriangle" class="mt-0.5 size-5 text-warn"/>
                                                <div>
                                                    <div class="text-[15px] font-semibold text-ink">"Hold. This request would likely be denied as written."</div>
                                                    <ul class="mt-2 space-y-1 text-[13px] text-muted">
                                                        {blocking
                                                            .into_iter()
                                                            .map(|b| {
                                                                let text = b.fix.clone().unwrap_or(b.evidence.clone());
                                                                view! { <li>"• " <b class="text-ink">{b.label.clone()} ":"</b> " " {text}</li> }
                                                            })
                                                            .collect_view()}
                                                    </ul>
                                                    <p class="mt-2 text-[13px] text-muted">
                                                        "Copilot added a task to re-evaluate on "
                                                        <b class="text-ink">"Mon, Nov 30"</b>
                                                        " (week 6 of therapy) and will re-run automatically when the PT note arrives. One avoided denial saves about 2 hours of rework and a two-week delay for the patient."
                                                    </p>
                                                </div>
                                            </div>
                                        </Card>
                                    }
                                        .into_any()
                                } else {
                                    let letter = ev.with(|e| medical_necessity_letter(cc, &e.0));
                                    let letter2 = letter.clone();
                                    let ma = cc.payer_type == "Medicare Advantage";
                                    view! {
                                        <div class="grid gap-6 lg:grid-cols-[1.3fr_1fr]">
                                            <Card>
                                                <CardHeader
                                                    title="Letter of medical necessity (draft)"
                                                    subtitle="Generated from criteria evidence. Requires provider review and signature."
                                                    right=move || {
                                                        let letter = letter2.clone();
                                                        view! {
                                                            <Button
                                                                size=BtnSize::Sm
                                                                variant=BtnVariant::Secondary
                                                                on_click=Callback::new(move |_| {
                                                                    copy_text(&letter);
                                                                    copied.set(true);
                                                                    set_timeout(move || copied.set(false), Duration::from_millis(1500));
                                                                })
                                                            >
                                                                <Icon name="Copy" class="size-3.5"/>
                                                                " "
                                                                {move || if copied.get() { "Copied" } else { "Copy" }}
                                                            </Button>
                                                        }
                                                    }
                                                />
                                                <textarea class="scroll-thin h-80 w-full resize-none bg-cream/50 p-5 font-mono text-[12px] leading-relaxed text-ink outline-none">
                                                    {letter}
                                                </textarea>
                                            </Card>
                                            <Card class="flex flex-col">
                                                <CardHeader title="Submission" subtitle=cc.pathway/>
                                                <div class="flex-1 space-y-3 p-5 text-[13px]">
                                                    <div class="flex justify-between">
                                                        <span class="text-muted">"Packet"</span>
                                                        <span class="font-semibold text-ink">{cc.attachments.len() + 2} " documents"</span>
                                                    </div>
                                                    <div class="flex justify-between">
                                                        <span class="text-muted">"Questionnaire"</span>
                                                        <span class="font-semibold text-ink">"auto-filled 14 / 14"</span>
                                                    </div>
                                                    <div class="flex justify-between">
                                                        <span class="text-muted">"Staff time"</span>
                                                        <span class="font-semibold text-sage">"4 min (vs. ~45 min manual)"</span>
                                                    </div>
                                                    <div class="flex justify-between">
                                                        <span class="text-muted">"Expected decision"</span>
                                                        <span class="font-semibold text-ink">
                                                            {if ma { "≤ 7 calendar days (CMS-0057-F)" } else { "3–5 business days" }}
                                                        </span>
                                                    </div>
                                                    {ma
                                                        .then(|| {
                                                            view! {
                                                                <p class="rounded-lg bg-sky-2 p-2.5 text-[12px] text-ocean">
                                                                    "Medicare Advantage plans must decide standard requests within 7 calendar days (72 h expedited) and expose a FHIR Prior Authorization API by Jan 1, 2027. This request goes through that API."
                                                                </p>
                                                            }
                                                        })}
                                                </div>
                                                <div class="border-t border-line p-5">
                                                    {move || {
                                                        if is_submitted() {
                                                            view! {
                                                                <div class="flex items-center gap-2 text-[13px] font-semibold text-sage">
                                                                    <Icon name="Clock" class="size-4"/>
                                                                    " Submitted · tracking # PA-"
                                                                    {cc.id.get(3..).unwrap_or("")}
                                                                    "-NY · status checked hourly"
                                                                </div>
                                                            }
                                                                .into_any()
                                                        } else {
                                                            view! {
                                                                <Button class="w-full" variant=BtnVariant::Sage on_click=Callback::new(move |_| submit())>
                                                                    <Icon name="Send" class="size-4"/>
                                                                    " Approve & submit"
                                                                </Button>
                                                            }
                                                                .into_any()
                                                        }
                                                    }}
                                                </div>
                                            </Card>
                                        </div>
                                    }
                                        .into_any()
                                };
                                view! { <div class="animate-fade-up">{inner}</div> }
                            })
                    }}
                </div>
            </div>
        </div>
    }
}
