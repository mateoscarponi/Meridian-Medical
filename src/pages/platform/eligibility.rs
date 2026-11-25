use crate::components::icons::Icon;
use crate::components::shell::ToolHeader;
use crate::components::ui::*;
use leptos::prelude::*;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Res {
    Active,
    Action,
    Inactive,
}

struct Row {
    time: &'static str,
    patient: &'static str,
    payer: &'static str,
    member_id: &'static str,
    visit: &'static str,
    result: Res,
    copay: &'static str,
    deductible: &'static str,
    flags: &'static [&'static str],
    fix: Option<&'static str>,
}

static ROWS: [Row; 10] = [
    Row { time: "8:15", patient: "Maria Gonzalez", payer: "Fidelis Care (Medicaid)", member_id: "FC88310442", visit: "Follow-up", result: Res::Active, copay: "$0", deductible: "n/a", flags: &["Established patient: Medicaid accepted"], fix: None },
    Row { time: "8:45", patient: "Harold Becker", payer: "Medicare Part B", member_id: "1EG4-TE5-MK72", visit: "Follow-up", result: Res::Active, copay: "20% coinsurance", deductible: "Met", flags: &[], fix: None },
    Row { time: "9:30", patient: "Priya Natarajan", payer: "Cigna Open Access", member_id: "U7730 18842", visit: "New consult", result: Res::Action, copay: "$50 specialist", deductible: "$1,140 remaining", flags: &["Scoliosis X-ray (72082) billed with visit: patient responsibility est. $96"], fix: Some("Good-faith estimate texted; collect at check-in") },
    Row { time: "10:00", patient: "Aisha Rahman", payer: "GEICO No-Fault", member_id: "NF-77120934", visit: "Follow-up", result: Res::Active, copay: "$0", deductible: "n/a", flags: &["No-fault claim open · NF-3 deadline tracked"], fix: None },
    Row { time: "10:45", patient: "Carmen Diaz", payer: "Empire BCBS", member_id: "YRP 451207733", visit: "New consult", result: Res::Active, copay: "$40 specialist", deductible: "$310 remaining", flags: &[], fix: None },
    Row { time: "11:30", patient: "Kevin Murphy", payer: "Oxford (UHC)", member_id: "OXF 33019276", visit: "Follow-up", result: Res::Inactive, copay: "—", deductible: "—", flags: &["Coverage terminated 10/31/2026 (271 EB*6)", "Possible new employer plan"], fix: Some("Patient texted for new card photo; slot held 24 h") },
    Row { time: "1:00", patient: "Robert Williams", payer: "NYSIF (Workers' Comp)", member_id: "WCB G3318042", visit: "Procedure (ESI)", result: Res::Active, copay: "$0", deductible: "n/a", flags: &["Within MTG → pre-authorized", "WCB case # on file"], fix: None },
    Row { time: "2:15", patient: "Thomas Russo", payer: "UnitedHealthcare", member_id: "UHC 90418827", visit: "Pre-op", result: Res::Action, copay: "$40 specialist", deductible: "$0 remaining", flags: &["Surgery auth pending (PA-24813)", "Out-of-pocket max reached in 2026"], fix: Some("Surgery date held; auth chaser polling hourly") },
    Row { time: "3:00", patient: "James O'Connor", payer: "Aetna PPO", member_id: "W219-55810", visit: "Follow-up", result: Res::Active, copay: "$35 specialist", deductible: "$0 remaining", flags: &[], fix: None },
    Row { time: "3:45", patient: "Linda Chen", payer: "Humana Medicare Advantage", member_id: "H5619-0042", visit: "Follow-up", result: Res::Active, copay: "$25 specialist", deductible: "n/a", flags: &["ESI auth HUM-AUTH-8841207 valid to 02/14/2027"], fix: None },
];

const SAMPLE_271: &str = "ISA*00*          *00*          *ZZ*CLEARINGHOUSE  *ZZ*MERIDIANORTHO  *261116*0612*^*00501*000004117*0*P*:~
ST*271*0001*005010X279A1~
BHT*0022*11*ELG-0011730*20261116*0612~
NM1*PR*2*OXFORD HEALTH PLANS*****PI*06111~
NM1*IL*1*MURPHY*KEVIN****MI*OXF33019276~
DTP*346*D8*20260101~
DTP*347*D8*20261031~
EB*6**30~                         <- 6 = Inactive · 30 = Health benefit plan coverage
MSG*COVERAGE TERMINATED~
SE*9*0001~";

