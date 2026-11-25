use crate::components::icons::Icon;
use crate::components::shell::ToolHeader;
use crate::components::ui::*;
use crate::domain::util::{locale, num, to_fixed};
use leptos::prelude::*;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cloud {
    Aws,
    Azure,
    Gcp,
}

struct StageDef {
    title: &'static str,
    detail: &'static str,
    svc: [&'static str; 3],
    count: f64,
}

static STAGES: [StageDef; 5] = [
    StageDef { title: "Sources", detail: "EMR (FHIR + HL7), PM/billing, clearinghouse 835/837, fax/Document AI, phone/SMS, PT system", svc: ["9 connectors", "9 connectors", "9 connectors"], count: 48210.0 },
    StageDef { title: "Ingest", detail: "MLLP listener, FHIR $export (bulk), SFTP/X12 drops, webhooks → durable queue", svc: ["ECS listener · SQS · S3 (raw, KMS)", "Container Apps · Service Bus · ADLS", "Cloud Run · Pub/Sub · GCS"], count: 48210.0 },
    StageDef { title: "Normalize", detail: "Map to FHIR R4, code systems (ICD-10, CPT, LOINC), patient matching, dedupe, data-quality checks", svc: ["Python workers · HealthLake", "Functions · Health Data Services", "Dataflow · Healthcare API"], count: 47962.0 },
    StageDef { title: "Store", detail: "Operational FHIR store + analytics warehouse (star schema), immutable audit log", svc: ["RDS Postgres · Redshift", "Azure SQL · Synapse", "Cloud SQL · BigQuery"], count: 47962.0 },
    StageDef { title: "Serve", detail: "APIs for apps & agent, dashboards, ML features, scheduled reports", svc: ["API Gateway · QuickSight · SageMaker", "APIM · Power BI · Azure ML", "Apigee · Looker · Vertex AI"], count: 47962.0 },
];

const DQ: [(&str, f64, u32); 5] = [
    ("Required fields present (PID-3, PID-5, PID-7)", 99.6, 182),
    ("Valid ICD-10-CM codes (no retired M54.5)", 99.1, 41),
    ("Appointment ↔ encounter reconciliation", 98.4, 77),
    ("Duplicate patients resolved by MPI", 99.8, 12),
    ("835 payments matched to 837 claims", 97.9, 64),
];

const TERMS: [(&str, &str, &str, &str); 5] = [
    ("eCW local", "LBP w/ sciatica R", "ICD-10-CM", "M54.41 · Lumbago with sciatica, right side"),
    ("athena order", "MRI L-SPINE WO", "CPT", "72148 · MRI lumbar spine w/o contrast"),
    ("PT system", "Pain NRS", "LOINC", "72514-3 · Pain severity 0-10 (reported)"),
    ("Intake form", "Smoker? No", "LOINC", "72166-2 · Tobacco smoking status = Never"),
    ("Fax OCR", "TFESI L4-5 RT", "CPT", "64483-RT · Transforaminal ESI, lumbar, single level"),
];

const SQL: &str = "-- Median authorization turnaround by payer, last 90 days
SELECT p.payer_name,
       COUNT(*)                                             AS requests,
       PERCENTILE_CONT(0.5) WITHIN GROUP
         (ORDER BY a.decided_at - a.submitted_at)           AS median_turnaround,
       AVG(CASE WHEN a.decision = 'denied' THEN 1 ELSE 0 END) AS denial_rate
FROM   fact_authorization a
JOIN   dim_payer p ON p.payer_key = a.payer_key
WHERE  a.submitted_at >= CURRENT_DATE - INTERVAL '90 days'
GROUP  BY p.payer_name
ORDER  BY median_turnaround DESC;";

const SQL_ROWS: [(&str, u32, &str, &str); 5] = [
    ("UnitedHealthcare", 214, "3.4 d", "7.9%"),
    ("Aetna", 163, "3.1 d", "6.1%"),
    ("Empire BCBS", 188, "2.7 d", "5.3%"),
    ("Humana MA", 97, "1.9 d", "4.1%"),
    ("Cigna", 72, "2.2 d", "3.8%"),
];

/// Strips accents (NFD + drop combining marks for Latin letters), lowercases and keeps [a-z0-9].
fn fold(c: char) -> char {
    match c {
        'à'..='å' | 'À'..='Å' | 'ā' | 'ă' | 'ą' | 'Ā' | 'Ă' | 'Ą' => 'a',
        'ç' | 'Ç' | 'ć' | 'ĉ' | 'ċ' | 'č' | 'Ć' | 'Ĉ' | 'Ċ' | 'Č' => 'c',
        'è'..='ë' | 'È'..='Ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' | 'Ē' | 'Ĕ' | 'Ė' | 'Ę' | 'Ě' => 'e',
        'ì'..='ï' | 'Ì'..='Ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'Ĩ' | 'Ī' | 'Ĭ' | 'Į' | 'İ' => 'i',
        'ñ' | 'Ñ' | 'ń' | 'ņ' | 'ň' | 'Ń' | 'Ņ' | 'Ň' => 'n',
        'ò'..='ö' | 'Ò'..='Ö' | 'ō' | 'ŏ' | 'ő' | 'Ō' | 'Ŏ' | 'Ő' => 'o',
        'ù'..='ü' | 'Ù'..='Ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' | 'Ũ' | 'Ū' | 'Ŭ' | 'Ů' | 'Ű' | 'Ų' => 'u',
        'ý' | 'ÿ' | 'Ý' | 'Ÿ' => 'y',
        'ś' | 'ŝ' | 'ş' | 'š' | 'Ś' | 'Ŝ' | 'Ş' | 'Š' => 's',
        'ź' | 'ż' | 'ž' | 'Ź' | 'Ż' | 'Ž' => 'z',
        'ğ' | 'ĝ' | 'ġ' | 'ģ' | 'Ğ' | 'Ĝ' | 'Ġ' | 'Ģ' => 'g',
        other => other,
    }
}

fn norm(s: &str) -> String {
    s.chars().map(fold).flat_map(char::to_lowercase).filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit()).collect()
}

