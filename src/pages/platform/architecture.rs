use crate::components::icons::Icon;
use crate::components::shell::ToolHeader;
use crate::components::ui::*;
use leptos::prelude::*;

const LAYERS: [(&str, &str, &[&str]); 5] = [
    ("Channels", "bg-gold-soft text-gold", &["Practice website & smart intake", "Patient portal", "SMS / IVR / voicemail", "Staff console", "Fax & email"]),
    (
        "Applications",
        "bg-sky-2 text-ocean",
        &["Prior Auth Copilot", "Document AI", "Clinical Scribe", "Claim Scrubber", "Scheduling", "Referrals", "AI Agent", "Command Center"],
    ),
    (
        "AI & services",
        "bg-sage-soft text-sage",
        &["LLM gateway (Claude / OpenAI, BAA, ZDR)", "Clinical NLP", "OCR / Document AI", "ML models (no-show, denial risk)", "Rules engine", "Workflow orchestrator"],
    ),
    (
        "Integration",
        "bg-[#ece3f3] text-[#6b4a93]",
        &["FHIR R4 gateway (SMART)", "HL7 v2 MLLP listener", "X12 via clearinghouse", "Payer PA APIs (Da Vinci)", "Fax / SMS providers", "WCB Medical Portal"],
    ),
    ("Data", "bg-sand text-ink", &["Operational Postgres", "FHIR store", "Warehouse (star schema)", "Object storage (raw)", "Feature store", "Immutable audit log"]),
];

const STACK: [(&str, &[&str]); 4] = [
    (
        "Languages",
        &[
            "Rust (this demo: Axum server + Leptos UI compiled to WebAssembly)",
            "TypeScript (Next.js, Node)",
            "Python (FastAPI, pandas, scikit-learn, PyTorch)",
            "Go (high-throughput HL7 listener)",
            "Java (Mirth Connect channels) · C# (.NET vendor SDKs)",
            "SQL",
        ],
    ),
    ("AI / ML", &["Claude & OpenAI APIs", "LangChain-style tool orchestration", "spaCy / transformer NER", "TensorFlow / PyTorch for custom models", "Evals & guardrails"]),
    ("Healthcare standards", &["HL7 v2.5.1", "FHIR R4 · US Core · SMART", "X12 5010 (270/271, 276/277, 278, 837P, 835)", "ICD-10-CM · CPT · LOINC"]),
    ("Cloud & infra", &["AWS (HIPAA-eligible services) or Azure / GCP", "Docker · Terraform · GitHub Actions", "Queues + dead-letter + retries", "OpenTelemetry monitoring"]),
];

