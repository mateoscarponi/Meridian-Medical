use crate::components::icons::Icon;
use crate::components::shell::ToolHeader;
use crate::components::ui::*;
use crate::domain::noshow::*;
use crate::domain::util::{js_round, num};
use leptos::prelude::*;
use std::time::Duration;

struct Appt {
    time: &'static str,
    patient: &'static str,
    provider: &'static str,
    location: &'static str,
    visit: &'static str,
    payer: &'static str,
    f: NoShowFeatures,
}

#[allow(clippy::too_many_arguments)]
const fn feat(lead: f64, prior: f64, new_patient: bool, confirmed: bool, medicaid: bool, nf_wc: bool, dist: f64, mon_fri: bool, early: bool) -> NoShowFeatures {
    NoShowFeatures {
        lead_time_days: lead,
        prior_no_shows: prior,
        new_patient,
        reminder_confirmed: confirmed,
        medicaid,
        no_fault_or_wc: nf_wc,
        distance_miles: dist,
        monday_or_friday: mon_fri,
        early_morning: early,
    }
}

static APPTS: [Appt; 10] = [
    Appt { time: "8:15", patient: "Maria Gonzalez", provider: "Dr. Okafor", location: "Garden City", visit: "Follow-up", payer: "Medicaid", f: feat(34.0, 2.0, false, false, true, false, 9.0, false, true) },
    Appt { time: "8:45", patient: "Harold Becker", provider: "D. Park, PA-C", location: "Huntington", visit: "Follow-up", payer: "Medicare", f: feat(14.0, 0.0, false, true, false, false, 4.0, false, true) },
    Appt { time: "9:30", patient: "Priya Natarajan", provider: "Dr. Hale", location: "Huntington", visit: "New consult", payer: "Cigna", f: feat(21.0, 0.0, true, false, false, false, 18.0, false, false) },
    Appt { time: "10:00", patient: "Aisha Rahman", provider: "Dr. Ruiz", location: "Smithtown", visit: "Follow-up", payer: "No-Fault", f: feat(7.0, 1.0, false, true, false, true, 2.0, false, false) },
    Appt { time: "10:45", patient: "Carmen Diaz", provider: "D. Park, PA-C", location: "Garden City", visit: "New consult", payer: "Empire BCBS", f: feat(4.0, 0.0, true, false, false, false, 3.0, false, false) },
    Appt { time: "11:30", patient: "Kevin Murphy", provider: "D. Park, PA-C", location: "Garden City", visit: "Follow-up", payer: "Oxford", f: feat(48.0, 3.0, false, false, false, false, 22.0, false, false) },
    Appt { time: "1:00", patient: "Robert Williams", provider: "Dr. Okafor", location: "Garden City", visit: "Procedure (ESI)", payer: "Workers' Comp", f: feat(10.0, 0.0, false, true, false, true, 16.0, false, false) },
    Appt { time: "2:15", patient: "Thomas Russo", provider: "Dr. Hale", location: "Hicksville", visit: "Pre-op", payer: "UnitedHealthcare", f: feat(12.0, 0.0, false, true, false, false, 3.0, false, false) },
    Appt { time: "3:00", patient: "James O'Connor", provider: "Dr. Okafor", location: "Garden City", visit: "Follow-up", payer: "Aetna", f: feat(26.0, 1.0, false, false, false, false, 8.0, false, false) },
    Appt { time: "3:45", patient: "Linda Chen", provider: "Dr. Hale", location: "Smithtown", visit: "Follow-up", payer: "Humana MA", f: feat(30.0, 0.0, false, false, false, false, 6.0, false, false) },
];

