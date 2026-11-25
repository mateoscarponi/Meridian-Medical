use crate::components::icons::Icon;
use crate::components::shell::ToolHeader;
use crate::components::ui::*;
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum TaskStatus {
    Done,
    Open,
    Late,
    Draft,
}

struct Task {
    label: &'static str,
    due: &'static str,
    days_left: Option<i32>,
    status: TaskStatus,
    note: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Wc,
    Nf,
}

struct Case {
    id: &'static str,
    kind: Kind,
    patient: &'static str,
    carrier: &'static str,
    claim: &'static str,
    event: &'static str,
    attorney: Option<&'static str>,
    tasks: &'static [Task],
}

use TaskStatus::*;

static CASES: [Case; 4] = [
    Case {
        id: "WC-118",
        kind: Kind::Wc,
        patient: "Jorge Castillo",
        carrier: "State Insurance Fund (NYSIF)",
        claim: "WCB # pending",
        event: "Injured 11/13/2026 · lifting at warehouse · first visit 11/15",
        attorney: None,
        tasks: &[
            Task { label: "Doctor's initial report (C-4)", due: "11/17/2026", days_left: Some(1), status: Draft, note: "Must be filed within 48 hours of first treatment. Draft generated from the 11/15 note; needs provider signature." },
            Task { label: "Causal relationship statement", due: "with C-4", days_left: Some(1), status: Draft, note: "Copilot extracted mechanism of injury and the provider's causation opinion." },
            Task { label: "Treatment plan vs. Medical Treatment Guidelines", due: "before treatment", days_left: None, status: Done, note: "PT 2–3×/wk and NSAIDs fall within the Mid & Low Back MTG; no variance PAR needed." },
        ],
    },
    Case {
        id: "WC-097",
        kind: Kind::Wc,
        patient: "Robert Williams",
        carrier: "NYSIF",
        claim: "WCB G331 8042",
        event: "Injured 09/14/2026 · neck · lifting pallet",
        attorney: Some("Represented: Klein & Partners"),
        tasks: &[
            Task { label: "C-4 initial report", due: "09/16/2026", days_left: None, status: Done, note: "Filed within 48 hours." },
            Task { label: "Progress report", due: "11/20/2026", days_left: Some(4), status: Open, note: "Pre-filled from the last 3 visits; pain scores and work status trended automatically." },
            Task { label: "Cervical ESI (62321)", due: "11/17/2026", days_left: Some(1), status: Done, note: "Within MTG; scheduled. Bill will carry the WCB case number." },
            Task { label: "Carrier IME", due: "12/03/2026", days_left: Some(17), status: Open, note: "Records packet (minimum necessary) prepared for the IME physician." },
        ],
    },
    Case {
        id: "NF-221",
        kind: Kind::Nf,
        patient: "Aisha Rahman",
        carrier: "GEICO (PIP)",
        claim: "NF-77120934",
        event: "Motor vehicle accident 09/20/2026 · cervical strain",
        attorney: None,
        tasks: &[
            Task { label: "NF-2 application (patient)", due: "10/20/2026", days_left: None, status: Done, note: "Received 10/02, within 30 days of the accident." },
            Task { label: "Bill for DOS 09/22 (NF-3)", due: "11/06/2026", days_left: Some(-10), status: Late, note: "Outside the 45-day window: hospital records arrived late. Justification letter drafted and attached." },
            Task { label: "Carrier verification request", due: "11/24/2026", days_left: Some(8), status: Open, note: "Requested: X-ray report & office notes. Packet assembled from the chart; ready to send." },
        ],
    },
    Case {
        id: "NF-230",
        kind: Kind::Nf,
        patient: "Daniel Ortiz",
        carrier: "Progressive (PIP)",
        claim: "NF-88412077",
        event: "Motor vehicle accident 11/02/2026 · lumbar strain",
        attorney: None,
        tasks: &[
            Task { label: "NF-2 application (patient)", due: "12/02/2026", days_left: Some(16), status: Open, note: "Bilingual SMS reminder sent with a pre-filled form link." },
            Task { label: "Bill for DOS 11/09 (NF-3)", due: "12/24/2026", days_left: Some(38), status: Open, note: "Will auto-generate once the visit note is signed." },
        ],
    },
];

fn due_chip(t: &Task) -> AnyView {
    match (t.status, t.days_left) {
        (Done, _) => view! { <Badge tone=Tone::Sage><Icon name="CheckCircle2" class="size-3"/> " Done"</Badge> }.into_any(),
        (Late, d) => view! { <Badge tone=Tone::Danger>{d.unwrap_or(0).abs()} " d late"</Badge> }.into_any(),
        (_, None) => view! { <Badge>"Open"</Badge> }.into_any(),
        (_, Some(d)) => {
            let tone = if d <= 2 { Tone::Danger } else if d <= 7 { Tone::Warn } else { Tone::Ocean };
            view! { <Badge tone=tone><Icon name="AlarmClock" class="size-3"/> " " {d} " d left"</Badge> }.into_any()
        }
    }
}

fn urgent_count() -> usize {
    CASES.iter().flat_map(|c| c.tasks.iter()).filter(|t| t.status != Done && (t.status == Late || t.days_left.unwrap_or(99) <= 2)).count()
}

