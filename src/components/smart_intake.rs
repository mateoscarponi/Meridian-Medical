//! Smart appointment request: a four-step, fully client-side triage that recommends a visit.
//! Nothing identifying is collected here; the result is handed to the portal via sessionStorage.

use super::icons::Icon;
use super::ui::cx;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Area {
    Neck,
    Back,
    Leg,
    Joint,
    Injury,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Dur {
    Lt2,
    TwoTo6,
    SixTo12,
    Gt12,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Ins {
    Commercial,
    Medicare,
    Wc,
    NoFault,
    Medicaid,
    SelfPay,
}

const AREAS: [(Area, &str); 5] = [
    (Area::Neck, "Neck / arm pain"),
    (Area::Back, "Lower back pain"),
    (Area::Leg, "Sciatica / leg pain"),
    (Area::Joint, "Joint pain"),
    (Area::Injury, "New injury"),
];
const DURS: [(Dur, &str); 4] = [(Dur::Lt2, "< 2 weeks"), (Dur::TwoTo6, "2–6 weeks"), (Dur::SixTo12, "6–12 weeks"), (Dur::Gt12, "3+ months")];
const INS: [(Ins, &str); 6] = [
    (Ins::Commercial, "Commercial"),
    (Ins::Medicare, "Medicare"),
    (Ins::Wc, "Workers' Comp"),
    (Ins::NoFault, "No-Fault (auto)"),
    (Ins::Medicaid, "Medicaid"),
    (Ins::SelfPay, "Self-pay"),
];
const FLAGS: [&str; 4] = [
    "Loss of bladder or bowel control",
    "Numbness in the groin / inner thighs",
    "Rapidly worsening leg or arm weakness",
    "Fever with severe back pain",
];

/// The recommendation handed to the secure portal.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rec {
    pub service: String,
    pub who: String,
    #[serde(rename = "where")]
    pub where_: String,
    pub when: String,
    pub bring: Vec<String>,
}

fn recommend(area: Area, dur: Dur, ins: Ins) -> Rec {
    let mut service = "Spine evaluation";
    let mut who = "Daniel Park, PA-C";
    let mut where_ = "Garden City";
    let mut when = "Tue, Nov 17 · 10:45 AM";
    if area == Area::Injury || (dur == Dur::Lt2 && area == Area::Joint) {
        service = "Walk-in injury clinic";
        who = "Walk-in team";
        where_ = "Hicksville";
        when = "Today · walk in, avg. wait 25 min";
    } else if area == Area::Joint {
        service = "Orthopedic consultation";
        who = "Dr. Elena Ruiz, MD";
        when = "Wed, Nov 18 · 9:15 AM";
    } else if dur == Dur::Gt12 || dur == Dur::SixTo12 {
        service = if area == Area::Neck { "Spine surgery consultation (cervical)" } else { "Interventional spine consultation" };
        who = if dur == Dur::Gt12 { "Dr. Adrian Hale, MD" } else { "Dr. Samuel Okafor, MD" };
        where_ = if dur == Dur::Gt12 { "Huntington" } else { "Garden City" };
        when = if dur == Dur::Gt12 { "Thu, Nov 19 · 1:00 PM" } else { "Wed, Nov 18 · 2:30 PM" };
    }
    let mut bring: Vec<String> = ["Photo ID & insurance card", "Any MRI / X-ray discs and reports", "Medication list"].map(String::from).to_vec();
    match ins {
        Ins::Wc => bring.extend(["WCB case # and carrier claim #", "Date of injury & employer info"].map(String::from)),
        Ins::NoFault => bring.extend(["No-fault claim # and NF-2 application", "Date of accident & police report #"].map(String::from)),
        Ins::Commercial => bring.push("PCP referral if your plan requires one".into()),
        Ins::Medicaid => bring.push("Note: Medicaid is accepted for established patients & hospital follow-ups".into()),
        _ => {}
    }
    Rec { service: service.into(), who: who.into(), where_: where_.into(), when: when.into(), bring }
}

#[component]
fn Chip(#[prop(into)] active: Signal<bool>, on_click: impl Fn() + 'static, label: &'static str) -> impl IntoView {
    view! {
        <button
            type="button"
            on:click=move |_| on_click()
            class=move || {
                cx(&[
                    "rounded-xl px-3 py-2 text-[13px] font-semibold ring-1 ring-inset transition-all duration-200",
                    if active.get() { "bg-navy text-white ring-navy shadow-sm" } else { "bg-white text-ink ring-line-2 hover:ring-ocean hover:text-ocean" },
                ])
            }
        >
            {label}
        </button>
    }
}

#[component]
pub fn SmartIntake() -> impl IntoView {
    let step = RwSignal::new(0usize);
    let area = RwSignal::new(None::<Area>);
    let dur = RwSignal::new(None::<Dur>);
    let flags = RwSignal::new(Vec::<&'static str>::new());
    let ins = RwSignal::new(None::<Ins>);
    let navigate = use_navigate();

    let rec = Memo::new(move |_| match (area.get(), dur.get(), ins.get()) {
        (Some(a), Some(d), Some(i)) => Some(recommend(a, d, i)),
        _ => None,
    });
    let urgent = move || !flags.with(|f| f.is_empty());
    let can_next = move || match step.get() {
        0 => area.get().is_some(),
        1 => dur.get().is_some(),
        2 => true,
        3 => ins.get().is_some(),
        _ => false,
    };

    let fieldset = move |s: usize| match s {
        0 => view! {
            <fieldset class="mt-5">
                <legend class="mb-3 text-[14px] font-semibold text-ink">"What brings you in?"</legend>
                <div class="flex flex-wrap gap-2">
                    {AREAS.iter().map(|(v, label)| { let v = *v; view! { <Chip active=Signal::derive(move || area.get() == Some(v)) on_click=move || area.set(Some(v)) label=label/> } }).collect_view()}
                </div>
            </fieldset>
        }
        .into_any(),
        1 => view! {
            <fieldset class="mt-5">
                <legend class="mb-3 text-[14px] font-semibold text-ink">"How long has it been going on?"</legend>
                <div class="flex flex-wrap gap-2">
                    {DURS.iter().map(|(v, label)| { let v = *v; view! { <Chip active=Signal::derive(move || dur.get() == Some(v)) on_click=move || dur.set(Some(v)) label=label/> } }).collect_view()}
                </div>
            </fieldset>
        }
        .into_any(),
        2 => view! {
            <fieldset class="mt-5">
                <legend class="mb-3 text-[14px] font-semibold text-ink">
                    "Any of these right now? "
                    <span class="font-normal text-muted">"(select all that apply)"</span>
                </legend>
                <div class="grid gap-2 sm:grid-cols-2">
                    {FLAGS
                        .iter()
                        .map(|f| {
                            let f = *f;
                            let checked = move || flags.with(|x| x.contains(&f));
                            view! {
                                <label class=move || {
                                    cx(&[
                                        "flex cursor-pointer items-center gap-2 rounded-xl px-3 py-2 text-[13px] ring-1 ring-inset transition",
                                        if checked() { "bg-danger-soft text-danger ring-danger/30" } else { "bg-white ring-line-2 hover:ring-ocean" },
                                    ])
                                }>
                                    <input
                                        type="checkbox"
                                        class="accent-[#b4483c]"
                                        prop:checked=checked
                                        on:change=move |ev| {
                                            let on = event_target_checked(&ev);
                                            flags.update(|x| if on { x.push(f) } else { x.retain(|y| *y != f) });
                                        }
                                    />
                                    {f}
                                </label>
                            }
                        })
                        .collect_view()}
                </div>
                <Show when=urgent>
                    <div class="mt-3 flex gap-2 rounded-xl bg-danger-soft p-3 text-[13px] text-danger" role="alert">
                        <Icon name="AlertTriangle" class="mt-0.5 size-4 shrink-0"/>
                        <span>
                            <b>"These symptoms can be an emergency."</b>
                            " Please go to the nearest emergency room or call 911 now. Our on-call team is also notified."
                        </span>
                    </div>
                </Show>
            </fieldset>
        }
        .into_any(),
        _ => view! {
            <fieldset class="mt-5">
                <legend class="mb-3 text-[14px] font-semibold text-ink">"How will this visit be covered?"</legend>
                <div class="flex flex-wrap gap-2">
                    {INS.iter().map(|(v, label)| { let v = *v; view! { <Chip active=Signal::derive(move || ins.get() == Some(v)) on_click=move || ins.set(Some(v)) label=label/> } }).collect_view()}
                </div>
            </fieldset>
        }
        .into_any(),
    };

    let go_portal = move |_| {
        // Hand off to the isolated portal through sessionStorage, never the URL (no health data in query strings or logs).
        if let Some(r) = rec.get_untracked() {
            if let Ok(s) = serde_json::to_string(&r) {
                super::browser::session_set("mos-intake", &s);
            }
        }
        navigate("/portal", Default::default());
    };

    view! {
        <div
            id="request"
            class="relative scroll-mt-32 rounded-3xl border border-line bg-white/90 p-5 shadow-[var(--shadow-lift)] backdrop-blur sm:p-6"
        >
            <div class="flex items-center justify-between">
                <div>
                    <div class="flex items-center gap-2 text-[12px] font-semibold uppercase tracking-[0.14em] text-sage">
                        <Icon name="Sparkles" class="size-3.5"/>
                        " Smart appointment request"
                    </div>
                    <h2 class="mt-1 font-display text-xl font-semibold text-ink">"Find the right visit in under a minute"</h2>
                </div>
                <Show when=move || step.get() < 4>
                    <span class="text-[12px] font-semibold text-faint">"Step " {move || (step.get() + 1).min(4)} " / 4"</span>
                </Show>
            </div>

            <div class="mt-4 h-1 overflow-hidden rounded-full bg-sand">
                <div
                    class="h-full rounded-full bg-sage transition-[width] duration-500"
                    style:width=move || format!("{}%", crate::domain::util::num(step.get().min(4) as f64 / 4.0 * 100.0))
                ></div>
            </div>
            <p class="mt-3 flex items-start gap-1.5 text-[11.5px] leading-snug text-muted">
                <Icon name="ShieldCheck" class="mt-px size-3.5 shrink-0 text-sage"/>
                <span>
                    "Private by design: your answers stay in this browser. We never ask for your name, contact details or insurance ID on this page; those go only into our encrypted patient portal."
                </span>
            </p>

            {move || {
                let s = step.get();
                if s < 4 {
                    view! {
                        // A fresh element per step replays the fade-up animation (React used key={step}).
                        <div class="animate-fade-up">
                            {fieldset(s)}
                            <div class="mt-6 flex items-center justify-between">
                                <button
                                    type="button"
                                    on:click=move |_| step.update(|s| *s = s.saturating_sub(1))
                                    class=cx(&[
                                        "inline-flex items-center gap-1 text-[13px] font-semibold text-muted hover:text-ink",
                                        if s == 0 { "invisible" } else { "" },
                                    ])
                                >
                                    <Icon name="ArrowLeft" class="size-4"/>
                                    " Back"
                                </button>
                                <button
                                    type="button"
                                    disabled=move || !can_next() || (step.get() == 2 && urgent())
                                    on:click=move |_| step.update(|s| *s += 1)
                                    class="inline-flex items-center gap-1.5 rounded-xl bg-navy px-4 py-2.5 text-[14px] font-semibold text-white transition hover:bg-ocean disabled:opacity-40"
                                >
                                    {if s == 3 { "See my options" } else { "Continue" }}
                                    " "
                                    <Icon name="ArrowRight" class="size-4"/>
                                </button>
                            </div>
                        </div>
                    }
                        .into_any()
                } else {
                    rec.get()
                        .map(|r| {
                            view! {
                                <div class="animate-fade-up mt-5">
                                    <div class="rounded-2xl bg-gradient-to-br from-sky-2 to-sage-soft p-4">
                                        <div class="text-[12px] font-semibold uppercase tracking-wide text-ocean">"Recommended"</div>
                                        <div class="mt-1 font-display text-lg font-semibold text-ink">{r.service.clone()}</div>
                                        <div class="mt-1 text-[14px] text-muted">{r.who.clone()} " · " {r.where_.clone()}</div>
                                        <div class="mt-3 inline-flex items-center gap-2 rounded-lg bg-white px-3 py-1.5 text-[13px] font-semibold text-ink shadow-sm">
                                            <Icon name="CalendarCheck2" class="size-4 text-sage"/>
                                            " Earliest: "
                                            {r.when.clone()}
                                        </div>
                                    </div>
                                    <div class="mt-4">
                                        <div class="text-[13px] font-semibold text-ink">"Bring to your visit"</div>
                                        <ul class="mt-2 grid gap-1.5 text-[13px] text-muted">
                                            {r
                                                .bring
                                                .iter()
                                                .map(|b| {
                                                    view! {
                                                        <li class="flex gap-2">
                                                            <Icon name="CheckCircle2" class="mt-0.5 size-4 shrink-0 text-sage"/>
                                                            " "
                                                            {b.clone()}
                                                        </li>
                                                    }
                                                })
                                                .collect_view()}
                                        </ul>
                                    </div>
                                    <div class="mt-5 flex items-center justify-between">
                                        <button
                                            type="button"
                                            on:click=move |_| step.set(3)
                                            class="inline-flex items-center gap-1 text-[13px] font-semibold text-muted hover:text-ink"
                                        >
                                            <Icon name="ArrowLeft" class="size-4"/>
                                            " Back"
                                        </button>
                                        <button
                                            type="button"
                                            on:click=go_portal.clone()
                                            class="inline-flex items-center gap-1.5 rounded-xl bg-sage px-4 py-2.5 text-[14px] font-semibold text-white transition hover:bg-[#25724f]"
                                        >
                                            <Icon name="Lock" class="size-4"/>
                                            " Continue in secure portal"
                                        </button>
                                    </div>
                                </div>
                            }
                        })
                        .into_any()
                }
            }}
        </div>
    }
}