const OPEN_SLOTS: [(&str, &str, &str); 4] = [
    ("Thu 2:00 PM · Garden City · Dr. Okafor", "Ana Pereira", "Waitlisted 9 d · same visit type · 3.1 mi · auth approved"),
    ("Thu 2:30 PM · Garden City · Dr. Okafor", "Luis Moreno", "Asked for earlier date · ES speaker · Medicare"),
    ("Thu 3:15 PM · Garden City · D. Park, PA-C", "Grace Kim", "New patient referral 2 d old · 1.8 mi"),
    ("Thu 3:45 PM · Garden City · D. Park, PA-C", "Daniel Shah", "Post-op check pulled forward · confirmed by SMS"),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum BoolKey {
    ReminderConfirmed,
    NewPatient,
    Medicaid,
    NoFaultOrWc,
    MondayOrFriday,
    EarlyMorning,
}

fn get_bool(f: &NoShowFeatures, k: BoolKey) -> bool {
    match k {
        BoolKey::ReminderConfirmed => f.reminder_confirmed,
        BoolKey::NewPatient => f.new_patient,
        BoolKey::Medicaid => f.medicaid,
        BoolKey::NoFaultOrWc => f.no_fault_or_wc,
        BoolKey::MondayOrFriday => f.monday_or_friday,
        BoolKey::EarlyMorning => f.early_morning,
    }
}
fn toggle(f: &mut NoShowFeatures, k: BoolKey) {
    let v = match k {
        BoolKey::ReminderConfirmed => &mut f.reminder_confirmed,
        BoolKey::NewPatient => &mut f.new_patient,
        BoolKey::Medicaid => &mut f.medicaid,
        BoolKey::NoFaultOrWc => &mut f.no_fault_or_wc,
        BoolKey::MondayOrFriday => &mut f.monday_or_friday,
        BoolKey::EarlyMorning => &mut f.early_morning,
    };
    *v = !*v;
}

const BOOL_FEATURES: [(BoolKey, &str); 6] = [
    (BoolKey::ReminderConfirmed, "Confirmed reminder"),
    (BoolKey::NewPatient, "New patient"),
    (BoolKey::Medicaid, "Medicaid"),
    (BoolKey::NoFaultOrWc, "No-fault / WC"),
    (BoolKey::MondayOrFriday, "Mon / Fri"),
    (BoolKey::EarlyMorning, "Before 9 AM"),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Offer {
    Sent,
    Accepted,
}

fn tone_of(t: RiskTone) -> Tone {
    match t {
        RiskTone::Danger => Tone::Danger,
        RiskTone::Warn => Tone::Warn,
        RiskTone::Good => Tone::Sage,
    }
}

fn pct(p: f64) -> String {
    num(js_round(p * 100.0))
}

#[component]
pub fn SchedulingPage() -> impl IntoView {
    let scored: Vec<Score> = APPTS.iter().map(|a| score_no_show(&a.f)).collect();
    let probs: Vec<f64> = scored.iter().map(|s| s.probability).collect();
    let contribs: Vec<Vec<Contribution>> = scored.into_iter().map(|s| s.contributions).collect();
    let probs = StoredValue::new(probs);
    let contribs = StoredValue::new(contribs);
    let sel = RwSignal::new(0usize);
    let sim = RwSignal::new(APPTS[5].f);
    let offers = RwSignal::new([None::<Offer>; 4]);
    let acted = RwSignal::new(Vec::<usize>::new());
    let expected: f64 = probs.with_value(|p| p.iter().sum());
    let sim_p = move || score_no_show(&sim.get()).probability;
    let cur_p = move || probs.with_value(|p| p[sel.get()]);
    let is_acted = move || acted.with(|a| a.contains(&sel.get()));
    let act = move || {
        let s = sel.get_untracked();
        acted.update(|a| a.push(s));
    };

    let send_offers = move || {
        for i in 0..OPEN_SLOTS.len() {
            offers.update(|o| o[i] = Some(Offer::Sent));
            set_timeout(
                move || offers.update(|o| o[i] = Some(if i == 2 { Offer::Sent } else { Offer::Accepted })),
                Duration::from_millis(900 + i as u64 * 500),
            );
        }
    };

    view! {
        <div class="mx-auto max-w-7xl">
            <ToolHeader
                eyebrow="Front office · Scheduling & predictive analytics"
                title="Smart Scheduling"
                description="A logistic-regression no-show model (trained in Rust, scored live here) ranks every appointment, explains why, and recommends the cheapest intervention that works. Open slots are backfilled from the waitlist automatically."
                dyn_tags=vec![format!("model: {}", MODEL.model), format!("AUC {}", num(MODEL.test_auc))]
                tags=&["explainable features", "waitlist matching", "two-way SMS"]
                impact=&[("13.4→8.6%", "no-show rate"), ("+38", "visits recovered / wk")]
            />

            <div class="grid gap-6 xl:grid-cols-[1.25fr_1fr]">
                <Card class="overflow-hidden">
                    <CardHeader
                        title="Tuesday, Nov 17 · all locations"
                        subtitle=format!("{} appointments · expected no-shows: {:.1}", APPTS.len(), expected)
                        icon=|| view! { <Icon name="CalendarClock" class="size-4"/> }
                    />
                    <ul class="divide-y divide-line">
                        {APPTS
                            .iter()
                            .enumerate()
                            .map(|(i, a)| {
                                let p = probs.with_value(|p| p[i]);
                                let (label, tone) = risk_band(p);
                                let bar = match tone {
                                    RiskTone::Danger => "bg-danger",
                                    RiskTone::Warn => "bg-warn",
                                    RiskTone::Good => "bg-sage",
                                };
                                view! {
                                    <li>
                                        <button
                                            on:click=move |_| sel.set(i)
                                            class=move || {
                                                cx(&[
                                                    "grid w-full grid-cols-[52px_1fr_auto] items-center gap-3 px-5 py-3 text-left transition sm:grid-cols-[52px_1fr_140px_auto]",
                                                    if sel.get() == i { "bg-sky-2" } else { "hover:bg-cream" },
                                                ])
                                            }
                                        >
                                            <span class="font-mono text-[12px] text-muted">{a.time}</span>
                                            <span class="min-w-0">
                                                <span class="block truncate text-[13.5px] font-semibold text-ink">
                                                    {a.patient}
                                                    " "
                                                    <Show when=move || acted.with(|x| x.contains(&i))>
                                                        <Icon name="CheckCircle2" class="ml-1 inline size-3.5 text-sage"/>
                                                    </Show>
                                                </span>
                                                <span class="block truncate text-[12px] text-muted">{a.visit} " · " {a.provider} " · " {a.location}</span>
                                            </span>
                                            <span class="hidden items-center gap-2 sm:flex">
                                                <span class="h-1.5 flex-1 rounded-full bg-sand">
                                                    <span class=cx(&["block h-1.5 rounded-full", bar]) style:width=format!("{}%", num((p * 200.0).min(100.0)))></span>
                                                </span>
                                                <span class="w-9 text-right text-[12px] font-semibold tabular-nums text-ink">{pct(p)} "%"</span>
                                            </span>
                                            <Badge tone=tone_of(tone)>{label}</Badge>
                                        </button>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                </Card>

                <div class="space-y-6">
                    <Card>
                        <CardHeader
                            title=Signal::derive(move || format!("{} · {}% no-show risk", APPTS[sel.get()].patient, pct(cur_p())))
                            subtitle=Signal::derive(move || {
                                let a = &APPTS[sel.get()];
                                format!("{} · {} · {}", a.time, a.visit, a.payer)
                            })
                        />
                        <div class="p-5">
                            <div class="text-[12px] font-semibold text-muted">"Why: contribution to log-odds"</div>
                            <div class="mt-3 space-y-2">
                                {move || {
                                    contribs
                                        .with_value(|c| c[sel.get()].clone())
                                        .into_iter()
                                        .filter(|c| c.value.abs() > 0.01)
                                        .take(6)
                                        .map(|c| {
                                            let w = (c.value.abs() * 60.0).min(100.0);
                                            view! {
                                                <div class="grid grid-cols-[150px_1fr] items-center gap-3 text-[12px]">
                                                    <span class="truncate text-ink">{c.label}</span>
                                                    <div class="relative h-4">
                                                        <div class="absolute inset-y-0 left-1/2 w-px bg-line-2"></div>
                                                        <div
                                                            class=cx(&["absolute inset-y-[3px] rounded-[3px]", if c.value > 0.0 { "left-1/2 bg-danger/80" } else { "right-1/2 bg-sage" }])
                                                            style:width=format!("{}%", num(w / 2.0))
                                                        ></div>
                                                    </div>
                                                </div>
                                            }
                                        })
                                        .collect_view()
                                }}
                                <div class="flex justify-between text-[10px] text-faint">
                                    <span>"← lowers risk"</span>
                                    <span>"raises risk →"</span>
                                </div>
                            </div>
                            <div class="mt-4 rounded-xl bg-sand p-3 text-[13px] text-ink">
                                <b>"Recommended:"</b>
                                " "
                                {move || intervention(cur_p(), &APPTS[sel.get()].f)}
                            </div>
                            <div class="mt-3 flex gap-2">
                                <Button size=BtnSize::Sm on_click=Callback::new(move |_| act()) disabled=Signal::derive(is_acted)>
                                    <Icon name="MessageSquareText" class="size-3.5"/>
                                    " "
                                    {move || if is_acted() { "Sent" } else { "Send two-way SMS" }}
                                </Button>
                                <Button size=BtnSize::Sm variant=BtnVariant::Secondary on_click=Callback::new(move |_| act()) disabled=Signal::derive(is_acted)>
                                    <Icon name="Phone" class="size-3.5"/>
                                    " Add to call list"
                                </Button>
                            </div>
                        </div>
                    </Card>

                    <Card>
                        <CardHeader
                            title="What-if simulator"
                            subtitle="Change the inputs and the model re-scores instantly"
                            icon=|| view! { <Icon name="FlaskConical" class="size-4"/> }
                            right=move || {
                                view! {
                                    <div class="text-right">
                                        <div class=move || {
                                            let p = sim_p();
                                            cx(&[
                                                "font-display text-2xl font-semibold tabular-nums",
                                                if p >= 0.3 { "text-danger" } else if p >= 0.15 { "text-warn" } else { "text-sage" },
                                            ])
                                        }>{move || pct(sim_p())} "%"</div>
                                        <div class="text-[11px] text-muted">"risk"</div>
                                    </div>
                                }
                            }
                        />
                        <div class="space-y-4 p-5">
                            {[(0usize, "Lead time", 0.0, 90.0, "d"), (1, "Prior no-shows", 0.0, 6.0, ""), (2, "Distance", 0.0, 45.0, " mi")]
                                .into_iter()
                                .map(|(k, label, min, max, unit)| {
                                    let get = move || sim.with(|s| match k { 0 => s.lead_time_days, 1 => s.prior_no_shows, _ => s.distance_miles });
                                    view! {
                                        <label class="block">
                                            <div class="flex justify-between text-[12px]">
                                                <span class="font-semibold text-muted">{label}</span>
                                                <span class="font-mono text-ink">{move || num(get())} {unit}</span>
                                            </div>
                                            <input
                                                type="range"
                                                min=num(min)
                                                max=num(max)
                                                value=move || num(get())
                                                prop:value=move || num(get())
                                                on:input=move |ev| {
                                                    let v: f64 = event_target_value(&ev).parse().unwrap_or(0.0);
                                                    sim.update(|s| match k {
                                                        0 => s.lead_time_days = v,
                                                        1 => s.prior_no_shows = v,
                                                        _ => s.distance_miles = v,
                                                    });
                                                }
                                                class="mt-1.5 w-full accent-[#1f5c8f]"
                                            />
                                        </label>
                                    }
                                })
                                .collect_view()}
                            <div class="flex flex-wrap gap-2">
                                {BOOL_FEATURES
                                    .iter()
                                    .map(|(k, label)| {
                                        let k = *k;
                                        view! {
                                            <button
                                                on:click=move |_| sim.update(|s| toggle(s, k))
                                                class=move || {
                                                    cx(&[
                                                        "rounded-lg px-2.5 py-1 text-[12px] font-semibold ring-1 ring-inset transition",
                                                        if sim.with(|s| get_bool(s, k)) { "bg-navy text-white ring-navy" } else { "bg-white text-muted ring-line-2" },
                                                    ])
                                                }
                                            >
                                                {*label}
                                            </button>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        </div>
                    </Card>
                </div>
            </div>

            <div class="mt-6 grid gap-6 lg:grid-cols-[1.25fr_1fr]">
                <Card>
                    <CardHeader
                        title="Waitlist backfill · Thursday"
                        subtitle="Open slots matched to waitlisted patients by visit type, provider, insurance, language and distance"
                        icon=|| view! { <Icon name="Users" class="size-4"/> }
                        right=move || {
                            view! {
                                <Button
                                    size=BtnSize::Sm
                                    variant=BtnVariant::Sage
                                    on_click=Callback::new(move |_| send_offers())
                                    disabled=Signal::derive(move || offers.with(|o| o.iter().any(|x| x.is_some())))
                                >
                                    <Icon name="Send" class="size-3.5"/>
                                    " Text offers"
                                </Button>
                            }
                        }
                    />
                    <ul class="divide-y divide-line">
                        {OPEN_SLOTS
                            .iter()
                            .enumerate()
                            .map(|(i, (slot, matched, why))| {
                                view! {
                                    <li class="flex flex-wrap items-center justify-between gap-3 px-5 py-3">
                                        <div>
                                            <div class="text-[13px] font-semibold text-ink">{*slot}</div>
                                            <div class="text-[12px] text-muted">
                                                <b class="text-ink">{*matched}</b>
                                                " · "
                                                {*why}
                                            </div>
                                        </div>
                                        {move || match offers.with(|o| o[i]) {
                                            Some(Offer::Accepted) => view! { <Badge tone=Tone::Sage><Icon name="CheckCircle2" class="size-3"/> " Booked"</Badge> }.into_any(),
                                            Some(Offer::Sent) => view! { <Badge tone=Tone::Ocean>"Offer sent · waiting"</Badge> }.into_any(),
                                            None => view! { <Badge>"Open"</Badge> }.into_any(),
                                        }}
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                </Card>
                <Card>
                    <CardHeader title="Model card" subtitle="Transparent by design"/>
                    <dl class="grid grid-cols-2 gap-x-4 gap-y-3 p-5 text-[12.5px]">
                        <dt class="text-muted">"Algorithm"</dt>
                        <dd class="text-ink">"Logistic regression (L2)"</dd>
                        <dt class="text-muted">"Training data"</dt>
                        <dd class="text-ink">{MODEL.trained_on.clone()}</dd>
                        <dt class="text-muted">"Test AUC"</dt>
                        <dd class="font-semibold text-ink">{num(MODEL.test_auc)}</dd>
                        <dt class="text-muted">"Base rate"</dt>
                        <dd class="text-ink">{format!("{:.1}%", MODEL.base_no_show_rate * 100.0)}</dd>
                        <dt class="text-muted">"Source"</dt>
                        <dd class="font-mono text-[11.5px] text-ink">"src/bin/train_noshow.rs"</dd>
                        <dt class="text-muted">"Production plan"</dt>
                        <dd class="text-ink">"Retrain monthly on de-identified EMR history; monitor drift and fairness by payer and language"</dd>
                    </dl>
                </Card>
            </div>
        </div>
    }
}