#[derive(Clone, Copy, PartialEq, Eq)]
enum St {
    Idle,
    Running,
    Done,
}

#[component]
pub fn EligibilityPage() -> impl IntoView {
    let state = RwSignal::new(St::Idle);
    let count = RwSignal::new(0usize);
    let resolved = RwSignal::new(Vec::<&'static str>::new());

    let run = move || {
        state.set(St::Running);
        count.set(0);
        for i in 0..ROWS.len() {
            set_timeout(
                move || {
                    count.set(i + 1);
                    if i == ROWS.len() - 1 {
                        state.set(St::Done);
                    }
                },
                Duration::from_millis(180 * (i as u64 + 1)),
            );
        }
    };
    let done = move |i: usize| state.get() == St::Done || count.get() > i;
    let actions = ROWS.iter().filter(|r| r.result != Res::Active).count();

    view! {
        <div class="mx-auto max-w-7xl">
            <ToolHeader
                eyebrow="Revenue cycle · Insurance verification"
                title="Eligibility & Benefits"
                description="Every appointment for tomorrow is verified overnight via X12 270/271, so front-desk staff start the day with exceptions only: terminated plans, patient balances to estimate, and authorizations still pending, each with the next step already taken."
                tags=&["X12 270 / 271", "batch verification", "good-faith estimates", "auth cross-check", "exceptions-only workflow"]
                impact=&[("−18 h", "phone verification / wk"), ("−61%", "eligibility denials")]
            />

            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                <Stat label="Appointments tomorrow" value=ROWS.len().to_string() tone=StatTone::Neutral hint="Tue, Nov 17 · all locations"/>
                <Stat
                    label="Verified active"
                    value=Signal::derive(move || {
                        if state.get() == St::Idle {
                            "—".to_string()
                        } else {
                            ROWS.iter().enumerate().filter(|(i, r)| r.result == Res::Active && done(*i)).count().to_string()
                        }
                    })
                />
                <Stat
                    label="Need attention"
                    value=Signal::derive(move || if state.get() == St::Idle { "—".to_string() } else { actions.to_string() })
                    tone=StatTone::Danger
                    hint="already actioned automatically"
                />
                <Stat
                    label="Run time"
                    value=Signal::derive(move || if state.get() == St::Done { "1.8 s".to_string() } else { "—".to_string() })
                    tone=StatTone::Neutral
                    hint="vs. ~6 min per patient by phone/portal"
                />
            </div>

            <Card class="mt-6 overflow-hidden">
                <CardHeader
                    title="Tomorrow's schedule"
                    subtitle=Signal::derive(move || {
                        if state.get() == St::Done { "Verified at 6:12 AM · next run in 24 h".to_string() } else { "Run the overnight batch on demand".to_string() }
                    })
                    icon=|| view! { <Icon name="BadgeCheck" class="size-4"/> }
                    right=move || {
                        view! {
                            <Button on_click=Callback::new(move |_| run()) disabled=Signal::derive(move || state.get() == St::Running)>
                                {move || {
                                    if state.get() == St::Running {
                                        view! { <Icon name="Loader2" class="size-4 animate-spin"/> }.into_any()
                                    } else {
                                        view! { <Icon name="Play" class="size-4"/> }.into_any()
                                    }
                                }}
                                " Verify all"
                            </Button>
                        }
                    }
                />
                <div class="overflow-x-auto">
                    <table class="w-full min-w-[860px] text-[13px]">
                        <thead class="bg-cream text-left text-[11px] uppercase tracking-wide text-faint">
                            <tr>
                                <th class="px-5 py-2.5">"Time"</th>
                                <th class="px-2 py-2.5">"Patient"</th>
                                <th class="px-2 py-2.5">"Payer"</th>
                                <th class="px-2 py-2.5">"Status"</th>
                                <th class="px-2 py-2.5">"Copay"</th>
                                <th class="px-2 py-2.5">"Deductible"</th>
                                <th class="px-5 py-2.5">"Notes & next step"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-line">
                            {ROWS
                                .iter()
                                .enumerate()
                                .map(|(i, r)| {
                                    let result = r.result;
                                    view! {
                                        <tr class=move || {
                                            cx(&[
                                                "transition-colors",
                                                if done(i) && result == Res::Inactive { "bg-danger-soft/40" } else { "" },
                                                if done(i) && result == Res::Action { "bg-warn-soft/40" } else { "" },
                                            ])
                                        }>
                                            <td class="px-5 py-3 font-mono text-[12px] text-muted">{r.time}</td>
                                            <td class="px-2 py-3">
                                                <div class="font-semibold text-ink">{r.patient}</div>
                                                <div class="text-[11.5px] text-muted">{r.visit}</div>
                                            </td>
                                            <td class="px-2 py-3">
                                                <div class="text-ink">{r.payer}</div>
                                                <div class="font-mono text-[11px] text-faint">{r.member_id}</div>
                                            </td>
                                            <td class="px-2 py-3">
                                                {move || {
                                                    if !done(i) {
                                                        if state.get() == St::Running {
                                                            view! { <Icon name="Loader2" class="size-4 animate-spin text-ocean"/> }.into_any()
                                                        } else {
                                                            view! { <span class="text-faint">"—"</span> }.into_any()
                                                        }
                                                    } else {
                                                        match result {
                                                            Res::Active => view! { <Badge tone=Tone::Sage><Icon name="CheckCircle2" class="size-3"/> " Active"</Badge> }.into_any(),
                                                            Res::Action => view! { <Badge tone=Tone::Warn><Icon name="AlertTriangle" class="size-3"/> " Action"</Badge> }.into_any(),
                                                            Res::Inactive => view! { <Badge tone=Tone::Danger><Icon name="XCircle" class="size-3"/> " Inactive"</Badge> }.into_any(),
                                                        }
                                                    }
                                                }}
                                            </td>
                                            <td class="px-2 py-3 text-ink">{move || if done(i) { r.copay } else { "" }}</td>
                                            <td class="px-2 py-3 text-muted">{move || if done(i) { r.deductible } else { "" }}</td>
                                            <td class="px-5 py-3">
                                                <Show when=move || done(i)>
                                                    <div class="space-y-1">
                                                        {r.flags.iter().map(|f| view! { <div class="text-[12px] text-muted">"• " {*f}</div> }).collect_view()}
                                                        {r
                                                            .fix
                                                            .map(|fix| {
                                                                let patient = r.patient;
                                                                view! {
                                                                    <div class="flex items-center gap-2">
                                                                        <span class="text-[12px] font-semibold text-ocean">"→ " {fix}</span>
                                                                        {move || {
                                                                            if !resolved.with(|x| x.contains(&patient)) {
                                                                                view! {
                                                                                    <button
                                                                                        on:click=move |_| resolved.update(|x| x.push(patient))
                                                                                        class="rounded-md px-1.5 py-0.5 text-[11px] font-semibold text-sage hover:bg-sage-soft"
                                                                                    >
                                                                                        "Mark resolved"
                                                                                    </button>
                                                                                }
                                                                                    .into_any()
                                                                            } else {
                                                                                view! { <Badge tone=Tone::Sage>"Resolved"</Badge> }.into_any()
                                                                            }
                                                                        }}
                                                                    </div>
                                                                }
                                                            })}
                                                    </div>
                                                </Show>
                                            </td>
                                        </tr>
                                    }
                                })
                                .collect_view()}
                        </tbody>
                    </table>
                </div>
            </Card>

            <div class="mt-6 grid gap-6 lg:grid-cols-2">
                <Card>
                    <CardHeader
                        title="Raw 271 response (inactive coverage)"
                        subtitle="What the clearinghouse returned for Kevin Murphy"
                        icon=|| view! { <Icon name="ShieldQuestion" class="size-4"/> }
                    />
                    <div class="p-4">
                        <CodeBlock value=SAMPLE_271 max_h="max-h-72"/>
                    </div>
                </Card>
                <Card>
                    <CardHeader title="Rules applied after each 271"/>
                    <ul class="space-y-2.5 p-5 text-[13px] text-muted">
                        <li>"• " <b class="text-ink">"Inactive or terminated plan:"</b> " text the patient for a new card photo, hold the slot 24 h, then release it to the waitlist."</li>
                        <li>"• " <b class="text-ink">"Patient balance expected:"</b> " send a good-faith estimate from the deductible and coinsurance left."</li>
                        <li>"• " <b class="text-ink">"Medicaid new patient:"</b> " flag against practice policy (established patients and hospital follow-ups only)."</li>
                        <li>"• " <b class="text-ink">"Service needs auth:"</b> " cross-check Prior Auth Copilot; no valid auth means no procedure slot."</li>
                        <li>"• " <b class="text-ink">"Workers' comp / no-fault:"</b> " confirm case or claim number and carrier instead of a commercial 270."</li>
                    </ul>
                </Card>
            </div>
        </div>
    }
}