fn jaro_winkler(a: &str, b: &str) -> f64 {
    let (a, b) = (norm(a), norm(b));
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    if a == b {
        return 1.0;
    }
    let range = (a.len().max(b.len()) / 2).saturating_sub(1);
    let mut am = vec![false; a.len()];
    let mut bm = vec![false; b.len()];
    let mut m = 0usize;
    for i in 0..a.len() {
        for j in i.saturating_sub(range)..(i + range + 1).min(b.len()) {
            if bm[j] || a[i] != b[j] {
                continue;
            }
            am[i] = true;
            bm[j] = true;
            m += 1;
            break;
        }
    }
    if m == 0 {
        return 0.0;
    }
    let (mut t, mut k) = (0usize, 0usize);
    for i in 0..a.len() {
        if !am[i] {
            continue;
        }
        while !bm[k] {
            k += 1;
        }
        if a[i] != b[k] {
            t += 1;
        }
        k += 1;
    }
    let mf = m as f64;
    let jaro = (mf / a.len() as f64 + mf / b.len() as f64 + (mf - t as f64 / 2.0) / mf) / 3.0;
    let mut p = 0;
    while p < 4 && p < a.len() && p < b.len() && a[p] == b[p] {
        p += 1;
    }
    jaro + p as f64 * 0.1 * (1.0 - jaro)
}

const FIELDS: [(&str, f64); 5] = [("First name", 0.2), ("Last name", 0.25), ("Date of birth", 0.3), ("Phone", 0.15), ("ZIP", 0.1)];

/// first, last, dob, phone, zip
type Rec = [String; 5];

fn split_name(s: &str) -> Vec<&str> {
    s.split(|c: char| c == '-' || c.is_whitespace()).collect()
}

fn field_score(k: usize, a: &Rec, b: &Rec) -> f64 {
    let mut s = match k {
        2 => {
            if a[2] == b[2] {
                1.0
            } else if a[2].chars().take(7).collect::<String>() == b[2].chars().take(7).collect::<String>() {
                0.4
            } else {
                0.0
            }
        }
        3 => {
            let (x, y) = (norm(&a[3]), norm(&b[3]));
            let last10 = |s: &str| s.chars().rev().take(10).collect::<Vec<_>>();
            if last10(&x) == last10(&y) { 1.0 } else { 0.0 }
        }
        _ => jaro_winkler(&a[k], &b[k]),
    };
    if k == 1 && s < 1.0 {
        let (na, nb) = (norm(&a[1]), norm(&b[1]));
        let compound = split_name(&a[1]).into_iter().chain(split_name(&b[1])).any(|p| {
            let np = norm(p);
            !np.is_empty() && na.starts_with(&np) && nb.starts_with(&np)
        });
        if compound {
            s = s.max(0.95);
        }
    }
    s
}

