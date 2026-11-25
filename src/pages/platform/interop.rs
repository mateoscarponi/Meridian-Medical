use crate::components::browser::fetch_text;
use crate::components::icons::Icon;
use crate::components::shell::ToolHeader;
use crate::components::ui::*;
use crate::domain::hl7::*;
use crate::domain::util::{js_round, now_ms, num};
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_json::{json, Value};

const ENDPOINTS: [(&str, &str); 7] = [
    ("CapabilityStatement", "/api/fhir/metadata"),
    ("Search patients by name", "/api/fhir/Patient?name=chen"),
    ("Read patient by id", "/api/fhir/Patient/pt-1004"),
    ("Coverage for a patient", "/api/fhir/Coverage?patient=pt-1002"),
    ("Tomorrow's appointments", "/api/fhir/Appointment?date=2026-11-17"),
    ("Orders (ServiceRequest)", "/api/fhir/ServiceRequest"),
    ("Unsupported type → OperationOutcome", "/api/fhir/Medication"),
];

const CONNECTORS: [(&str, &str, &str, &str); 6] = [
    ("eClinicalWorks", "FHIR R4 (US Core) + HL7 v2 interfaces", "ADT, SIU, ORU, DFT", "Live"),
    ("athenahealth", "athenaOne REST API + FHIR R4", "Scheduling, clinical docs, claims", "Live"),
    ("Epic (hospital partners)", "FHIR R4 · SMART Backend Services", "Discharge summaries, imaging", "Live"),
    ("Oracle Health (Cerner)", "FHIR R4 · Care Everywhere-style HIE query", "Records retrieval", "Pilot"),
    ("NextGen", "FHIR R4 + HL7 v2", "Referrals in/out", "Pilot"),
    ("AdvancedMD / DrChrono", "Vendor REST APIs (OAuth 2.0)", "PT & partner practice data", "Planned"),
];

const X12: [(&str, &str); 5] = [
    ("270 / 271", "Eligibility & benefits inquiry / response"),
    ("278", "Prior authorization request & response"),
    ("837P", "Professional claim"),
    ("276 / 277", "Claim status inquiry / response"),
    ("835", "Electronic remittance advice (payments & denials)"),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Out {
    Fhir,
    Ack,
}

#[derive(Clone, PartialEq)]
struct Resp {
    status: u16,
    ms: f64,
    body: Value,
}

fn sample(id: &str) -> &'static SampleMessage {
    SAMPLE_MESSAGES.iter().find(|s| s.id == id).unwrap_or(&SAMPLE_MESSAGES[0])
}

fn to_json(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or(Value::Null)
}