#[component]
pub fn WcNoFaultPage() -> impl IntoView {
    let filter = RwSignal::new(None::<Kind>);
    let signed = RwSignal::new(Vec::<String>::new());

    view! {
        <div class="mx-auto max-w-7xl">
            <ToolHeader
                eyebrow="Revenue cycle · Workers' comp & no-fault"
                title="WC & No-Fault Desk"
                description="Workers' compensation and no-fault cases carry strict New York filing deadlines. This desk tracks every clock, drafts the forms from the chart, and escalates before a deadline is missed, which is often the difference between getting paid and writing it off."
                tags=&["NY WCB forms", "Medical Treatment Guidelines", "No-fault NF-2 / NF-3", "deadline engine", "minimum-necessary records"]
                dyn_impact=vec![(urgent_count().to_string(), "deadlines ≤ 48 h or late".into()), ("0".into(), "missed C-4s since go-live".into())]
            />

            <div class="mb-5 flex flex-wrap items-center gap-2">
                {[(None, "All cases"), (Some(Kind::Wc), "Workers' comp"), (Some(Kind::Nf), "No-fault")]
                    .into_iter()
                    .map(|(k, l)| {
                        view! {
                            <button
                                on:click=move |_| filter.set(k)
                                class=move || {
                                    cx(&[
                                        "rounded-xl px-3 py-1.5 text-[13px] font-semibold ring-1 ring-inset transition",
                                        if filter.get() == k { "bg-navy text-white ring-navy" } else { "bg-white text-muted ring-line-2 hover:text-ink" },
                                    ])
                                }
                            >
                                {l}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>

            <div class="grid gap-6 lg:grid-cols-2">
                {move || {
                    let f = filter.get();
                    CASES
                        .iter()
                        .filter(|c| f.is_none_or(|k| c.kind == k))
                        .map(|c| {
                            let wc = c.kind == Kind::Wc;
                            view! {
                                <Card class="animate-fade-up">
                                    <div class="flex items-start justify-between gap-3 border-b border-line px-5 py-4">
                                        <div class="flex min-w-0 items-start gap-3">
                                            <div class="mt-0.5 grid size-8 shrink-0 place-items-center rounded-lg bg-sky-2 text-ocean">
                                                {if wc { view! { <Icon name="Briefcase" class="size-4"/> } } else { view! { <Icon name="Car" class="size-4"/> } }}
                                            </div>
                                            <div class="min-w-0">
                                                <h3 class="text-[15px] font-semibold text-ink">
                                                    <span class="flex items-center gap-2">
                                                        {c.patient} " " <span class="font-mono text-[11px] font-normal text-faint">{c.id}</span>
                                                    </span>
                                                </h3>
                                                <p class="mt-0.5 text-[13px] text-muted">{c.carrier} " · " {c.claim}</p>
                                            </div>
                                        </div>
                                        <div class="shrink-0">
                                            <Badge tone={if wc { Tone::Gold } else { Tone::Ocean }}>{if wc { "Workers' comp" } else { "No-fault" }}</Badge>
                                        </div>
                                    </div>
                                    <div class="border-b border-line px-5 py-2.5 text-[12.5px] text-muted">
                                        {c.event}
                                        {c.attorney.map(|a| view! { " · " <Icon name="Scale" class="mb-0.5 inline size-3"/> " " {a} })}
                                    </div>
                                    <ul class="divide-y divide-line">
                                        {c
                                            .tasks
                                            .iter()
                                            .map(|t| {
                                                let key = format!("{}{}", c.id, t.label);
                                                let key2 = key.clone();
                                                let is_signed = move || signed.with(|s| s.contains(&key));
                                                let is_signed2 = is_signed.clone();
                                                let is_draft = t.status == Draft;
                                                view! {
                                                    <li class="px-5 py-3.5">
                                                        <div class="flex items-start justify-between gap-3">
                                                            <div class="min-w-0">
                                                                <div class="text-[13px] font-semibold text-ink">{t.label}</div>
                                                                <div class="text-[11.5px] text-faint">"Due " {t.due}</div>
                                                            </div>
                                                            {move || {
                                                                if is_signed() {
                                                                    view! { <Badge tone=Tone::Sage><Icon name="CheckCircle2" class="size-3"/> " Filed"</Badge> }.into_any()
                                                                } else {
                                                                    due_chip(t)
                                                                }
                                                            }}
                                                        </div>
                                                        <p class="mt-1.5 text-[12.5px] text-muted">{t.note}</p>
                                                        <Show when=move || is_draft && !is_signed2()>
                                                            {
                                                                let key = key2.clone();
                                                                view! {
                                                                    <Button size=BtnSize::Sm class="mt-2" on_click=Callback::new(move |_| signed.update(|s| s.push(key.clone())))>
                                                                        <Icon name="FileSignature" class="size-3.5"/>
                                                                        " Review, sign & file"
                                                                    </Button>
                                                                }
                                                            }
                                                        </Show>
                                                    </li>
                                                }
                                            })
                                            .collect_view()}
                                    </ul>
                                </Card>
                            }
                        })
                        .collect_view()
                }}
            </div>

            <Card class="mt-6">
                <CardHeader title="Deadline rules encoded" subtitle="Configurable per carrier; always confirm current Board and DFS requirements"/>
                <div class="grid gap-px bg-line sm:grid-cols-2 lg:grid-cols-4">
                    {[
                        ("C-4 initial report", "Filed within 48 hours of first treatment of a work injury."),
                        ("MTG compliance", "Care within the Medical Treatment Guidelines is pre-authorized; a variance goes through the WCB Medical Portal (PAR)."),
                        ("NF-2 application", "Patient files with the no-fault carrier within 30 days of the accident."),
                        ("NF-3 billing", "Provider bills within 45 days of service, or attaches a reasonable justification for the delay."),
                    ]
                        .into_iter()
                        .map(|(t, d)| {
                            view! {
                                <div class="bg-white p-4">
                                    <div class="text-[13px] font-semibold text-ink">{t}</div>
                                    <div class="mt-1 text-[12px] text-muted">{d}</div>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </Card>
        </div>
    }
}