const ROADMAP: [(&str, &str, &str, &[&str]); 3] = [
    (
        "Days 1–30",
        "Listen, measure, quick wins",
        "Search",
        &[
            "Shadow front desk, auth, billing and providers at each site",
            "2-week time study → baseline for every KPI",
            "Read-only EMR / clearinghouse access and data map",
            "Ship: pre-visit checklist SMS and fax-intake classification",
        ],
    ),
    (
        "Days 31–60",
        "Revenue cycle automation",
        "Rocket",
        &[
            "Prior Auth Copilot for the top 5 CPTs by volume",
            "Claim scrubber rules from 12 months of 835 denials",
            "Central pipeline + Command Center dashboard",
            "Weekly KPI review with operations",
        ],
    ),
    (
        "Days 61–90",
        "Scale & intelligence",
        "Flag",
        &[
            "No-show model live with waitlist backfill",
            "Clinical scribe pilot with 1–2 providers",
            "AI agent for rescheduling & status questions (EN/ES)",
            "90-day results vs. baseline → next-quarter roadmap",
        ],
    ),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Snip {
    Hl7,
    Ml,
    Llm,
}

fn snippet(s: Snip) -> &'static str {
    match s {
        Snip::Hl7 => r#"// HL7 v2 → FHIR (src/domain/hl7.rs, simplified)
pub fn parse_hl7(input: &str) -> Hl7Message {
    let text = input.replace("\r\n", "\r").replace('\n', "\r");
    let lines: Vec<&str> = text.split('\r').filter(|l| !l.is_empty()).collect();
    let field_sep = lines[0].chars().nth(3).unwrap_or('|');        // MSH-1
    let enc: Vec<char> = lines[0].chars().skip(4).take(4).collect(); // MSH-2
    let segments = lines.iter().map(|line| {
        let mut parts = line.split(field_sep);
        let id = parts.next().unwrap_or_default().to_string();
        let fields = parts.map(|raw| raw.split(enc[1])
            .map(|rep| rep.split(enc[0]).map(unescape).collect()).collect()).collect();
        Hl7Segment { id, fields }
    }).collect();
    Hl7Message { segments, kind: field(&segments[0], 9), control_id: field(&segments[0], 10) }
}"#,
        Snip::Ml => r#"// No-show model (src/bin/train_noshow.rs, excerpt): dependency-free logistic regression
fn train(x: &[[f64; 9]], y: &[f64], lr: f64, epochs: usize, l2: f64) -> (f64, [f64; 9]) {
    let (mut w, mut b) = ([0.0; 9], 0.0);
    let n = x.len() as f64;
    for _ in 0..epochs {
        let (mut gw, mut gb) = ([0.0; 9], 0.0);
        for (xi, yi) in x.iter().zip(y) {
            let err = sigmoid(b + dot(&w, xi)) - yi;
            gb += err;
            for (g, v) in gw.iter_mut().zip(xi) { *g += err * v; }
        }
        b -= lr * gb / n;
        for (wj, gj) in w.iter_mut().zip(gw) { *wj -= lr * (gj / n + l2 * *wj); }
    }
    (b, w) // exported to data/no-show-weights.json (~100× faster than the Python version)
}"#,
        Snip::Llm => r#"// LLM call with on-device fallback (src/server/api.rs, excerpt)
let res = client.post("https://api.anthropic.com/v1/messages")
    .header("x-api-key", key)
    .header("anthropic-version", "2026-10-21")
    .header("anthropic-beta", "server-side-fallback-2026-07-01")
    .json(&json!({
        "model": "claude-opus-5-5",
        "max_tokens": 4000,
        "fallbacks": "default",
        "output_config": { "effort": "low" },
        "system": SYSTEM_SCRIBE,          // documentation-only, no diagnosis
        "messages": [{ "role": "user", "content": redacted_transcript }],
    }))
    .send().await?;
if msg["stop_reason"] == "refusal" { return local_fallback(); }"#,
    }
}