#[component]
pub fn InteropPage() -> impl IntoView {
    let sample_id = RwSignal::new(SAMPLE_MESSAGES[0].id);
    let text = RwSignal::new(SAMPLE_MESSAGES[0].text.to_string());
    let open_seg = RwSignal::new(Some(1usize));
    let out = RwSignal::new(Out::Fhir);
    let server = RwSignal::new(None::<Resp>);
    let sending = RwSignal::new(false);
    let ep = RwSignal::new(ENDPOINTS[1].1);
    let ep_res = RwSignal::new(None::<Resp>);
    let ep_loading = RwSignal::new(false);

    let parsed = Memo::new(move |_| text.with(|t| parse_hl7(t)));
    let fhir = Memo::new(move |_| parsed.with(|p| if p.segments.is_empty() { None } else { Some(hl7_to_fhir(p)) }));
    let resource_count = move || fhir.with(|f| f.as_ref().and_then(|b| b["entry"].as_array().map(|a| a.len())).unwrap_or(0));

    // Choosing a sample loads it (the original did this in the Segmented onChange).
    Effect::new(move |prev: Option<&'static str>| {
        let id = sample_id.get();
        if prev.is_some_and(|p| p != id) {
            text.set(sample(id).text.to_string());
            server.set(None);
            open_seg.set(Some(1));
        }
        id
    });

    let send = move || {
        sending.set(true);
        let body = text.get_untracked();
        spawn_local(async move {
            let t0 = now_ms();
            if let Ok((status, t)) = fetch_text("POST", "/api/hl7", Some("text/plain"), Some(body)).await {
                server.set(Some(Resp { status, ms: js_round(now_ms() - t0), body: to_json(&t) }));
            }
            sending.set(false);
        });
    };
    let call = move |url: &'static str| {
        ep.set(url);
        ep_loading.set(true);
        spawn_local(async move {
            let t0 = now_ms();
            if let Ok((status, t)) = fetch_text("GET", url, None, None).await {
                ep_res.set(Some(Resp { status, ms: js_round(now_ms() - t0), body: to_json(&t) }));
            }
            ep_loading.set(false);
        });
    };

    view! {
        <div class="mx-auto max-w-7xl">
            {move || {
                let segs = parsed.with(|p| p.segments.len());
                view! {
                    <ToolHeader
                        eyebrow="Data & integration · Healthcare interoperability"
                        title="HL7 / FHIR Interoperability Lab"
                        description="A working integration layer: paste any HL7 v2 message to parse it field by field, map it to FHIR R4 resources and get the ACK. Then query this app's live FHIR API. The same code powers the agent, the scheduler and the dashboards."
                        tags=&["HL7 v2.5.1", "FHIR R4 · US Core", "SMART on FHIR", "X12 5010", "MLLP / REST"]
                        dyn_impact=vec![(segs.to_string(), "segments parsed".into()), (resource_count().to_string(), "FHIR resources".into())]
                    />
                }
            }}

            <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
                <Segmented value=sample_id options=SAMPLE_MESSAGES.iter().map(|s| (s.id, s.label.split(" — ").next().unwrap_or(s.label))).collect()/>
                <p class="text-[12.5px] text-muted">{move || sample(sample_id.get()).description}</p>
            </div>

            <div class="grid gap-6 xl:grid-cols-3">
                <Card class="flex flex-col">
                    <CardHeader
                        title="1 · Raw HL7 v2"
                        subtitle=Signal::derive(move || {
                            parsed.with(|p| if p.kind.is_empty() { "Paste a message".to_string() } else { format!("{} · control ID {} · v{}", p.kind, p.control_id, p.version) })
                        })
                        icon=|| view! { <Icon name="Terminal" class="size-4"/> }
                    />
                    <textarea
                        prop:value=move || text.get()
                        on:input=move |ev| {
                            text.set(event_target_value(&ev));
                            server.set(None);
                        }
                        spellcheck="false"
                        class="scroll-thin min-h-[360px] w-full flex-1 resize-none bg-ink p-4 font-mono text-[11.5px] leading-relaxed whitespace-pre text-[#cfe1ee] outline-none"
                    >
                        {SAMPLE_MESSAGES[0].text}
                    </textarea>
                    <div class="flex items-center justify-between gap-2 border-t border-line p-3">
                        {move || {
                            match parsed.with(|p| p.errors.first().cloned()) {
                                Some(e) => view! { <Badge tone=Tone::Danger>{e}</Badge> }.into_any(),
                                None => view! { <Badge tone=Tone::Sage><Icon name="CheckCircle2" class="size-3"/> " Valid structure"</Badge> }.into_any(),
                            }
                        }}
                        <Button
                            size=BtnSize::Sm
                            on_click=Callback::new(move |_| send())
                            disabled=Signal::derive(move || sending.get() || parsed.with(|p| p.segments.is_empty()))
                        >
                            {move || {
                                if sending.get() {
                                    view! { <Icon name="Loader2" class="size-3.5 animate-spin"/> }.into_any()
                                } else {
                                    view! { <Icon name="Play" class="size-3.5"/> }.into_any()
                                }
                            }}
                            " POST /api/hl7"
                        </Button>
                    </div>
                </Card>

                <Card class="flex flex-col">
                    <CardHeader title="2 · Parsed segments" subtitle="Click a segment to see named fields" icon=|| view! { <Icon name="Network" class="size-4"/> }/>
                    <div class="scroll-thin max-h-[440px] flex-1 overflow-y-auto">
                        {move || {
                            parsed
                                .get()
                                .segments
                                .into_iter()
                                .enumerate()
                                .map(|(i, s)| {
                                    let is_open = move || open_seg.get() == Some(i);
                                    let fields = s.fields.clone();
                                    let sid = s.id.clone();
                                    view! {
                                        <div class="border-b border-line last:border-0">
                                            <button
                                                on:click=move |_| open_seg.update(|o| *o = if *o == Some(i) { None } else { Some(i) })
                                                class="flex w-full items-center gap-2 px-4 py-2.5 text-left hover:bg-cream"
                                            >
                                                <Icon name="ChevronRight" class=Signal::derive(move || cx(&["size-3.5 text-faint transition", if is_open() { "rotate-90" } else { "" }]))/>
                                                <span class="rounded bg-navy px-1.5 py-0.5 font-mono text-[11px] font-semibold text-white">{s.id.clone()}</span>
                                                <span class="text-[12.5px] text-ink">{s.label.clone()}</span>
                                                <span class="ml-auto text-[11px] text-faint">{s.fields.len()} " fields"</span>
                                            </button>
                                            <Show when=is_open>
                                                <table class="w-full bg-cream/60 text-[11.5px]">
                                                    <tbody>
                                                        {fields
                                                            .iter()
                                                            .map(|f| {
                                                                let value = f
                                                                    .components
                                                                    .iter()
                                                                    .map(|rep| rep.iter().filter(|c| !c.is_empty()).cloned().collect::<Vec<_>>().join(" · "))
                                                                    .collect::<Vec<_>>()
                                                                    .join(" ~ ");
                                                                view! {
                                                                    <tr class="border-t border-line/70">
                                                                        <td class="w-16 px-4 py-1.5 font-mono text-faint">{sid.clone()} "-" {f.index}</td>
                                                                        <td class="w-40 py-1.5 pr-2 font-semibold text-muted">{f.name.clone()}</td>
                                                                        <td class="py-1.5 pr-4 font-mono break-all text-ink">{value}</td>
                                                                    </tr>
                                                                }
                                                            })
                                                            .collect_view()}
                                                    </tbody>
                                                </table>
                                            </Show>
                                        </div>
                                    }
                                })
                                .collect_view()
                        }}
                    </div>
                </Card>

                <Card class="flex flex-col">
                    <CardHeader
                        title="3 · Output"
                        subtitle=Signal::derive(move || match server.get() {
                            Some(s) => format!("Server: HTTP {} · {} ms round trip", s.status, num(s.ms)),
                            None => "Mapped client-side; POST to run on the server".to_string(),
                        })
                        right=move || view! { <Segmented value=out options=vec![(Out::Fhir, "FHIR Bundle"), (Out::Ack, "ACK")]/> }
                    />
                    <div class="flex-1 p-3">
                        <CodeBlock
                            value=Signal::derive(move || {
                                let srv = server.get();
                                match out.get() {
                                    Out::Fhir => match srv {
                                        Some(s) => pretty_json(&s.body["fhir"]),
                                        None => pretty_json(&fhir.get().unwrap_or_else(|| json!({}))),
                                    },
                                    Out::Ack => match srv {
                                        Some(s) => s.body["ack"].as_str().unwrap_or("undefined").replace('\r', "\n"),
                                        None => "POST the message to receive the HL7 ACK (MSA|AA = accepted).".to_string(),
                                    },
                                }
                            })
                            max_h="max-h-[400px]"
                        />
                    </div>
                </Card>
            </div>

            <div class="mt-6 grid gap-6 xl:grid-cols-[1fr_1.2fr]">
                <Card>
                    <CardHeader title="Live FHIR API" subtitle="Real endpoints served by this app (synthetic data)" icon=|| view! { <Icon name="Server" class="size-4"/> }/>
                    <ul class="divide-y divide-line">
                        {ENDPOINTS
                            .iter()
                            .map(|(label, url)| {
                                let url = *url;
                                view! {
                                    <li>
                                        <button
                                            on:click=move |_| call(url)
                                            class=move || {
                                                cx(&[
                                                    "flex w-full items-center justify-between gap-3 px-5 py-2.5 text-left transition",
                                                    if ep.get() == url && ep_res.with(|r| r.is_some()) { "bg-sky-2" } else { "hover:bg-cream" },
                                                ])
                                            }
                                        >
                                            <span>
                                                <span class="block text-[13px] font-semibold text-ink">{*label}</span>
                                                <span class="block font-mono text-[11.5px] text-ocean">"GET " {url}</span>
                                            </span>
                                            <Icon name="ArrowRight" class="size-4 text-faint"/>
                                        </button>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                </Card>
                <Card class="flex flex-col">
                    <CardHeader
                        title="Response"
                        subtitle=Signal::derive(move || match ep_res.get() {
                            Some(r) => format!("HTTP {} · {} ms · application/fhir+json", r.status, num(r.ms)),
                            None => "Select an endpoint".to_string(),
                        })
                        right=move || move || ep_loading.get().then(|| view! { <Icon name="Loader2" class="size-4 animate-spin text-ocean"/> })
                    />
                    <div class="flex-1 p-3">
                        <CodeBlock
                            value=Signal::derive(move || ep_res.get().map(|r| pretty_json(&r.body)).unwrap_or_else(|| "// response appears here".to_string()))
                            max_h="max-h-[420px]"
                        />
                    </div>
                </Card>
            </div>

            <div class="mt-6 grid gap-6 lg:grid-cols-[1.4fr_1fr]">
                <Card class="overflow-hidden">
                    <CardHeader title="EMR / EHR connectors" subtitle="How each system connects to the platform"/>
                    <div class="overflow-x-auto">
                        <table class="w-full min-w-[560px] text-[13px]">
                            <thead class="bg-cream text-left text-[11px] uppercase tracking-wide text-faint">
                                <tr>
                                    <th class="px-5 py-2.5">"System"</th>
                                    <th class="px-2 py-2.5">"Method"</th>
                                    <th class="px-2 py-2.5">"Data"</th>
                                    <th class="px-5 py-2.5 text-right">"Status"</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-line">
                                {CONNECTORS
                                    .iter()
                                    .map(|(emr, method, feeds, status)| {
                                        let tone = match *status {
                                            "Live" => Tone::Sage,
                                            "Pilot" => Tone::Ocean,
                                            _ => Tone::Neutral,
                                        };
                                        view! {
                                            <tr>
                                                <td class="px-5 py-3 font-semibold text-ink">{*emr}</td>
                                                <td class="px-2 py-3 text-muted">{*method}</td>
                                                <td class="px-2 py-3 text-muted">{*feeds}</td>
                                                <td class="px-5 py-3 text-right"><Badge tone=tone>{*status}</Badge></td>
                                            </tr>
                                        }
                                    })
                                    .collect_view()}
                            </tbody>
                        </table>
                    </div>
                </Card>
                <Card>
                    <CardHeader title="Payer transactions (X12 5010)" subtitle="Through a clearinghouse API"/>
                    <ul class="divide-y divide-line">
                        {X12
                            .iter()
                            .map(|(k, v)| {
                                view! {
                                    <li class="flex items-center gap-4 px-5 py-3 text-[13px]">
                                        <span class="w-20 font-mono font-semibold text-ocean">{*k}</span>
                                        <span class="text-muted">{*v}</span>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                </Card>
            </div>
        </div>
    }
}