#[component]
pub fn PipelinePage() -> impl IntoView {
    let cloud = RwSignal::new(Cloud::Aws);
    let running = RwSignal::new(-1i32);
    let ran = RwSignal::new(false);
    let a = RwSignal::new(["María", "González-Ruiz", "1968-04-12", "516-555-0191", "11550"].map(String::from));
    let b = RwSignal::new(["Maria", "Gonzalez", "1968-04-12", "(516) 555-0191", "11550"].map(String::from));

    let per = Memo::new(move |_| {
        let (ra, rb) = (a.get(), b.get());
        (0..5).map(|k| field_score(k, &ra, &rb)).collect::<Vec<f64>>()
    });
    let score = move || per.with(|p| p.iter().zip(FIELDS.iter()).map(|(s, f)| s * f.1).sum::<f64>());
    let decision = move || {
        let s = score();
        if s >= 0.92 {
            0
        } else if s >= 0.78 {
            1
        } else {
            2
        }
    };

    let run = move || {
        ran.set(false);
        for i in 0..STAGES.len() {
            set_timeout(move || running.set(i as i32), Duration::from_millis(i as u64 * 650));
        }
        set_timeout(
            move || {
                running.set(-1);
                ran.set(true);
            },
            Duration::from_millis(STAGES.len() as u64 * 650),
        );
    };
    let ci = move || match cloud.get() {
        Cloud::Aws => 0,
        Cloud::Azure => 1,
        Cloud::Gcp => 2,
    };

    view! {
        <div class="mx-auto max-w-7xl">
            <ToolHeader
                eyebrow="Data & integration · Centralized data pipelines"
                title="Data Pipeline & Patient Matching"
                description="Data from every system lands in one normalized, de-duplicated model, mapped to FHIR, coded with standard terminologies, quality-checked and loaded into a warehouse that powers dashboards, models and the agent. It runs on any major cloud."
                tags=&["ETL / ELT", "FHIR R4 normalization", "MPI (Jaro-Winkler)", "data quality", "star schema", "AWS · Azure · GCP"]
                impact=&[("99.3%", "records passing DQ"), ("< 5 min", "source → dashboard")]
            />

            <Card>
                <CardHeader
                    title="Nightly + streaming pipeline"
                    subtitle=Signal::derive(move || {
                        if ran.get() {
                            "Last run: just now · 48,210 records in · 248 rejected to quarantine with reasons".to_string()
                        } else {
                            "HL7 streams in real time; bulk sources run nightly".to_string()
                        }
                    })
                    icon=|| view! { <Icon name="Database" class="size-4"/> }
                    right=move || {
                        view! {
                            <div class="flex items-center gap-2">
                                <Segmented value=cloud options=vec![(Cloud::Aws, "AWS"), (Cloud::Azure, "Azure"), (Cloud::Gcp, "GCP")]/>
                                <Button size=BtnSize::Sm on_click=Callback::new(move |_| run()) disabled=Signal::derive(move || running.get() >= 0)>
                                    {move || {
                                        if running.get() >= 0 {
                                            view! { <Icon name="Loader2" class="size-3.5 animate-spin"/> }.into_any()
                                        } else {
                                            view! { <Icon name="Play" class="size-3.5"/> }.into_any()
                                        }
                                    }}
                                    " Run"
                                </Button>
                            </div>
                        }
                    }
                />
                <div class="grid gap-3 p-5 md:grid-cols-5">
                    {STAGES
                        .iter()
                        .enumerate()
                        .map(|(i, s)| {
                            let active = move || running.get() == i as i32;
                            let done = move || ran.get() || running.get() > i as i32;
                            view! {
                                <div class="relative">
                                    <div class=move || {
                                        cx(&[
                                            "h-full rounded-xl border p-4 transition-all duration-500",
                                            if active() {
                                                "border-ocean bg-sky-2 shadow-[var(--shadow-lift)]"
                                            } else if done() {
                                                "border-sage/40 bg-sage-soft/40"
                                            } else {
                                                "border-line bg-cream/50"
                                            },
                                        ])
                                    }>
                                        <div class="flex items-center justify-between">
                                            <span class="text-[11px] font-bold uppercase tracking-wide text-faint">{i + 1} " · " {s.title}</span>
                                            {move || {
                                                if active() {
                                                    Some(view! { <Icon name="Loader2" class="size-3.5 animate-spin text-ocean"/> }.into_any())
                                                } else if done() {
                                                    Some(view! { <Icon name="CheckCircle2" class="size-3.5 text-sage"/> }.into_any())
                                                } else {
                                                    None
                                                }
                                            }}
                                        </div>
                                        <div class="mt-2 font-mono text-[11.5px] font-semibold text-ocean">{move || s.svc[ci()]}</div>
                                        <p class="mt-2 text-[12px] leading-relaxed text-muted">{s.detail}</p>
                                        <div class="mt-3 font-display text-lg font-semibold tabular-nums text-ink">
                                            {move || if done() || active() { locale(s.count) } else { "—".to_string() }}
                                        </div>
                                    </div>
                                    {(i < STAGES.len() - 1)
                                        .then(|| {
                                            view! {
                                                <div class=move || {
                                                    cx(&["absolute -right-2.5 top-1/2 z-10 hidden h-0.5 w-2 md:block", if done() { "bg-sage" } else { "bg-line-2" }])
                                                }></div>
                                            }
                                        })}
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </Card>

            <div class="mt-6 grid gap-6 xl:grid-cols-[1.2fr_1fr]">
                <Card>
                    <CardHeader
                        title="Patient matching (MPI)"
                        subtitle="Same person from two systems? Edit either record to see the score change"
                        icon=|| view! { <Icon name="GitMerge" class="size-4"/> }
                        right=move || {
                            view! {
                                <Badge tone=Signal::derive(move || match decision() {
                                    0 => Tone::Sage,
                                    1 => Tone::Warn,
                                    _ => Tone::Neutral,
                                })>
                                    {move || match decision() {
                                        0 => "Auto-link",
                                        1 => "Human review",
                                        _ => "Different people",
                                    }}
                                </Badge>
                            }
                        }
                    />
                    <div class="p-5">
                        <div class="grid grid-cols-[110px_1fr_1fr_64px] items-center gap-2 text-[12px]">
                            <div></div>
                            <div class="font-semibold text-muted">"EMR (eClinicalWorks)"</div>
                            <div class="font-semibold text-muted">"Referral fax (OCR)"</div>
                            <div class="text-right font-semibold text-muted">"Sim."</div>
                            {FIELDS
                                .iter()
                                .enumerate()
                                .map(|(k, (label, _))| {
                                    let s = move || per.with(|p| p[k]);
                                    let input_cls = cx(&[INPUT_CLS, "!py-1.5 !text-[12.5px]"]);
                                    let input_cls2 = input_cls.clone();
                                    view! {
                                        <div class="font-semibold text-ink">{*label}</div>
                                        <input
                                            value=move || a.with(|r| r[k].clone())
                                            prop:value=move || a.with(|r| r[k].clone())
                                            on:input=move |ev| {
                                                let v = event_target_value(&ev);
                                                a.update(|r| r[k] = v);
                                            }
                                            class=input_cls
                                        />
                                        <input
                                            value=move || b.with(|r| r[k].clone())
                                            prop:value=move || b.with(|r| r[k].clone())
                                            on:input=move |ev| {
                                                let v = event_target_value(&ev);
                                                b.update(|r| r[k] = v);
                                            }
                                            class=input_cls2
                                        />
                                        <div class=move || {
                                            let v = s();
                                            cx(&["text-right font-mono text-[12px] font-semibold", if v >= 0.95 { "text-sage" } else if v >= 0.8 { "text-warn" } else { "text-danger" }])
                                        }>{move || to_fixed(s(), 2)}</div>
                                    }
                                })
                                .collect_view()}
                        </div>
                        <div class="mt-5 flex items-center justify-between rounded-xl bg-sand p-4">
                            <div class="text-[13px] text-muted">
                                "Weighted match score "
                                <span class="text-[11px]">"(auto-link ≥ 0.92 · review ≥ 0.78)"</span>
                            </div>
                            <div class=move || {
                                cx(&[
                                    "font-display text-3xl font-semibold tabular-nums",
                                    match decision() {
                                        0 => "text-sage",
                                        1 => "text-warn",
                                        _ => "text-muted",
                                    },
                                ])
                            }>{move || to_fixed(score(), 3)}</div>
                        </div>
                        <p class="mt-3 text-[12px] text-muted">
                            "Accent-insensitive Jaro-Winkler on names, compound-surname handling, exact DOB and phone matching. Borderline pairs go to a review queue, never an automatic merge."
                        </p>
                    </div>
                </Card>

                <Card>
                    <CardHeader
                        title="Data-quality checks"
                        subtitle="Failing records are quarantined with a reason, not dropped"
                        icon=|| view! { <Icon name="TriangleAlert" class="size-4"/> }
                    />
                    <ul class="divide-y divide-line">
                        {DQ
                            .iter()
                            .map(|(check, pass, issues)| {
                                view! {
                                    <li class="px-5 py-3">
                                        <div class="flex items-center justify-between gap-3 text-[13px]">
                                            <span class="text-ink">{*check}</span>
                                            <span class="font-semibold tabular-nums text-ink">{num(*pass)} "%"</span>
                                        </div>
                                        <div class="mt-1.5 flex items-center gap-3">
                                            <div class="h-1.5 flex-1 rounded-full bg-sand">
                                                <div class="h-1.5 rounded-full bg-sage" style:width=format!("{}%", num(*pass))></div>
                                            </div>
                                            <span class="w-24 text-right text-[11px] text-muted">{*issues} " quarantined"</span>
                                        </div>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                </Card>
            </div>

            <div class="mt-6 grid gap-6 xl:grid-cols-2">
                <Card class="overflow-hidden">
                    <CardHeader title="Terminology normalization" subtitle="Local codes and free text → standard code systems"/>
                    <div class="overflow-x-auto">
                        <table class="w-full min-w-[520px] text-[12.5px]">
                            <thead class="bg-cream text-left text-[11px] uppercase tracking-wide text-faint">
                                <tr>
                                    <th class="px-5 py-2.5">"Source"</th>
                                    <th class="px-2 py-2.5">"Raw value"</th>
                                    <th class="px-2 py-2.5">"System"</th>
                                    <th class="px-5 py-2.5">"Standard code"</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-line">
                                {TERMS
                                    .iter()
                                    .map(|t| {
                                        view! {
                                            <tr>
                                                <td class="px-5 py-2.5 text-muted">{t.0}</td>
                                                <td class="px-2 py-2.5 font-mono text-ink">{t.1}</td>
                                                <td class="px-2 py-2.5"><Badge tone=Tone::Ocean>{t.2}</Badge></td>
                                                <td class="px-5 py-2.5 text-ink">{t.3}</td>
                                            </tr>
                                        }
                                    })
                                    .collect_view()}
                            </tbody>
                        </table>
                    </div>
                </Card>
                <Card class="overflow-hidden">
                    <CardHeader
                        title="Warehouse query"
                        subtitle="Star schema: fact_authorization, fact_claim, fact_appointment + dimensions"
                        icon=|| view! { <Icon name="Table2" class="size-4"/> }
                    />
                    <div class="p-4">
                        <CodeBlock value=SQL max_h="max-h-64"/>
                    </div>
                    <table class="w-full text-[12.5px]">
                        <thead class="bg-cream text-left text-[11px] uppercase tracking-wide text-faint">
                            <tr>
                                <th class="px-5 py-2">"payer_name"</th>
                                <th class="px-2 py-2 text-right">"requests"</th>
                                <th class="px-2 py-2 text-right">"median"</th>
                                <th class="px-5 py-2 text-right">"denial_rate"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-line">
                            {SQL_ROWS
                                .iter()
                                .map(|r| {
                                    view! {
                                        <tr>
                                            <td class="px-5 py-2 text-ink">{r.0}</td>
                                            <td class="px-2 py-2 text-right tabular-nums text-muted">{r.1}</td>
                                            <td class="px-2 py-2 text-right tabular-nums text-ink">{r.2}</td>
                                            <td class="px-5 py-2 text-right tabular-nums text-muted">{r.3}</td>
                                        </tr>
                                    }
                                })
                                .collect_view()}
                        </tbody>
                    </table>
                </Card>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mpi_matches_default_pair() {
        let a = ["María", "González-Ruiz", "1968-04-12", "516-555-0191", "11550"].map(String::from);
        let b = ["Maria", "Gonzalez", "1968-04-12", "(516) 555-0191", "11550"].map(String::from);
        let score: f64 = (0..5).map(|k| field_score(k, &a, &b) * FIELDS[k].1).sum();
        assert!(score >= 0.92, "{score}");
        assert_eq!(jaro_winkler("Martha", "Marhta"), 0.9611111111111111);
    }
}
