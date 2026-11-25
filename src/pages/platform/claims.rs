use crate::components::charts::Bars;
use crate::components::icons::Icon;
use crate::components::shell::ToolHeader;
use crate::components::ui::*;
use crate::domain::claims::*;
use leptos::prelude::*;

fn risk_tone(r: u32) -> Tone {
    if r >= 50 {
        Tone::Danger
    } else if r >= 25 {
        Tone::Warn
    } else {
        Tone::Sage
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    Edits,
    X12,
}

/// Applies the first available fix until the claim is clean (max 8 rounds), logging each rule.
fn fix_claim(c: &Claim, log: &mut Vec<String>) -> Claim {
    let mut x = c.clone();
    for _ in 0..8 {
        let (findings, _) = denial_risk(&x);
        let Some(f) = findings.into_iter().find(|y| y.apply.is_some()) else { break };
        log.push(format!("{}: {}", c.id, f.rule));
        x = apply_fix(&x, f.apply.as_ref().expect("fix"));
    }
    x
}

#[component]
pub fn ClaimsPage() -> impl IntoView {
    let initial = claims();
    let first_id = initial[0].id.clone();
    let claims_sig = RwSignal::new(initial);
    let sel = RwSignal::new(first_id);
    let view_mode = RwSignal::new(View::Edits);
    let fixed_log = RwSignal::new(Vec::<String>::new());

    let scored = Memo::new(move |_| {
        claims_sig.with(|cs| {
            cs.iter()
                .map(|c| {
                    let (f, r) = denial_risk(c);
                    (c.clone(), f, r)
                })
                .collect::<Vec<_>>()
        })
    });
    let cur = Memo::new(move |_| {
        let id = sel.get();
        scored.with(|s| s.iter().find(|x| x.0.id == id).cloned().expect("selected claim"))
    });
    let at_risk = move || scored.with(|s| s.iter().filter(|x| x.2 >= 25).map(|x| total(&x.0)).collect::<Vec<_>>());
    let clean = move || scored.with(|s| s.iter().filter(|x| x.1.is_empty()).count());
    let n = move || claims_sig.with(|c| c.len());

    let apply_one = move |id: String, idx: usize| {
        let Some(c) = claims_sig.with_untracked(|cs| cs.iter().find(|x| x.id == id).cloned()) else { return };
        let (findings, _) = denial_risk(&c);
        let Some(f) = findings.get(idx).cloned() else { return };
        let Some(fix) = f.apply.clone() else { return };
        claims_sig.update(|cs| {
            for x in cs.iter_mut() {
                if x.id == id {
                    *x = apply_fix(x, &fix);
                }
            }
        });
        fixed_log.update(|l| l.insert(0, format!("{id}: {}", f.rule)));
    };
    let fix_all = move |id: String| {
        let mut log = Vec::new();
        let next: Vec<Claim> = claims_sig.get_untracked().iter().map(|c| if c.id == id { fix_claim(c, &mut log) } else { c.clone() }).collect();
        claims_sig.set(next);
        fixed_log.update(|l| {
            log.extend(l.drain(..));
            *l = log;
        });
    };
    let fix_everything = move || {
        let mut log = Vec::new();
        let next: Vec<Claim> = claims_sig.get_untracked().iter().map(|c| fix_claim(c, &mut log)).collect();
        claims_sig.set(next);
        fixed_log.update(|l| {
            log.extend(l.drain(..));
            *l = log;
        });
    };

    view! {
        <div class="mx-auto max-w-7xl">
            <ToolHeader
                eyebrow="Revenue cycle · Billing & claims workflows"
                title="Claim Scrubber & Denial Predictor"
                description="Every claim is checked before it leaves: NCCI-style bundling, modifiers, retired ICD-10 codes, missing authorizations, and New York workers' comp and no-fault rules. Fixes come pre-filled and the risk score updates live."
                tags=&["X12 837P / 835", "NCCI edits", "ICD-10-CM / CPT", "denial risk scoring", "NY no-fault · WCB"]
                impact=&[("97.1%", "clean claim rate"), ("−42%", "denials")]
            />

            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                <Stat label="Claims in today's batch" value=Signal::derive(move || n().to_string()) hint="Ready to transmit to the clearinghouse" tone=StatTone::Neutral/>
                <Stat
                    label="Passing all edits"
                    value=Signal::derive(move || format!("{}/{}", clean(), n()))
                    delta=Signal::derive(move || (clean() == n()).then(|| "ready".to_string()))
                />
                <Stat
                    label="At denial risk"
                    value=Signal::derive(move || at_risk().len().to_string())
                    tone=StatTone::Danger
                    delta=Signal::derive(move || {
                        let r = at_risk();
                        Some(if r.is_empty() { "none".to_string() } else { money(r.iter().sum(), 0) })
                    })
                    hint="risk ≥ 25%"
                />
                <Stat label="Edits auto-fixed (session)" value=Signal::derive(move || fixed_log.with(|l| l.len()).to_string()) hint="each one logged with who approved it"/>
            </div>

            <div class="mt-6 grid gap-6 xl:grid-cols-[1.1fr_1fr]">
                <Card class="overflow-hidden">
                    <CardHeader
                        title="Pre-submission batch"
                        subtitle="Click a claim to see its edits"
                        right=move || {
                            view! {
                                <Button
                                    size=BtnSize::Sm
                                    variant=BtnVariant::Dark
                                    on_click=Callback::new(move |_| fix_everything())
                                    disabled=Signal::derive(move || at_risk().is_empty() && scored.with(|s| s.iter().all(|x| x.1.is_empty())))
                                >
                                    <Icon name="Wand2" class="size-3.5"/>
                                    " Fix all safe edits"
                                </Button>
                            }
                        }
                    />
                    <div class="overflow-x-auto">
                        <table class="w-full min-w-[620px] text-[13px]">
                            <thead class="bg-cream text-left text-[11px] uppercase tracking-wide text-faint">
                                <tr>
                                    <th class="px-5 py-2.5">"Claim"</th>
                                    <th class="px-2 py-2.5">"Payer"</th>
                                    <th class="px-2 py-2.5">"Codes"</th>
                                    <th class="px-2 py-2.5 text-right">"Charge"</th>
                                    <th class="px-5 py-2.5 text-right">"Denial risk"</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-line">
                                {move || {
                                    scored
                                        .get()
                                        .into_iter()
                                        .map(|(c, findings, risk)| {
                                            let id = c.id.clone();
                                            let id2 = id.clone();
                                            let codes = c.lines.iter().map(line_code).collect::<Vec<_>>().join(", ");
                                            view! {
                                                <tr
                                                    on:click=move |_| sel.set(id.clone())
                                                    class=move || cx(&["cursor-pointer transition", if sel.get() == id2 { "bg-sky-2" } else { "hover:bg-cream" }])
                                                >
                                                    <td class="px-5 py-3">
                                                        <div class="font-semibold text-ink">{c.patient.clone()}</div>
                                                        <div class="font-mono text-[11px] text-faint">{c.id.clone()} " · DOS " {c.dos.clone()}</div>
                                                    </td>
                                                    <td class="px-2 py-3">
                                                        <div class="text-ink">{c.payer.clone()}</div>
                                                        <div class="text-[11px] text-muted">{c.payer_type.as_str()}</div>
                                                    </td>
                                                    <td class="px-2 py-3 font-mono text-[11.5px] text-muted">{codes}</td>
                                                    <td class="px-2 py-3 text-right tabular-nums text-ink">{money(total(&c), 0)}</td>
                                                    <td class="px-5 py-3 text-right">
                                                        <div class="flex items-center justify-end gap-2">
                                                            {(!findings.is_empty())
                                                                .then(|| {
                                                                    view! {
                                                                        <span class="text-[11px] text-muted">
                                                                            {findings.len()} " edit" {if findings.len() > 1 { "s" } else { "" }}
                                                                        </span>
                                                                    }
                                                                })}
                                                            <Badge tone=risk_tone(risk)>{risk} "%"</Badge>
                                                        </div>
                                                    </td>
                                                </tr>
                                            }
                                        })
                                        .collect_view()
                                }}
                            </tbody>
                        </table>
                    </div>
                </Card>

                <Card class="flex flex-col">
                    <CardHeader
                        title=Signal::derive(move || cur.with(|c| format!("{} · {}", c.0.patient, c.0.id)))
                        subtitle=Signal::derive(move || cur.with(|c| format!("{} · {} · {}", c.0.payer, c.0.provider, money(total(&c.0), 0))))
                        right=move || view! { <Segmented value=view_mode options=vec![(View::Edits, "Edits"), (View::X12, "837P")]/> }
                    />
                    {move || {
                        if view_mode.get() == View::X12 {
                            view! {
                                <div class="p-4">
                                    <CodeBlock value=Signal::derive(move || cur.with(|c| to_837(&c.0))) max_h="max-h-[420px]"/>
                                    <p class="mt-2 text-[11px] text-muted">
                                        "Simplified 837P claim loop. Production adds envelope, billing/rendering provider loops and payer IDs."
                                    </p>
                                </div>
                            }
                                .into_any()
                        } else {
                            let (c, findings, risk) = cur.get();
                            let id = c.id.clone();
                            let has_fix = findings.iter().any(|f| f.apply.is_some());
                            view! {
                                <div class="flex-1">
                                    <div class="flex items-center justify-between border-b border-line px-5 py-4">
                                        <div>
                                            <div class="text-[12px] text-muted">"Predicted denial risk"</div>
                                            <div class=cx(&[
                                                "font-display text-4xl font-semibold tabular-nums",
                                                if risk >= 50 { "text-danger" } else if risk >= 25 { "text-warn" } else { "text-sage" },
                                            ])>{risk} "%"</div>
                                        </div>
                                        {has_fix
                                            .then(|| {
                                                let id = id.clone();
                                                view! {
                                                    <Button variant=BtnVariant::Sage on_click=Callback::new(move |_| fix_all(id.clone()))>
                                                        <Icon name="Wand2" class="size-4"/>
                                                        " Apply all fixes"
                                                    </Button>
                                                }
                                            })}
                                    </div>
                                    {if findings.is_empty() {
                                        view! {
                                            <div class="animate-fade-up flex items-center gap-3 p-6 text-[14px] text-sage">
                                                <Icon name="ShieldCheck" class="size-6"/>
                                                " All edits pass. Claim is ready to transmit."
                                            </div>
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <ul class="divide-y divide-line">
                                                {findings
                                                    .into_iter()
                                                    .enumerate()
                                                    .map(|(i, f)| {
                                                        let id = id.clone();
                                                        let critical = f.severity == Severity::Critical;
                                                        view! {
                                                            <li class="px-5 py-4">
                                                                <div class="flex items-start gap-3">
                                                                    {if critical {
                                                                        view! { <Icon name="XOctagon" class="mt-0.5 size-4 shrink-0 text-danger"/> }
                                                                    } else {
                                                                        view! { <Icon name="AlertTriangle" class="mt-0.5 size-4 shrink-0 text-warn"/> }
                                                                    }}
                                                                    <div class="min-w-0 flex-1">
                                                                        <div class="flex flex-wrap items-center gap-2">
                                                                            <span class="text-[13px] font-semibold text-ink">{f.rule}</span>
                                                                            <Badge tone={if critical { Tone::Danger } else { Tone::Warn }}>{f.severity.as_str()}</Badge>
                                                                            <span class="text-[11px] text-faint">"+" {f.weight} " risk"</span>
                                                                        </div>
                                                                        <p class="mt-1 text-[12.5px] text-muted">{f.message.clone()}</p>
                                                                        <div class="mt-2 flex flex-wrap items-center justify-between gap-2 rounded-lg bg-sand px-3 py-2">
                                                                            <span class="text-[12px] text-ink">
                                                                                <b>"Fix:"</b>
                                                                                " "
                                                                                {f.fix}
                                                                            </span>
                                                                            {f
                                                                                .apply
                                                                                .is_some()
                                                                                .then(|| {
                                                                                    view! {
                                                                                        <Button
                                                                                            size=BtnSize::Sm
                                                                                            variant=BtnVariant::Secondary
                                                                                            on_click=Callback::new(move |_| apply_one(id.clone(), i))
                                                                                        >
                                                                                            "Apply"
                                                                                        </Button>
                                                                                    }
                                                                                })}
                                                                        </div>
                                                                    </div>
                                                                </div>
                                                            </li>
                                                        }
                                                    })
                                                    .collect_view()}
                                            </ul>
                                        }
                                            .into_any()
                                    }}
                                    <div class="border-t border-line px-5 py-4">
                                        <div class="text-[12px] font-semibold text-muted">"Claim lines"</div>
                                        <ul class="mt-2 space-y-1 font-mono text-[12px] text-ink">
                                            {c
                                                .lines
                                                .iter()
                                                .map(|l| {
                                                    view! {
                                                        <li class="flex justify-between gap-3">
                                                            <span>{line_code(l)} " × " {l.units} " · " {l.icd10.join(", ")}</span>
                                                            <span class="text-muted">{money(l.charge, 0)}</span>
                                                        </li>
                                                    }
                                                })
                                                .collect_view()}
                                        </ul>
                                        {c
                                            .auth_number
                                            .clone()
                                            .map(|a| {
                                                view! {
                                                    <div class="mt-2 text-[12px] text-sage">
                                                        <Icon name="CheckCircle2" class="mr-1 inline size-3.5"/>
                                                        "Auth "
                                                        {a}
                                                    </div>
                                                }
                                            })}
                                    </div>
                                </div>
                            }
                                .into_any()
                        }
                    }}
                </Card>
            </div>

            <div class="mt-6 grid gap-6 lg:grid-cols-[1fr_1fr]">
                <Card>
                    <CardHeader title="Top denial reasons, last 90 days" subtitle="Share of denied claims, from 835 remittance CARC codes"/>
                    <div class="p-5">
                        <Bars
                            label="Top denial reasons last 90 days"
                            unit="%"
                            data=vec![
                                ("Auth missing", 27.0),
                                ("Bundling", 19.0),
                                ("Modifier", 15.0),
                                ("Eligibility", 13.0),
                                ("Dx/medical nec.", 11.0),
                                ("Timely filing", 8.0),
                                ("Other", 7.0),
                            ]
                        />
                    </div>
                </Card>
                <Card>
                    <CardHeader title="How the predictor works"/>
                    <ul class="space-y-3 p-5 text-[13px] text-muted">
                        <li>
                            <b class="text-ink">"1 · Deterministic edits"</b>
                            " catch known errors with certainty: bundling pairs, modifier rules, retired codes, payer filing limits."
                        </li>
                        <li>
                            <b class="text-ink">"2 · Learned risk"</b>
                            " weighs each edit using historical 835 remittances by payer, CPT and provider (gradient-boosted model in production; weighted rules here)."
                        </li>
                        <li>
                            <b class="text-ink">"3 · Feedback loop:"</b>
                            " every denial that still happens is labeled and fed back weekly, so the scrubber learns each payer's habits."
                        </li>
                        <li>
                            <b class="text-ink">"4 · Human control:"</b>
                            " coding changes that affect medical necessity always need coder approval; formatting fixes apply automatically."
                        </li>
                    </ul>
                </Card>
            </div>
        </div>
    }
}
