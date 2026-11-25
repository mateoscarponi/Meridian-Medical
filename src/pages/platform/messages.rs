use crate::components::browser::fetch_text;
use crate::components::icons::Icon;
use crate::components::shell::ToolHeader;
use crate::components::ui::*;
use crate::domain::util::{js_round, num};
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashMap;

struct Msg {
    id: u32,
    from: &'static str,
    channel: &'static str,
    time: &'static str,
    lang: &'static str,
    text: &'static str,
    intent: &'static str,
    conf: f64,
    draft: &'static str,
    route: &'static str,
}

const URGENT: &str = "Urgent: clinical";

static MSGS: [Msg; 5] = [
    Msg {
        id: 1, from: "Maria Gonzalez", channel: "SMS", time: "7:52 AM", lang: "ES",
        text: "Hola, necesito cambiar mi cita del martes. ¿Tienen algo el jueves en Garden City? Gracias.",
        intent: "Appointment", conf: 0.97, route: "Front desk · Garden City",
        draft: "Hola Maria, con gusto. Tenemos el jueves 19 de noviembre a las 2:00 PM o 3:15 PM en Garden City. Responda 1 o 2 para confirmar y cancelamos su cita del martes. — Equipo de Meridian Orthopedics & Spine Care",
    },
    Msg {
        id: 2, from: "Thomas Russo", channel: "Portal", time: "8:05 AM", lang: "EN",
        text: "I had my neck surgery 9 days ago. Since last night I have a fever of 101.8 and the incision is red and draining. Is that normal?",
        intent: URGENT, conf: 0.99, route: "On-call surgeon · callback < 15 min",
        draft: "Mr. Russo, thank you for letting us know. Fever with redness and drainage after surgery needs to be checked today. Our on-call surgeon has been notified and will call you within 15 minutes. If you have trouble breathing or swallowing, or feel very unwell, call 911 or go to the nearest emergency room now. — Meridian Orthopedics & Spine Care Team",
    },
    Msg {
        id: 3, from: "Robert Williams", channel: "Voicemail", time: "8:21 AM", lang: "EN",
        text: "(transcribed) Hi, this is Robert Williams, workers' comp patient. Just checking if my neck injection was approved yet. Thanks.",
        intent: "Prior auth status", conf: 0.95, route: "Auto-answered from auth tracker",
        draft: "Hi Robert, good news: your neck injection falls within the New York workers' comp treatment guidelines, so no separate approval is needed. You're scheduled for Tuesday, Nov 17 at 1:00 PM in Garden City. Please bring your WCB case number. — Meridian Orthopedics & Spine Care Team",
    },
    Msg {
        id: 4, from: "Kevin Murphy", channel: "SMS", time: "8:40 AM", lang: "EN",
        text: "Got a bill for $150 but I thought my insurance covered the visit?",
        intent: "Billing", conf: 0.93, route: "Billing team",
        draft: "Hi Kevin, thanks for reaching out. We're reviewing your 11/14 visit with Oxford now and will text you within one business day with an explanation of benefits breakdown. No payment is due while we review. — Meridian Orthopedics & Spine Care Team",
    },
    Msg {
        id: 5, from: "Carmen Diaz", channel: "Portal", time: "9:02 AM", lang: "ES",
        text: "¿Puedo subir los discos de mi resonancia antes de la cita? Me los dieron en otro centro.",
        intent: "Records", conf: 0.94, route: "Auto-answered · imaging upload link",
        draft: "Hola Carmen, ¡sí! Use este enlace seguro para subir sus imágenes antes de la cita: [enlace seguro]. También puede traer el disco ese día. Así el doctor podrá revisarlas con usted. — Equipo de Meridian Orthopedics & Spine Care",
    },
];

fn intent_tone(i: &str) -> Tone {
    match i {
        URGENT => Tone::Danger,
        "Appointment" => Tone::Ocean,
        "Prior auth status" => Tone::Gold,
        "Records" => Tone::Sage,
        _ => Tone::Neutral,
    }
}

fn channel_icon(c: &str) -> &'static str {
    match c {
        "SMS" => "Smartphone",
        "Portal" => "MessageSquare",
        _ => "Mic",
    }
}

fn msg(id: u32) -> &'static Msg {
    MSGS.iter().find(|m| m.id == id).unwrap_or(&MSGS[0])
}