#[component]
pub fn ArchitecturePage() -> impl IntoView {
    let snip = RwSignal::new(Snip::Hl7);
    view! {
        <div class="mx-auto max-w-7xl">
            <ToolHeader
                eyebrow="Trust & build · How it fits together"
                title="Platform Architecture & Rollout"
                description="A modular platform that sits next to the EMR instead of replacing it: standards-based integration, a central data layer, AI services behind guardrails, and a 90-day rollout plan built around measurable outcomes."
                tags=&["modular services", "standards-first", "cloud-agnostic", "secure by default", "measured rollout"]
            />

            <Card>
                <CardHeader
                    title="Reference architecture"
                    icon=|| view! { <Icon name="Layers" class="size-4"/> }
                    subtitle="Every layer is replaceable; standards keep it vendor-neutral"
                />
                <div class="space-y-2.5 p-5">
                    {LAYERS
                        .iter()
                        .map(|(name, tone, items)| {
                            view! {
                                <div class="grid items-stretch gap-2.5 md:grid-cols-[150px_1fr]">
                                    <div class=cx(&["flex items-center rounded-xl px-4 py-3 text-[13px] font-bold", tone])>{*name}</div>
                                    <div class="flex flex-wrap gap-2 rounded-xl border border-line bg-cream/50 p-2.5">
                                        {items
                                            .iter()
                                            .map(|i| view! { <span class="rounded-lg bg-white px-3 py-1.5 text-[12.5px] font-medium text-ink shadow-sm ring-1 ring-line">{*i}</span> })
                                            .collect_view()}
                                    </div>
                                </div>
                            }
                        })
                        .collect_view()}
                    <div class="mt-1 flex items-center gap-2 rounded-xl bg-ink px-4 py-3 text-[12.5px] text-white/80">
                        <Icon name="Lock" class="size-4 text-gold-2"/>
                        " Cross-cutting: SSO/MFA · RBAC · encryption · audit logging · PHI redaction · monitoring & alerting · BAAs"
                    </div>
                </div>
            </Card>

            <div class="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                {[
                    ("99.95%", "uptime target (SLO)", "multi-AZ, health checks, auto-restart"),
                    ("< 200 ms", "p95 API latency", "caching + read replicas for dashboards"),
                    ("0 lost messages", "HL7 & X12 delivery", "durable queues, retries, dead-letter + replay"),
                    ("< 1 h / 15 min", "RTO / RPO", "PITR backups, infra as code, DR runbook"),
                ]
                    .into_iter()
                    .map(|(v, l, d)| {
                        view! {
                            <Card class="p-5">
                                <div class="font-display text-2xl font-semibold text-ink">{v}</div>
                                <div class="text-[12.5px] font-semibold text-ocean">{l}</div>
                                <div class="mt-1 text-[12px] text-muted">{d}</div>
                            </Card>
                        }
                    })
                    .collect_view()}
            </div>

            <div class="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                {STACK
                    .iter()
                    .map(|(area, items)| {
                        view! {
                            <Card class="p-5">
                                <div class="flex items-center gap-2 text-[13px] font-semibold text-ink">
                                    {if *area == "Cloud & infra" {
                                        view! { <Icon name="Cloud" class="size-4 text-ocean"/> }
                                    } else {
                                        view! { <Icon name="Code2" class="size-4 text-ocean"/> }
                                    }}
                                    {*area}
                                </div>
                                <ul class="mt-3 space-y-1.5 text-[12.5px] text-muted">
                                    {items.iter().map(|i| view! { <li>"• " {*i}</li> }).collect_view()}
                                </ul>
                            </Card>
                        }
                    })
                    .collect_view()}
            </div>

            <Card class="mt-6">
                <CardHeader
                    title="First 90 days"
                    subtitle="Start with the people doing the work, then automate what measurably helps"
                    icon=|| view! { <Icon name="Users" class="size-4"/> }
                />
                <div class="grid gap-px bg-line md:grid-cols-3">
                    {ROADMAP
                        .iter()
                        .map(|(phase, title, icon, items)| {
                            view! {
                                <div class="bg-white p-5">
                                    <div class="flex items-center gap-2">
                                        <div class="grid size-8 place-items-center rounded-lg bg-navy text-white">
                                            <Icon name=icon class="size-4"/>
                                        </div>
                                        <div>
                                            <div class="text-[11px] font-bold uppercase tracking-wide text-gold">{*phase}</div>
                                            <div class="text-[14px] font-semibold text-ink">{*title}</div>
                                        </div>
                                    </div>
                                    <ul class="mt-4 space-y-2 text-[12.5px] text-muted">
                                        {items
                                            .iter()
                                            .map(|i| {
                                                view! {
                                                    <li class="flex gap-2">
                                                        <span class="mt-[7px] size-1.5 shrink-0 rounded-full bg-sage"></span>
                                                        {*i}
                                                    </li>
                                                }
                                            })
                                            .collect_view()}
                                    </ul>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </Card>

            <Card class="mt-6">
                <CardHeader
                    title="From the codebase"
                    subtitle="Real excerpts from this demo"
                    icon=|| view! { <Icon name="Code2" class="size-4"/> }
                    right=move || view! { <Segmented value=snip options=vec![(Snip::Hl7, "Rust · HL7"), (Snip::Ml, "Rust · ML"), (Snip::Llm, "LLM API")]/> }
                />
                <div class="p-4">
                    <CodeBlock value=Signal::derive(move || snippet(snip.get()).to_string()) max_h="max-h-96"/>
                </div>
            </Card>
        </div>
    }
}
