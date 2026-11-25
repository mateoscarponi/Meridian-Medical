use crate::components::highlight::HighlightedText;
use crate::components::icons::Icon;
use crate::components::shell::ToolHeader;
use crate::components::ui::*;
use crate::domain::nlp::*;
use crate::domain::util::{js_round, json_num, num, round2};
use leptos::prelude::*;
use serde_json::{json, Value};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Sample {
    Lumbar,
    Urgent,
    Cervical,
}

fn sample_text(s: Sample) -> &'static str {
    match s {
        Sample::Lumbar => SAMPLE_NOTE,
        Sample::Urgent => "62-year-old female 3 weeks after L4-L5 microdiscectomy calls with new saddle anesthesia and urinary retention since this morning, with progressive weakness in both legs. Reports low back pain 8/10. No fever. Taking gabapentin and acetaminophen.",
        Sample::Cervical => "Patient with neck pain and pain radiating into the left arm for 4 months, numbness in the thumb. Completed 12 weeks of physical therapy and 8 weeks of gabapentin. MRI of the cervical spine shows a left foraminal disc herniation at C5-C6 with nerve root compression. No myelopathy. Considering ACDF.",
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Summary,
    Json,
}

#[derive(Clone, PartialEq)]
struct Analysis {
    entities: Vec<Entity>,
    active: Vec<String>,
    negated: Vec<String>,
    icd: Vec<IcdSuggestion>,
    weeks: f64,
}

fn analyze(text: &str) -> Analysis {
    let entities = extract_entities(text);
    let rf = red_flags(&entities);
    let icd = suggest_icd10(text, &entities);
    let (weeks, _) = conservative_care_weeks(text);
    Analysis {
        active: rf.active.iter().map(|e| e.text.clone()).collect(),
        negated: rf.negated.iter().map(|e| e.text.clone()).collect(),
        entities,
        icd,
        weeks,
    }
}

fn by_type(a: &Analysis, t: EntityType) -> Vec<String> {
    a.entities.iter().filter(|e| e.kind == t && !e.negated).map(|e| e.text.clone()).collect()
}

fn uniq(v: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for s in v {
        if !out.contains(&s) {
            out.push(s);
        }
    }
    out
}

fn structured(a: &Analysis) -> Value {
    json!({
        "symptoms": by_type(a, EntityType::Symptom),
        "anatomy": uniq(by_type(a, EntityType::Anatomy)),
        "imaging_findings": by_type(a, EntityType::Imaging),
        "medications": by_type(a, EntityType::Medication),
        "conservative_care": { "modalities": by_type(a, EntityType::Therapy), "documented_weeks": json_num(a.weeks) },
        "procedures_planned_or_prior": by_type(a, EntityType::Procedure),
        "red_flags": { "present": a.active, "explicitly_denied": a.negated },
        "icd10_suggestions": a.icd.iter().map(|i| json!({ "code": i.code, "display": i.display, "confidence": json_num(round2(i.confidence)) })).collect::<Vec<_>>(),
    })
}

#[component]
fn Row(label: &'static str, items: Vec<String>) -> impl IntoView {
    view! {
        <div class="grid grid-cols-[140px_1fr] gap-3 px-5 py-3 text-[13px]">
            <div class="font-semibold text-muted">{label}</div>
            <div class="flex flex-wrap gap-1.5">
                {if items.is_empty() {
                    view! { <span class="text-faint">"—"</span> }.into_any()
                } else {
                    items.into_iter().map(|t| view! { <span class="rounded-md bg-sand px-2 py-0.5 text-ink">{t}</span> }).collect_view().into_any()
                }}
            </div>
        </div>
    }
}

