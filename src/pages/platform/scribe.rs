use crate::components::browser::{fetch_text, sleep_ms};
use crate::components::icons::Icon;
use crate::components::shell::ToolHeader;
use crate::components::ui::*;
use crate::domain::nlp::*;
use crate::domain::util::num;
use leptos::prelude::*;
use leptos::task::spawn_local;
use regex_lite::Regex;

pub const TRANSCRIPT: &str = "DR: Good morning, Mr. O'Connor. What brings you back today?
PT: The pain in my lower back is still going down my right leg. It's been about 10 weeks now since I hurt it lifting boxes at work.
DR: Any numbness or tingling?
PT: Yes, numbness and tingling on the top of my right foot. And my big toe feels weak.
DR: How did physical therapy go?
PT: I did 8 weeks of physical therapy and the home exercises. I also took naproxen for 6 weeks. It helps a little but the leg pain comes back.
DR: Any trouble controlling your bladder or bowels, or numbness in the groin?
PT: No, nothing like that. No fever either.
DR: On exam, straight leg raise is positive on the right at 40 degrees. Strength is 4 out of 5 in right great toe extension. Reflexes are symmetric. Sensation is decreased in the right L5 dermatome.
DR: Your MRI shows a disc extrusion at L4-L5 pressing on the right L5 nerve root, which matches your symptoms.
DR: Plan: I'd like to do a right L4-L5 transforaminal epidural steroid injection. Continue the home exercise program, and we'll see you 2 weeks after the injection. If you're not better, we'll talk about a surgical consult.
PT: Okay, let's do the injection.";

fn uniq(v: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for s in v {
        if !out.contains(&s) {
            out.push(s);
        }
    }
    out
}

fn or(s: String, fallback: &str) -> String {
    if s.is_empty() {
        fallback.to_string()
    } else {
        s
    }
}