#[component]
pub fn MessagesPage() -> impl IntoView {
    let sel = RwSignal::new(2u32);
    let sent = RwSignal::new(Vec::<u32>::new());
    let drafts = RwSignal::new(MSGS.iter().map(|m| (m.id, m.draft.to_string())).collect::<HashMap<u32, String>>());
    let engine = RwSignal::new(HashMap::<u32, String>::new());
    let loading = RwSignal::new(false);
    let m = move || msg(sel.get());
    let is_sent = move || sent.with(|s| s.contains(&sel.get()));

    let regenerate = move || {
        let mm = msg(sel.get_untracked());
        loading.set(true);
        spawn_local(async move {
            let body = serde_json::json!({ "task": "reply", "input": format!("Patient {} via {}: {}", mm.from, mm.channel, mm.text) }).to_string();
            let label = match fetch_text("POST", "/api/ai", Some("application/json"), Some(body)).await {
                Ok((_, text)) => match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(j) if j["engine"] == "claude" && j["text"].as_str().is_some_and(|t| !t.is_empty()) => {
                        let t = j["text"].as_str().unwrap_or_default().to_string();
                        drafts.update(|d| {
                            d.insert(mm.id, t);
                        });
                        "Claude".to_string()
                    }
                    Ok(_) => "Template library (no API key configured)".to_string(),
                    Err(_) => "Template library (offline)".to_string(),
                },
                Err(_) => "Template library (offline)".to_string(),
            };
            engine.update(|e| {
                e.insert(mm.id, label);
            });
            loading.set(false);
        });
    };

    // Urgent first, then original order (stable sort, like the original comparator).
    let mut ordered: Vec<&'static Msg> = MSGS.iter().collect();
    ordered.sort_by_key(|x| if x.intent == URGENT { 0 } else { 1 });

    view! {
        <div class="mx-auto max-w-7xl">
            <ToolHeader
                eyebrow="Front office · Patient communication systems"
                title="Patient Messages"
                description="One inbox for SMS, portal and voicemail. AI classifies intent, catches emergencies first, drafts replies in the patient's language, and answers routine questions straight from live data. Staff approve before anything is sent."
                tags=&["intent classification", "red-flag escalation", "EN / ES", "voicemail transcription", "LLM drafts + approval"]
                impact=&[("−23 h", "phone & inbox / week"), ("4 min", "median first response")]
            />

            <div class="grid gap-6 xl:grid-cols-[380px_1fr]">
                <Card class="h-fit overflow-hidden">
                    <CardHeader title="Inbox" subtitle="Sorted by urgency, then time"/>
                    <ul class="divide-y divide-line">
                        {ordered
                            .into_iter()
                            .map(|x| {
                                let id = x.id;
                                let urgent = x.intent == URGENT;
                                view! {
                                    <li>
                                        <button
                                            on:click=move |_| sel.set(id)
                                            class=move || {
                                                let is_sel = sel.get() == id;
                                                cx(&[
                                                    "w-full px-4 py-3.5 text-left transition",
                                                    if is_sel { "bg-sky-2 shadow-[inset_3px_0_0_var(--color-ocean)]" } else { "hover:bg-cream" },
                                                    if urgent && !is_sel { "bg-danger-soft/40" } else { "" },
                                                ])
                                            }
                                        >
                                            <div class="flex items-center justify-between gap-2">
                                                <span class="flex items-center gap-1.5 text-[13px] font-semibold text-ink">
                                                    <Icon name=channel_icon(x.channel) class="size-3.5 text-muted"/>
                                                    {x.from}
                                                </span>
                                                <span class="text-[11px] text-faint">{x.time}</span>
                                            </div>
                                            <p class="mt-1 line-clamp-2 text-[12.5px] text-muted">{x.text}</p>
                                            <div class="mt-2 flex flex-wrap items-center gap-1.5">
                                                <Badge tone=intent_tone(x.intent)>
                                                    {urgent.then(|| view! { <Icon name="AlertOctagon" class="size-3"/> })}
                                                    {x.intent}
                                                </Badge>
                                                <Badge>{x.lang}</Badge>
                                                <Show when=move || sent.with(|s| s.contains(&id))>
                                                    <Badge tone=Tone::Sage><Icon name="CheckCircle2" class="size-3"/> " Replied"</Badge>
                                                </Show>
                                            </div>
                                        </button>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                </Card>

                <div class="space-y-6">
                    <Show when=move || m().intent == URGENT>
                        <div class="animate-fade-up flex items-start gap-3 rounded-2xl border border-danger/30 bg-danger-soft p-4 text-danger" role="alert">
                            <Icon name="Phone" class="mt-0.5 size-5 shrink-0"/>
                            <div class="text-[13px]">
                                <b class="text-[14px]">"Escalated automatically at " {move || m().time} "."</b>
                                " Possible post-operative infection. On-call surgeon paged; callback required within 15 minutes; the message was pulled out of the routine queue."
                            </div>
                        </div>
                    </Show>
                    <Card>
                        <CardHeader
                            title=Signal::derive(move || m().from.to_string())
                            subtitle=Signal::derive(move || format!("{} · {} · {}", m().channel, m().time, if m().lang == "ES" { "Spanish" } else { "English" }))
                            right=move || {
                                view! {
                                    <Badge tone=Signal::derive(move || intent_tone(m().intent))>
                                        {move || m().intent} " · " {move || num(js_round(m().conf * 100.0))} "%"
                                    </Badge>
                                }
                            }
                        />
                        <div class="p-5">
                            <div class="max-w-xl rounded-2xl rounded-tl-sm bg-sand px-4 py-3 text-[14px] text-ink">{move || m().text}</div>
                            <div class="mt-2 text-[12px] text-muted">"Routed to: " <b class="text-ink">{move || m().route}</b></div>
                        </div>
                    </Card>
                    <Card>
                        <CardHeader
                            title="Suggested reply"
                            subtitle=Signal::derive(move || match engine.with(|e| e.get(&sel.get()).cloned()) {
                                Some(e) => format!("Drafted by: {e}"),
                                None => "Drafted from the template library + live schedule / auth data".to_string(),
                            })
                            icon=|| view! { <Icon name="Sparkles" class="size-4"/> }
                            right=move || {
                                view! {
                                    <Button
                                        size=BtnSize::Sm
                                        variant=BtnVariant::Secondary
                                        on_click=Callback::new(move |_| regenerate())
                                        disabled=Signal::derive(move || loading.get())
                                    >
                                        {move || {
                                            if loading.get() {
                                                view! { <Icon name="Loader2" class="size-3.5 animate-spin"/> }.into_any()
                                            } else {
                                                view! { <Icon name="Bot" class="size-3.5"/> }.into_any()
                                            }
                                        }}
                                        " Regenerate with AI"
                                    </Button>
                                }
                            }
                        />
                        <textarea
                            prop:value=move || drafts.with(|d| d.get(&sel.get()).cloned().unwrap_or_default())
                            on:input=move |ev| {
                                let v = event_target_value(&ev);
                                let id = sel.get_untracked();
                                drafts.update(|d| {
                                    d.insert(id, v);
                                });
                            }
                            class="h-36 w-full resize-none p-5 text-[14px] leading-relaxed text-ink outline-none"
                        >
                            // initial draft (message id 2) so the server-rendered page is complete
                            {MSGS[1].draft}
                        </textarea>
                        <div class="flex flex-wrap items-center justify-between gap-3 border-t border-line p-4">
                            <span class="inline-flex items-center gap-1.5 text-[12px] text-muted">
                                <Icon name="Languages" class="size-3.5"/>
                                " Reply language matches the patient ("
                                {move || m().lang}
                                ")"
                            </span>
                            <Button
                                variant=BtnVariant::Sage
                                on_click=Callback::new(move |_| {
                                    let id = sel.get_untracked();
                                    sent.update(|s| s.push(id));
                                })
                                disabled=Signal::derive(is_sent)
                            >
                                {move || {
                                    if is_sent() {
                                        view! { <Icon name="CheckCircle2" class="size-4"/> " Sent" }.into_any()
                                    } else {
                                        view! { <Icon name="Send" class="size-4"/> " Approve & send" }.into_any()
                                    }
                                }}
                            </Button>
                        </div>
                    </Card>

                    <Card>
                        <CardHeader title="Outbound campaigns" subtitle="Proactive messages that prevent inbound calls" icon=|| view! { <Icon name="Megaphone" class="size-4"/> }/>
                        <div class="grid gap-px bg-line sm:grid-cols-3">
                            {[
                                ("Pre-visit checklist", "72 h before visit: forms link, bring MRI discs, ID & referral", "−31% day-of reschedules"),
                                ("Imaging upload nudge", "New patients with outside imaging get a secure upload link", "64% upload before visit"),
                                ("Post-procedure check-in", "Day 1 & day 7 after injections or surgery, red-flag questions", "Early issues caught in 9 cases"),
                            ]
                                .into_iter()
                                .map(|(t, d, k)| {
                                    view! {
                                        <div class="bg-white p-4">
                                            <div class="text-[13px] font-semibold text-ink">{t}</div>
                                            <div class="mt-1 text-[12px] text-muted">{d}</div>
                                            <div class="mt-2 text-[12px] font-semibold text-sage">{k}</div>
                                        </div>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </Card>
                </div>
            </div>
        </div>
    }
}
