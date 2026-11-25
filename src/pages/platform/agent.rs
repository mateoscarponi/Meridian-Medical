use crate::components::browser::fetch_text;
use crate::components::icons::Icon;
use crate::components::shell::ToolHeader;
use crate::components::ui::*;
use leptos::html::Div;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Clone)]
enum Step {
    User(&'static str),
    Agent { text: &'static str, escalate: bool },
    Tool { name: &'static str, args: Value, fetch_url: Option<&'static str>, result: Option<Value>, pick: Option<usize> },
}

fn agent(text: &'static str) -> Step {
    Step::Agent { text, escalate: false }
}
fn fetch_tool(name: &'static str, args: Value, url: &'static str, pick: usize) -> Step {
    Step::Tool { name, args, fetch_url: Some(url), result: None, pick: Some(pick) }
}
fn static_tool(name: &'static str, args: Value, result: Value) -> Step {
    Step::Tool { name, args, fetch_url: None, result: Some(result), pick: None }
}

/// `{ total, entry: first `max` resources }` of a FHIR Bundle.
fn pick_bundle(r: &Value, max: usize) -> Value {
    let entry: Vec<Value> = r["entry"].as_array().map(|e| e.iter().take(max).map(|x| x["resource"].clone()).collect()).unwrap_or_default();
    let mut o = serde_json::Map::new();
    if !r["total"].is_null() {
        o.insert("total".into(), r["total"].clone());
    }
    o.insert("entry".into(), Value::Array(entry));
    Value::Object(o)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Key {
    Reschedule,
    Auth,
    Redflag,
}

struct Scenario {
    title: &'static str,
    channel: &'static str,
    steps: Vec<Step>,
}

fn scenario(k: Key) -> Scenario {
    match k {
        Key::Reschedule => Scenario {
            title: "Reschedule + outside MRI (Spanish)",
            channel: "SMS",
            steps: vec![
                Step::User("Hola, soy Maria Gonzalez. Necesito mover mi cita del martes y tengo una resonancia de otro lugar."),
                agent("¡Hola Maria! Con gusto le ayudo. Para proteger su información, ¿me confirma su fecha de nacimiento?"),
                Step::User("12 de abril de 1968"),
                fetch_tool("lookup_patient", json!({ "name": "Gonzalez", "birthdate": "1968-04-12" }), "/api/fhir/Patient?name=gonzalez", 1),
                fetch_tool("get_appointments", json!({ "patient": "pt-1001", "date": "2026-11-17" }), "/api/fhir/Appointment?patient=pt-1001", 1),
                static_tool(
                    "find_open_slots",
                    json!({ "location": "Garden City", "provider": "Dr. Okafor", "from": "2026-11-18", "visit": "follow-up", "language": "es" }),
                    json!({ "slots": ["2026-11-19T14:00", "2026-11-19T15:15", "2026-11-20T09:30"] }),
                ),
                agent("Gracias, Maria, ya la encontré. Su cita es el martes 17 a las 8:15 AM con el Dr. Okafor. Tengo disponible el jueves 19 a las 2:00 PM o 3:15 PM, o el viernes 20 a las 9:30 AM en Garden City. ¿Cuál prefiere?"),
                Step::User("El jueves a las 2 está perfecto."),
                static_tool(
                    "reschedule_appointment",
                    json!({ "appointment": "appt-3000", "newStart": "2026-11-19T14:00:00-05:00" }),
                    json!({ "status": "booked", "id": "appt-3000", "previous": "cancelled → slot released to waitlist", "hl7": "SIU^S14 sent to EMR" }),
                ),
                static_tool(
                    "send_secure_link",
                    json!({ "patient": "pt-1001", "purpose": "imaging_upload", "language": "es", "expiresHours": 72 }),
                    json!({ "sent": true, "channel": "SMS" }),
                ),
                agent("¡Listo! Su cita quedó el jueves 19 de noviembre a las 2:00 PM en Garden City. Le envié un enlace seguro para subir su resonancia antes de la cita, así el doctor la revisa con usted. ¿Algo más en lo que le pueda ayudar?"),
            ],
        },
        Key::Auth => Scenario {
            title: "Workers' comp authorization status",
            channel: "Web chat",
            steps: vec![
                Step::User("Hi, Robert Williams here. Is my neck injection approved? Workers' comp case."),
                agent("Hi Robert, happy to check. To verify it's you, can you share your date of birth?"),
                Step::User("June 2, 1981"),
                fetch_tool("lookup_patient", json!({ "name": "Williams", "birthdate": "1981-06-02" }), "/api/fhir/Patient?name=williams", 1),
                fetch_tool("get_coverage", json!({ "patient": "pt-1004" }), "/api/fhir/Coverage?patient=pt-1004", 1),
                static_tool(
                    "get_prior_auth_status",
                    json!({ "order": "62321", "patient": "pt-1004" }),
                    json!({ "case": "PA-24816", "status": "not_required", "reason": "Within NY WCB Medical Treatment Guidelines", "scheduled": "2026-11-17T13:00 Garden City" }),
                ),
                agent("Thanks, Robert. Good news: your cervical injection is within the New York workers' comp treatment guidelines, so it doesn't need a separate approval. You're scheduled for Tuesday, Nov 17 at 1:00 PM in Garden City. Please bring your WCB case number and arrange a ride home. Anything else?"),
            ],
        },
        Key::Redflag => Scenario {
            title: "Red flag → immediate human handoff",
            channel: "Web chat",
            steps: vec![
                Step::User("My back pain got a lot worse since yesterday and now I can't really feel when I need to pee. Can I get an appointment this week?"),
                static_tool(
                    "safety_screen",
                    json!({ "text": "…can't really feel when I need to pee…" }),
                    json!({ "red_flags": ["possible urinary retention / loss of bladder sensation"], "risk": "emergent", "policy": "no scheduling; escalate" }),
                ),
                static_tool(
                    "handoff_to_human",
                    json!({ "queue": "on-call clinician", "priority": "emergent", "transcript": true }),
                    json!({ "paged": "Dr. Okafor (on call)", "eta": "callback < 10 min" }),
                ),
                Step::Agent {
                    escalate: true,
                    text: "I'm concerned about what you're describing. New changes in bladder feeling with worsening back pain can be a medical emergency. Please go to the nearest emergency room now or call 911. I've alerted our on-call doctor, who will call you within 10 minutes. Can you confirm the best number to reach you?",
                },
            ],
        },
    }
}

const TOOLS: [(&str, &str); 6] = [
    ("lookup_patient", "FHIR Patient search + identity verification (name + DOB)"),
    ("get_appointments / find_open_slots", "FHIR Appointment + scheduling rules"),
    ("reschedule_appointment", "Writes back to the EMR (HL7 SIU^S14)"),
    ("get_coverage / get_prior_auth_status", "Coverage + Prior Auth Copilot"),
    ("send_secure_link", "Forms, imaging upload, payments (expiring)"),
    ("safety_screen / handoff_to_human", "Red-flag detection → on-call escalation"),
];

#[component]
pub fn AgentPage() -> impl IntoView {
    let key = RwSignal::new(Key::Reschedule);
    let shown = RwSignal::new(0usize);
    let results = RwSignal::new(HashMap::<usize, Value>::new());
    let auto = RwSignal::new(false);
    let chat = NodeRef::<Div>::new();
    let steps = Memo::new(move |_| key.get());
    let sc = move || scenario(steps.get());
    let len = move || sc().steps.len();

    let reset = move || {
        shown.set(0);
        results.set(HashMap::new());
        auto.set(false);
    };
    // Reset whenever the scenario changes (skips the initial run, which needs no reset).
    Effect::new(move |prev: Option<Key>| {
        let k = key.get();
        if prev.is_some_and(|p| p != k) {
            reset();
        }
        k
    });

    let next = move || {
        let i = shown.get_untracked();
        let s = scenario(key.get_untracked());
        let Some(step) = s.steps.get(i).cloned() else { return };
        match step {
            Step::Tool { fetch_url: Some(url), pick, .. } => {
                spawn_local(async move {
                    let r = match fetch_text("GET", url, None, None).await {
                        Ok((_, t)) => match serde_json::from_str::<Value>(&t) {
                            Ok(j) => pick.map(|m| pick_bundle(&j, m)).unwrap_or(j),
                            Err(_) => json!({ "error": "network" }),
                        },
                        Err(_) => json!({ "error": "network" }),
                    };
                    results.update(|x| {
                        x.insert(i, r);
                    });
                    shown.set(i + 1);
                });
            }
            Step::Tool { result, .. } => {
                results.update(|x| {
                    x.insert(i, result.unwrap_or(Value::Null));
                });
                shown.set(i + 1);
            }
            _ => shown.set(i + 1),
        }
    };

    // Auto-play: one step per tick while `auto` is on.
    Effect::new(move |_| {
        if !auto.get() {
            return;
        }
        let n = shown.get();
        let s = scenario(key.get());
        if n >= s.steps.len() {
            auto.set(false);
            return;
        }
        let delay = if matches!(s.steps[n], Step::Tool { .. }) { 650 } else { 1100 };
        if let Ok(h) = set_timeout_with_handle(next, Duration::from_millis(delay)) {
            on_cleanup(move || h.clear());
        }
    });

    // Keep the chat scrolled to the newest message.
    Effect::new(move |_| {
        shown.track();
        if let Some(el) = chat.get() {
            let opts = web_sys::ScrollToOptions::new();
            opts.set_top(el.scroll_height() as f64);
            opts.set_behavior(web_sys::ScrollBehavior::Smooth);
            el.scroll_to_with_scroll_to_options(&opts);
        }
    });

    let visible = move || sc().steps.into_iter().take(shown.get()).enumerate().collect::<Vec<_>>();
    let tool_items = move || visible().into_iter().filter(|(_, s)| matches!(s, Step::Tool { .. })).collect::<Vec<_>>();

    view! {
        <div class="mx-auto max-w-7xl">
            <ToolHeader
                eyebrow="Front office · AI agents"
                title="AI Front-Desk Agent"
                description="A tool-using agent that handles routine patient requests end to end, around the clock and in the patient's language. Its tools read and write real systems (the patient lookups below call this app's live FHIR API), and its guardrails escalate anything clinical to a human right away."
                tags=&["LLM tool use", "FHIR R4 read/write", "identity verification", "guardrails", "human handoff"]
                impact=&[("62%", "requests resolved end-to-end"), ("24/7", "EN / ES coverage")]
            />

            <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
                <Segmented value=key options=vec![(Key::Reschedule, "Reschedule (ES)"), (Key::Auth, "Auth status (WC)"), (Key::Redflag, "Red flag")]/>
                <div class="flex gap-2">
                    <Button size=BtnSize::Sm variant=BtnVariant::Secondary on_click=Callback::new(move |_| reset())>
                        <Icon name="RotateCcw" class="size-3.5"/>
                        " Reset"
                    </Button>
                    <Button
                        size=BtnSize::Sm
                        variant=BtnVariant::Secondary
                        on_click=Callback::new(move |_| next())
                        disabled=Signal::derive(move || shown.get() >= len() || auto.get())
                    >
                        <Icon name="SkipForward" class="size-3.5"/>
                        " Step"
                    </Button>
                    <Button size=BtnSize::Sm on_click=Callback::new(move |_| auto.set(true)) disabled=Signal::derive(move || shown.get() >= len() || auto.get())>
                        <Icon name="Play" class="size-3.5"/>
                        " Play conversation"
                    </Button>
                </div>
            </div>

            <div class="grid gap-6 xl:grid-cols-[1fr_1fr]">
                <Card class="flex h-[620px] flex-col overflow-hidden">
                    <CardHeader
                        title=Signal::derive(move || sc().title.to_string())
                        subtitle=Signal::derive(move || format!("Channel: {}", sc().channel))
                        icon=|| view! { <Icon name="Bot" class="size-4"/> }
                        right=|| view! { <Badge tone=Tone::Sage><Icon name="Lock" class="size-3"/> " Verified session"</Badge> }
                    />
                    <div node_ref=chat class="scroll-thin flex-1 space-y-3 overflow-y-auto bg-cream/50 p-5">
                        <Show when=move || shown.get() == 0>
                            <div class="grid h-full place-items-center text-[13px] text-muted">"Press “Play conversation” or step through it."</div>
                        </Show>
                        <For each=visible key=|(i, _)| *i let:item>
                            {match item.1 {
                                Step::User(text) => view! {
                                    <div class="animate-fade-up flex justify-end gap-2">
                                        <div class="max-w-[80%] rounded-2xl rounded-br-sm bg-navy px-4 py-2.5 text-[13.5px] text-white">{text}</div>
                                        <div class="grid size-7 shrink-0 place-items-center rounded-full bg-sand text-muted">
                                            <Icon name="User" class="size-3.5"/>
                                        </div>
                                    </div>
                                }
                                .into_any(),
                                Step::Agent { text, escalate } => view! {
                                    <div class="animate-fade-up flex gap-2">
                                        <div class=cx(&["grid size-7 shrink-0 place-items-center rounded-full text-white", if escalate { "bg-danger" } else { "bg-sage" }])>
                                            {if escalate {
                                                view! { <Icon name="AlertOctagon" class="size-3.5"/> }
                                            } else {
                                                view! { <Icon name="Bot" class="size-3.5"/> }
                                            }}
                                        </div>
                                        <div class=cx(&[
                                            "max-w-[80%] rounded-2xl rounded-bl-sm px-4 py-2.5 text-[13.5px] shadow-sm",
                                            if escalate { "bg-danger-soft text-danger ring-1 ring-danger/20" } else { "bg-white text-ink ring-1 ring-line" },
                                        ])>{text}</div>
                                    </div>
                                }
                                .into_any(),
                                Step::Tool { name, .. } => view! {
                                    <div class="animate-fade-up ml-9 flex items-center gap-2 text-[11.5px] text-muted">
                                        <Icon name="Wrench" class="size-3.5 text-gold"/>
                                        " called "
                                        <code class="rounded bg-white px-1.5 py-0.5 font-mono text-[11px] text-ink ring-1 ring-line">{name}</code>
                                    </div>
                                }
                                .into_any(),
                            }}
                        </For>
                    </div>
                </Card>

                <Card class="flex h-[620px] flex-col overflow-hidden">
                    <CardHeader
                        title="Agent trace"
                        subtitle="Every tool call, its arguments and the live result"
                        icon=|| view! { <Icon name="Wrench" class="size-4"/> }
                    />
                    <div class="scroll-thin flex-1 space-y-3 overflow-y-auto bg-ink p-4 font-mono text-[11.5px] leading-relaxed text-[#cfe1ee]">
                        <Show when=move || !visible().iter().any(|(_, s)| matches!(s, Step::Tool { .. }))>
                            <div class="text-white/40">"// tool calls will appear here"</div>
                        </Show>
                        <For each=tool_items key=|(i, _)| *i let:item>
                            {
                                let (i, step) = item;
                                let Step::Tool { name, args, fetch_url, .. } = step else { unreachable!() };
                                view! {
                                    <div class="animate-fade-up rounded-lg border border-white/10 bg-white/[0.03] p-3">
                                        <div class="text-[#e6c88a]">"→ " {name} "(" {serde_json::to_string(&args).unwrap_or_default()} ")"</div>
                                        {fetch_url.map(|u| view! { <div class="mt-1 text-white/40">"GET " {u}</div> })}
                                        <pre class="mt-2 whitespace-pre-wrap text-[#9fd8b8]">
                                            "← "
                                            {move || results.with(|r| r.get(&i).map(pretty_json).unwrap_or_else(|| "undefined".into()))}
                                        </pre>
                                    </div>
                                }
                            }
                        </For>
                    </div>
                </Card>
            </div>

            <div class="mt-6 grid gap-6 lg:grid-cols-2">
                <Card>
                    <CardHeader title="Tools available to the agent"/>
                    <ul class="divide-y divide-line">
                        {TOOLS
                            .iter()
                            .map(|(n, d)| {
                                view! {
                                    <li class="flex items-start justify-between gap-4 px-5 py-3 text-[13px]">
                                        <code class="font-mono text-[12px] text-ocean">{*n}</code>
                                        <span class="text-right text-muted">{*d}</span>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                </Card>
                <Card>
                    <CardHeader title="Guardrails" icon=|| view! { <Icon name="ShieldCheck" class="size-4"/> }/>
                    <ul class="space-y-2.5 p-5 text-[13px] text-muted">
                        <li>"• " <b class="text-ink">"No clinical advice."</b> " Symptoms are screened first; red flags skip scheduling and go straight to the on-call clinician."</li>
                        <li>"• " <b class="text-ink">"Identity before PHI:"</b> " name + DOB match, and write actions need a verified session."</li>
                        <li>"• " <b class="text-ink">"Minimum necessary:"</b> " tools return only the fields the task needs."</li>
                        <li>"• " <b class="text-ink">"Scoped writes:"</b> " the agent can book, reschedule and send links, never edit clinical data."</li>
                        <li>"• " <b class="text-ink">"Full audit trail"</b> " of every message and tool call, plus a human takeover button in the staff console."</li>
                        <li>"• " <b class="text-ink">"LLM under a BAA"</b> " with zero data retention, prompt-injection filtering on inbound text, and evals run before each release."</li>
                    </ul>
                </Card>
            </div>
        </div>
    }
}