#[component]
pub fn ChartNlpPage() -> impl IntoView {
    let sample = RwSignal::new(Sample::Lumbar);
    let text = RwSignal::new(SAMPLE_NOTE.to_string());
    let hidden = RwSignal::new(Vec::<EntityType>::new());
    let view_mode = RwSignal::new(ViewMode::Summary);
    let a = Memo::new(move |_| text.with(|t| analyze(t)));

    // Picking a sample loads its text (the original did this in the Segmented onChange).
    Effect::new(move |prev: Option<Sample>| {
        let s = sample.get();
        if prev.is_some_and(|p| p != s) {
            text.set(sample_text(s).to_string());
        }
        s
    });

    view! {
        <div class="mx-auto max-w-7xl">
            {move || {
                let (n, w) = a.with(|a| (a.entities.len(), a.weeks));
                view! {
                    <ToolHeader
                        eyebrow="Clinical AI · NLP-based chart analysis"
                        title="Chart NLP"
                        description="Turns free-text notes into structured data: symptoms, levels, imaging findings, conservative-care duration, red flags (with negation), and ICD-10 suggestions. It runs entirely in the browser, so no PHI leaves the device. Edit the note and watch it update."
                        tags=&["named-entity recognition", "NegEx negation", "duration normalization", "ICD-10-CM", "red-flag triage"]
                        dyn_impact=vec![(n.to_string(), "entities found".into()), (format!("{} wk", num(w)), "conservative care".into())]
                    />
                }
            }}

            <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
                <Segmented
                    value=sample
                    options=vec![(Sample::Lumbar, "Lumbar radiculopathy"), (Sample::Urgent, "Post-op red flags"), (Sample::Cervical, "Cervical / ACDF")]
                />
                <button
                    on:click=move |_| text.set(sample_text(sample.get_untracked()).to_string())
                    class="inline-flex items-center gap-1.5 text-[13px] font-semibold text-muted hover:text-ink"
                >
                    <Icon name="RotateCcw" class="size-3.5"/>
                    " Reset text"
                </button>
            </div>

            <Show when=move || a.with(|a| !a.active.is_empty())>
                <div class="animate-fade-up mb-5 flex items-start gap-3 rounded-2xl border border-danger/30 bg-danger-soft p-4 text-danger" role="alert">
                    <Icon name="AlertOctagon" class="mt-0.5 size-5 shrink-0"/>
                    <div>
                        <div class="text-[15px] font-semibold">"Red flags present: possible cauda equina / progressive neurologic deficit"</div>
                        <div class="mt-0.5 text-[13px]">
                            "Found: "
                            {move || a.with(|a| a.active.join(", "))}
                            ". Routed to the on-call surgeon with a callback within 15 minutes; patient instructed to go to the ER."
                        </div>
                    </div>
                </div>
            </Show>

            <div class="grid gap-6 xl:grid-cols-2">
                <Card>
                    <CardHeader title="Clinical note" subtitle="Type or paste any spine / orthopedic note" icon=|| view! { <Icon name="ScanText" class="size-4"/> }/>
                    <textarea
                        prop:value=move || text.get()
                        on:input=move |ev| text.set(event_target_value(&ev))
                        class="scroll-thin h-44 w-full resize-y border-b border-line bg-cream/40 p-5 text-[14px] leading-relaxed text-ink outline-none"
                    >
                        {SAMPLE_NOTE}
                    </textarea>
                    <div class="p-5 text-[14px] leading-7 text-ink">
                        <HighlightedText text=text entities=Signal::derive(move || a.with(|a| a.entities.clone())) hidden=hidden/>
                    </div>
                    <div class="flex flex-wrap gap-1.5 border-t border-line px-5 py-3">
                        {EntityType::ALL
                            .iter()
                            .map(|t| {
                                let t = *t;
                                let off = move || hidden.with(|h| h.contains(&t));
                                view! {
                                    <button
                                        on:click=move |_| hidden.update(|h| if h.contains(&t) { h.retain(|x| *x != t) } else { h.push(t) })
                                        class=move || cx(&["inline-flex items-center gap-1.5 rounded-lg px-2 py-1 text-[11px] font-semibold ring-1 ring-inset ring-line transition", if off() { "opacity-40" } else { "bg-white" }])
                                    >
                                        <span class=cx(&["ent size-2.5 rounded-sm !p-0", t.swatch()])></span>
                                        {t.label()}
                                        " "
                                        <span class="text-faint">{move || a.with(|a| a.entities.iter().filter(|e| e.kind == t).count())}</span>
                                    </button>
                                }
                            })
                            .collect_view()}
                        <span class="ml-auto self-center text-[11px] text-faint">
                            <span class="line-through">"struck"</span>
                            " = negated"
                        </span>
                    </div>
                </Card>

                <div class="space-y-6">
                    <Card>
                        <CardHeader
                            title="Structured output"
                            subtitle="What gets written back to the chart / data pipeline"
                            icon=|| view! { <Icon name="Braces" class="size-4"/> }
                            right=move || view! { <Segmented value=view_mode options=vec![(ViewMode::Summary, "Summary"), (ViewMode::Json, "JSON")]/> }
                        />
                        {move || {
                            let an = a.get();
                            if view_mode.get() == ViewMode::Json {
                                let s = pretty_json(&structured(&an));
                                view! {
                                    <div class="p-4">
                                        <CodeBlock value=s max_h="max-h-[460px]"/>
                                    </div>
                                }
                                    .into_any()
                            } else {
                                let modalities = by_type(&an, EntityType::Therapy);
                                let weeks = an.weeks;
                                view! {
                                    <div class="divide-y divide-line">
                                        <Row label="Symptoms" items=by_type(&an, EntityType::Symptom)/>
                                        <Row label="Levels / anatomy" items=uniq(by_type(&an, EntityType::Anatomy))/>
                                        <Row label="Imaging" items=by_type(&an, EntityType::Imaging)/>
                                        <Row label="Medications" items=by_type(&an, EntityType::Medication)/>
                                        <div class="grid grid-cols-[140px_1fr] gap-3 px-5 py-3 text-[13px]">
                                            <div class="font-semibold text-muted">"Conservative care"</div>
                                            <div class="text-ink">
                                                <b>{num(weeks)} " weeks"</b>
                                                " documented"
                                                {if modalities.is_empty() { String::new() } else { format!(" · {}", modalities.join(", ")) }}
                                                <div class="mt-1">
                                                    <Badge tone={if weeks >= 6.0 { Tone::Sage } else { Tone::Warn }}>
                                                        {if weeks >= 6.0 { "Meets typical 6-week payer threshold" } else { "Below typical 6-week threshold" }}
                                                    </Badge>
                                                </div>
                                            </div>
                                        </div>
                                        <div class="grid grid-cols-[140px_1fr] gap-3 px-5 py-3 text-[13px]">
                                            <div class="font-semibold text-muted">"Red flags"</div>
                                            <div>
                                                {if !an.active.is_empty() {
                                                    view! {
                                                        <span class="font-semibold text-danger">
                                                            <Icon name="ShieldAlert" class="mr-1 inline size-3.5"/>
                                                            {an.active.join(", ")}
                                                        </span>
                                                    }
                                                        .into_any()
                                                } else {
                                                    view! {
                                                        <span class="text-sage">
                                                            <Icon name="CheckCircle2" class="mr-1 inline size-3.5"/>
                                                            "None active"
                                                        </span>
                                                    }
                                                        .into_any()
                                                }}
                                                {(!an.negated.is_empty())
                                                    .then(|| view! { <div class="mt-1 text-[12px] text-muted">"Explicitly denied: " {an.negated.join(", ")}</div> })}
                                            </div>
                                        </div>
                                    </div>
                                }
                                    .into_any()
                            }
                        }}
                    </Card>
                    <Card>
                        <CardHeader title="ICD-10-CM suggestions" subtitle="Specific codes supersede generic ones; the coder confirms"/>
                        <ul class="divide-y divide-line">
                            {move || {
                                let icd = a.with(|a| a.icd.clone());
                                if icd.is_empty() {
                                    view! { <li class="px-5 py-4 text-[13px] text-muted">"No codable findings yet."</li> }.into_any()
                                } else {
                                    icd.into_iter()
                                        .map(|i| {
                                            view! {
                                                <li class="flex items-center justify-between gap-4 px-5 py-3">
                                                    <div>
                                                        <div class="text-[13px] font-semibold text-ink">
                                                            <span class="mr-2 font-mono text-ocean">{i.code}</span>
                                                            {i.display}
                                                        </div>
                                                        <div class="text-[12px] text-muted">{i.why}</div>
                                                    </div>
                                                    <div class="w-20 text-right">
                                                        <div class="text-[12px] font-semibold tabular-nums text-ink">{num(js_round(i.confidence * 100.0))} "%"</div>
                                                        <div class="mt-1 h-1 rounded-full bg-sand">
                                                            <div class="h-1 rounded-full bg-sage" style:width=format!("{}%", num(i.confidence * 100.0))></div>
                                                        </div>
                                                    </div>
                                                </li>
                                            }
                                        })
                                        .collect_view()
                                        .into_any()
                                }
                            }}
                        </ul>
                    </Card>
                </div>
            </div>
        </div>
    }
}