/// On-device SOAP note generator (deterministic clinical NLP; used when no LLM is configured).
pub fn local_soap(t: &str) -> String {
    let ents = extract_entities(t);
    let lines: Vec<&str> = t.split('\n').collect();
    let pt = lines.iter().filter(|l| l.starts_with("PT:")).map(|l| l[3..].trim()).collect::<Vec<_>>().join(" ");
    let dr: Vec<&str> = lines.iter().filter(|l| l.starts_with("DR:")).map(|l| l[3..].trim()).collect();
    let (weeks, evidence) = conservative_care_weeks(&pt);
    let pt_ents = extract_entities(&pt);
    let rf = red_flags(&pt_ents);
    let icd = suggest_icd10(t, &ents);
    let dur = ents.iter().find(|e| e.kind == EntityType::Duration).map(|e| e.text.clone()).unwrap_or_else(|| "not discussed".into());
    let symptoms = uniq(pt_ents.iter().filter(|e| e.kind == EntityType::Symptom && !e.negated).map(|e| e.text.to_lowercase()));
    let exam_re = Regex::new(r"(?i)straight leg|strength|reflex|sensation|exam").expect("regex");
    let exam = dr.iter().filter(|l| exam_re.is_match(l)).copied().collect::<Vec<_>>().join(" ");
    let exam = Regex::new(r"(?i)^On exam,\s*").expect("regex").replace(&exam, "").to_string();
    let img_re = Regex::new(r"(?i)MRI|X-?ray|CT").expect("regex");
    let imaging = dr.iter().find(|l| img_re.is_match(l)).map(|s| s.to_string()).unwrap_or_else(|| "Not discussed.".into());
    let plan_re = Regex::new(r"(?i)^Plan").expect("regex");
    let plan = dr
        .iter()
        .find(|l| plan_re.is_match(l))
        .map(|l| Regex::new(r"(?i)^Plan:\s*").expect("regex").replace(l, "").to_string())
        .unwrap_or_else(|| "Not discussed.".into());
    let procs = uniq(extract_entities(&plan).into_iter().filter(|e| e.kind == EntityType::Procedure).map(|e| e.text));
    let denied: Vec<String> = rf.negated.iter().map(|r| r.text.clone()).chain(pt_ents.iter().filter(|e| e.negated && e.kind != EntityType::Redflag).map(|e| e.text.clone())).collect();
    let assoc = symptoms.iter().filter(|s| !s.contains("back pain")).cloned().collect::<Vec<_>>().join(", ");
    let ev = evidence.join(" ");
    let ev = ev.strip_prefix("I ").map(str::to_string).unwrap_or(ev);
    let icd_lines = icd.iter().map(|i| format!("{} – {}", i.code, i.display)).collect::<Vec<_>>().join("\n");
    let orders = procs.iter().map(|p| format!("{p} (CPT 64483, prior authorization required for commercial plans)")).collect::<Vec<_>>().join("; ");
    format!(
        "SUBJECTIVE\nLow back pain radiating to the right leg for {dur}, onset after lifting at work. Associated {}. Conservative care: {} weeks documented ({}). Denies {}.\n\nOBJECTIVE\n{}\nImaging: {imaging}\n\nASSESSMENT\n{}\nRadicular symptoms concordant with imaging; failed {} weeks of conservative care.\n\nPLAN\n{plan}\nOrders: {}.\nFollow-up 2 weeks post-procedure. Return precautions reviewed.",
        or(assoc, "symptoms not specified"),
        num(weeks),
        or(ev, "not discussed"),
        or(denied.join(", "), "red-flag symptoms"),
        or(exam, "Exam not dictated."),
        or(icd_lines, "No codable assessment found."),
        num(weeks),
        or(orders, "none"),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Engine {
    Claude,
    Local,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Lang {
    En,
    Es,
}

#[component]
pub fn ScribePage() -> impl IntoView {
    let transcript = RwSignal::new(TRANSCRIPT.to_string());
    let note = RwSignal::new(String::new());
    let engine = RwSignal::new(None::<Engine>);
    let engine_note = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let signed = RwSignal::new(false);
    let lang = RwSignal::new(Lang::En);
    let icd = Memo::new(move |_| transcript.with(|t| suggest_icd10(t, &extract_entities(t))));

    let generate = move || {
        loading.set(true);
        signed.set(false);
        let t = transcript.get_untracked();
        spawn_local(async move {
            let body = serde_json::json!({ "task": "scribe", "input": t }).to_string();
            match fetch_text("POST", "/api/ai", Some("application/json"), Some(body)).await.and_then(|(_, b)| serde_json::from_str::<serde_json::Value>(&b).map_err(|e| e.to_string())) {
                Ok(j) => {
                    sleep_ms(900).await;
                    if j["engine"] == "claude" && j["text"].as_str().is_some_and(|s| !s.is_empty()) {
                        note.set(j["text"].as_str().unwrap_or_default().to_string());
                        engine.set(Some(Engine::Claude));
                        engine_note.set(format!(
                            "{} · {} identifiers redacted before the call",
                            j["model"].as_str().unwrap_or("Claude"),
                            j["phiRedacted"].as_u64().unwrap_or(0)
                        ));
                    } else {
                        note.set(local_soap(&t));
                        engine.set(Some(Engine::Local));
                        engine_note.set(j["note"].as_str().unwrap_or("On-device clinical NLP (no API key configured)").to_string());
                    }
                }
                Err(_) => {
                    note.set(local_soap(&t));
                    engine.set(Some(Engine::Local));
                    engine_note.set("Offline: on-device clinical NLP".into());
                }
            }
            loading.set(false);
        });
    };

    let instructions = move || match lang.get() {
        Lang::En => "You are scheduled for a right L4-L5 epidural steroid injection. Arrange a driver for the day of the procedure. Keep doing your home exercises. Call us or go to the ER right away if you have trouble controlling your bladder or bowels, numbness in the groin, or new weakness in your legs.",
        Lang::Es => "Tiene programada una inyección epidural de esteroides en L4-L5 del lado derecho. Organice que alguien lo lleve el día del procedimiento. Continúe con sus ejercicios en casa. Llámenos o vaya a la sala de emergencias de inmediato si tiene problemas para controlar la vejiga o los intestinos, adormecimiento en la ingle o debilidad nueva en las piernas.",
    };

    view! {
        <div class="mx-auto max-w-7xl">
            <ToolHeader
                eyebrow="Clinical AI · Clinical documentation assistance"
                title="Clinical Scribe"
                description="Ambient visit transcript → structured SOAP note with ICD-10 suggestions, E/M support, orders that kick off prior authorization automatically, and bilingual after-visit instructions. The provider reviews and signs; nothing files without them."
                tags=&["LLM (Claude) with on-device fallback", "speech-to-text ready", "ICD-10 / CPT", "EN / ES", "human-in-the-loop"]
                impact=&[("−6 min", "documentation per visit"), ("Same day", "chart closure")]
            />

            <div class="grid gap-6 xl:grid-cols-2">
                <Card class="flex flex-col">
                    <CardHeader
                        title="Visit transcript"
                        subtitle="Follow-up · James O'Connor · Dr. Okafor · 11/16/2026"
                        icon=|| view! { <Icon name="Mic" class="size-4"/> }
                        right=|| {
                            view! {
                                <Badge tone=Tone::Danger>
                                    <span class="size-1.5 animate-pulse-soft rounded-full bg-danger"></span>
                                    " Recorded with consent"
                                </Badge>
                            }
                        }
                    />
                    <textarea
                        prop:value=move || transcript.get()
                        on:input=move |ev| transcript.set(event_target_value(&ev))
                        class="scroll-thin min-h-[420px] w-full flex-1 resize-none bg-cream/40 p-5 font-mono text-[12.5px] leading-relaxed text-ink outline-none"
                    >
                        {TRANSCRIPT}
                    </textarea>
                    <div class="flex items-center justify-between gap-3 border-t border-line p-4">
                        <div class="text-[12px] text-muted">"Speaker-labeled transcript (DR / PT). Editable for the demo."</div>
                        <Button on_click=Callback::new(move |_| generate()) disabled=Signal::derive(move || loading.get())>
                            {move || {
                                if loading.get() {
                                    view! { <Icon name="Loader2" class="size-4 animate-spin"/> }.into_any()
                                } else {
                                    view! { <Icon name="Sparkles" class="size-4"/> }.into_any()
                                }
                            }}
                            " "
                            {move || if loading.get() { "Drafting note…" } else { "Generate SOAP note" }}
                        </Button>
                    </div>
                </Card>

                <Card class="flex flex-col">
                    <CardHeader
                        title="Draft note"
                        subtitle=Signal::derive(move || match engine.get() {
                            Some(Engine::Claude) => {
                                let n = engine_note.get();
                                format!("Generated with Claude {}", if n.is_empty() { String::new() } else { format!("({n})") })
                            }
                            Some(Engine::Local) => engine_note.get(),
                            None => "Generate to draft the note".to_string(),
                        })
                        icon=|| view! { <Icon name="FileText" class="size-4"/> }
                        right=move || {
                            move || {
                                engine
                                    .get()
                                    .map(|e| {
                                        let claude = e == Engine::Claude;
                                        view! {
                                            <Badge tone={if claude { Tone::Ocean } else { Tone::Sage }}>
                                                {if claude {
                                                    view! { <Icon name="Bot" class="size-3"/> }
                                                } else {
                                                    view! { <Icon name="Stethoscope" class="size-3"/> }
                                                }}
                                                {if claude { "LLM" } else { "On-device" }}
                                            </Badge>
                                        }
                                    })
                            }
                        }
                    />
                    <Show
                        when=move || !note.get().is_empty()
                        fallback=|| {
                            view! {
                                <div class="grid min-h-[420px] flex-1 place-items-center p-8 text-center text-[13px] text-muted">
                                    <div>
                                        <Icon name="PenLine" class="mx-auto size-8 text-line-2"/>
                                        <p class="mt-3 max-w-xs">
                                            "The note appears here for review. Edits are tracked, and signing files it to the EMR as a FHIR DocumentReference."
                                        </p>
                                    </div>
                                </div>
                            }
                        }
                    >
                        <textarea
                            prop:value=move || note.get()
                            on:input=move |ev| note.set(event_target_value(&ev))
                            class="scroll-thin min-h-[420px] w-full flex-1 resize-none p-5 font-mono text-[12.5px] leading-relaxed text-ink outline-none"
                        ></textarea>
                    </Show>
                    <div class="flex items-center justify-between gap-3 border-t border-line p-4">
                        <div class="text-[12px] text-muted">
                            {move || if signed.get() { "Signed by Dr. Okafor · filed to EMR · 11/16/2026 10:42" } else { "Requires provider signature" }}
                        </div>
                        {move || {
                            let s = signed.get();
                            view! {
                                <Button
                                    variant={if s { BtnVariant::Secondary } else { BtnVariant::Sage }}
                                    disabled=Signal::derive(move || note.get().is_empty() || signed.get())
                                    on_click=Callback::new(move |_| signed.set(true))
                                >
                                    {if s { view! { <Icon name="CheckCircle2" class="size-4"/> } } else { view! { <Icon name="Send" class="size-4"/> } }}
                                    " "
                                    {if s { "Signed" } else { "Sign & file" }}
                                </Button>
                            }
                        }}
                    </div>
                </Card>
            </div>

            <Show when=move || !note.get().is_empty()>
                <div class="animate-fade-up mt-6 grid gap-6 lg:grid-cols-3">
                    <Card>
                        <CardHeader title="Coding support" subtitle="Coder confirms before claim creation"/>
                        <div class="space-y-3 p-5 text-[13px]">
                            {move || {
                                icd.get()
                                    .into_iter()
                                    .map(|i| {
                                        view! {
                                            <div class="flex items-start gap-2">
                                                <span class="font-mono font-semibold text-ocean">{i.code}</span>
                                                <span class="text-ink">{i.display}</span>
                                            </div>
                                        }
                                    })
                                    .collect_view()
                            }}
                            <div class="rounded-lg bg-sand p-3">
                                <div class="font-semibold text-ink">"E/M suggestion: 99214"</div>
                                <div class="mt-0.5 text-[12px] text-muted">
                                    "Moderate MDM: chronic illness with progression, decision for an elective procedure, prescription drug management. Add modifier 25 if billed with the injection on the same day."
                                </div>
                            </div>
                        </div>
                    </Card>
                    <Card>
                        <CardHeader title="Actions triggered on signing" subtitle="Automations, each logged to the audit trail"/>
                        <ul class="space-y-3 p-5 text-[13px]">
                            {[
                                ("Order 64483 created", "→ Prior Auth Copilot case PA-24812 opened with evidence pre-attached"),
                                ("Procedure scheduling", "→ Garden City suite, next available after expected auth decision"),
                                ("After-visit summary", "→ sent to patient portal in their preferred language"),
                                ("Quality measures", "→ pain score & functional status captured for MIPS reporting"),
                            ]
                                .into_iter()
                                .map(|(a, b)| {
                                    view! {
                                        <li class=move || cx(&["flex gap-2", if signed.get() { "" } else { "opacity-60" }])>
                                            {move || {
                                                if signed.get() {
                                                    view! { <Icon name="CheckCircle2" class="mt-0.5 size-4 shrink-0 text-sage"/> }
                                                } else {
                                                    view! { <Icon name="ClipboardCheck" class="mt-0.5 size-4 shrink-0 text-faint"/> }
                                                }
                                            }}
                                            <span>
                                                <b class="text-ink">{a}</b>
                                                " "
                                                <span class="text-muted">{b}</span>
                                            </span>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ul>
                    </Card>
                    <Card>
                        <CardHeader
                            title="Patient instructions"
                            subtitle="Plain language, 6th-grade reading level"
                            right=move || view! { <Segmented value=lang options=vec![(Lang::En, "EN"), (Lang::Es, "ES")]/> }
                        />
                        <p class="p-5 text-[13px] leading-relaxed text-ink">{instructions}</p>
                    </Card>
                </div>
            </Show>
        </div>
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn local_soap_has_sections() {
        let n = super::local_soap(super::TRANSCRIPT);
        assert!(n.starts_with("SUBJECTIVE\nLow back pain radiating to the right leg for 10 weeks"));
        // Same as the original TypeScript: the transcript yields no ICD-10 rule match.
        assert!(n.contains("No codable assessment found."));
        assert!(n.contains("Conservative care: 8 weeks documented"));
    }
}
