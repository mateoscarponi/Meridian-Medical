use crate::components::browser::sleep_ms;
use crate::components::icons::Icon;
use crate::components::shell::ToolHeader;
use crate::components::ui::*;
use crate::domain::util::{js_round, num};
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_json::{json, Value};

struct Doc {
    id: &'static str,
    source: &'static str,
    from: &'static str,
    received: &'static str,
    kind: &'static str,
    type_conf: f64,
    pages: u32,
    lines: &'static [&'static str],
    fields: &'static [(&'static str, &'static str, f64)],
    queue: &'static str,
    actions: &'static [&'static str],
}

static DOCS: [Doc; 5] = [
    Doc {
        id: "DOC-5521", source: "Fax", from: "(516) 555-0172 · Patel Family Medicine", received: "8:02 AM", kind: "Referral", type_conf: 0.98, pages: 2,
        lines: &["PATEL FAMILY MEDICINE — REFERRAL FORM", "Date: 11/13/2026        Urgency: Routine", "Patient: Carmen Diaz      DOB: 08/15/1972", "Insurance: Empire BCBS   Member ID: YRP451207733", "Referred to: Orthopedic Spine — Meridian Orthopedics & Spine", "Reason: Right sciatica x 5 weeks, failed NSAIDs", "Dx: M54.31 Sciatica, right side", "Referring: Rina Patel, MD   NPI 1888888801", "Please evaluate and treat. Notes attached."],
        fields: &[("Patient", "Carmen Diaz", 0.99), ("DOB", "08/15/1972", 0.99), ("Insurance", "Empire BCBS", 0.97), ("Member ID", "YRP451207733", 0.95), ("Reason", "Right sciatica x 5 weeks, failed NSAIDs", 0.93), ("Dx", "M54.31", 0.98), ("Referring", "Rina Patel, MD", 0.97)],
        queue: "Referrals → Scheduling",
        actions: &["Matched existing patient MOS-205518 (score 0.97)", "Eligibility verified (271: active, specialist copay $40)", "Referral created; offered 3 slots by SMS", "Loop-closure task set for referring office"],
    },
    Doc {
        id: "DOC-5522", source: "Fax", from: "Nassau Imaging Center", received: "8:17 AM", kind: "Imaging report (MRI)", type_conf: 0.99, pages: 3,
        lines: &["NASSAU IMAGING CENTER — MRI REPORT", "Patient: James O'Connor   DOB: 09/30/1975   MRN: MOS-198344", "Exam: MRI LUMBAR SPINE WITHOUT CONTRAST (CPT 72148)", "Date of exam: 11/15/2026", "IMPRESSION:", "1. Right paracentral disc extrusion at L4-L5 contacting", "   the descending right L5 nerve root.", "2. Moderate right lateral recess stenosis at L4-L5.", "Radiologist: K. Iyer, MD"],
        fields: &[("Patient", "James O'Connor", 0.99), ("MRN", "MOS-198344", 0.99), ("Exam", "MRI LUMBAR SPINE WITHOUT CONTRAST", 0.98), ("Date of exam", "11/15/2026", 0.99), ("Impression", "Right paracentral disc extrusion at L4-L5", 0.94), ("Radiologist", "K. Iyer, MD", 0.96)],
        queue: "Clinical inbox → Dr. Okafor",
        actions: &["Filed to chart as DiagnosticReport", "Attached to pending prior auth PA-24812 (criterion: imaging concordance ✓)", "Provider notified: result ready for review"],
    },
    Doc {
        id: "DOC-5523", source: "Mobile photo", from: "Patient upload · Aisha Rahman", received: "8:40 AM", kind: "Insurance card", type_conf: 0.97, pages: 1,
        lines: &["GEICO — PERSONAL INJURY PROTECTION", "Claim #: NF-77120934", "Insured: Aisha Rahman", "Date of loss: 09/20/2026", "Adjuster: M. Torres  (800) 555-0110", "Policy: 4410-22-889"],
        fields: &[("Payer", "GEICO", 0.98), ("Claim #", "NF-77120934", 0.97), ("Insured", "Aisha Rahman", 0.99), ("Date of loss", "09/20/2026", 0.96), ("Adjuster", "M. Torres", 0.92)],
        queue: "Billing → No-fault",
        actions: &["Coverage updated (No-Fault, claim # added)", "NF-3 billing deadline set: 45 days from each DOS", "Adjuster contact saved to the case"],
    },
    Doc {
        id: "DOC-5524", source: "Portal upload", from: "Payer portal · Humana", received: "9:05 AM", kind: "Authorization decision", type_conf: 0.96, pages: 1,
        lines: &["HUMANA — PRIOR AUTHORIZATION DETERMINATION", "Member: Linda Chen   ID: H5619-0042", "Service: 64483 Transforaminal ESI, lumbar", "Decision: APPROVED", "Auth #: HUM-AUTH-8841207", "Valid: 11/16/2026 – 02/14/2027   Units: 1"],
        fields: &[("Member", "Linda Chen", 0.99), ("Service", "64483", 0.99), ("Decision", "APPROVED", 0.99), ("Auth #", "HUM-AUTH-8841207", 0.98), ("Valid", "11/16/2026 – 02/14/2027", 0.97)],
        queue: "Authorizations → Scheduling",
        actions: &["Auth # written to the order and future claim (REF*G1)", "Procedure slot offered to patient", "Prior-auth case closed: 2.4 days end to end"],
    },
    Doc {
        id: "DOC-5525", source: "Email", from: "Law office · records request", received: "9:31 AM", kind: "Records request (legal)", type_conf: 0.91, pages: 2,
        lines: &["RE: Request for medical records — HIPAA authorization enclosed", "Patient: Robert Williams   DOB: 06/02/1981", "Records requested: 08/01/2026 to present", "Purpose: Workers' compensation hearing", "Signed authorization: YES (page 2)"],
        fields: &[("Patient", "Robert Williams", 0.99), ("Date range", "08/01/2026 to present", 0.95), ("Purpose", "Workers' compensation hearing", 0.93), ("Authorization", "YES", 0.9)],
        queue: "Release of information → Human review",
        actions: &["Authorization validity check: signature & expiration found", "Minimum-necessary packet assembled (WC encounters only)", "Held for staff approval before release (policy)"],
    },
];

fn fhir_of(id: &str) -> Value {
    match id {
        "DOC-5521" => json!({ "resourceType": "ServiceRequest", "status": "active", "intent": "order", "category": [{ "text": "Referral" }], "code": { "text": "Orthopedic spine evaluation" }, "reasonCode": [{ "coding": [{ "system": "http://hl7.org/fhir/sid/icd-10-cm", "code": "M54.31" }] }], "subject": { "reference": "Patient/pt-1007" }, "requester": { "display": "Rina Patel, MD", "identifier": { "system": "http://hl7.org/fhir/sid/us-npi", "value": "1888888801" } } }),
        "DOC-5522" => json!({ "resourceType": "DiagnosticReport", "status": "final", "code": { "coding": [{ "system": "http://www.ama-assn.org/go/cpt", "code": "72148" }] }, "subject": { "reference": "Patient/pt-1002" }, "effectiveDateTime": "2026-11-15", "conclusion": "Right paracentral disc extrusion at L4-L5 contacting the descending right L5 nerve root." }),
        "DOC-5523" => json!({ "resourceType": "Coverage", "status": "active", "type": { "text": "No-Fault (PIP)" }, "subscriberId": "NF-77120934", "beneficiary": { "reference": "Patient/pt-1005" }, "payor": [{ "display": "GEICO" }] }),
        "DOC-5524" => json!({ "resourceType": "ClaimResponse", "status": "active", "use": "preauthorization", "outcome": "complete", "preAuthRef": "HUM-AUTH-8841207", "preAuthPeriod": { "start": "2026-11-16", "end": "2027-02-14" }, "patient": { "reference": "Patient/pt-1003" } }),
        _ => json!({ "resourceType": "Task", "status": "requested", "intent": "order", "code": { "text": "Release of information" }, "for": { "reference": "Patient/pt-1004" }, "businessStatus": { "text": "Awaiting staff approval" } }),
    }
}

fn source_icon(s: &str) -> &'static str {
    match s {
        "Fax" => "Printer",
        "Portal upload" => "Upload",
        "Email" => "Inbox",
        _ => "Smartphone",
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum St {
    Queued,
    Reading,
    Done,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Fields,
    Fhir,
}

fn doc(id: &str) -> &'static Doc {
    DOCS.iter().find(|d| d.id == id).unwrap_or(&DOCS[0])
}

fn pct(x: f64) -> String {
    num(js_round(x * 100.0))
}

#[component]
pub fn DocumentsPage() -> impl IntoView {
    let sel = RwSignal::new(DOCS[0].id);
    let status = RwSignal::new([St::Queued; 5]);
    let view_mode = RwSignal::new(ViewMode::Fields);
    let busy = StoredValue::new(false);
    let idx = |id: &str| DOCS.iter().position(|d| d.id == id).unwrap_or(0);
    let d = move || doc(sel.get());
    let st = move || status.with(|s| s[idx(sel.get())]);
    let processed = move || status.with(|s| s.iter().filter(|x| **x == St::Done).count());

    let process_one = move |id: &'static str| async move {
        status.update(|s| s[idx(id)] = St::Reading);
        sleep_ms(1500).await;
        status.update(|s| s[idx(id)] = St::Done);
    };
    let process_all = move || {
        if busy.get_value() {
            return;
        }
        busy.set_value(true);
        let snapshot = status.get_untracked();
        spawn_local(async move {
            for (i, d) in DOCS.iter().enumerate() {
                if snapshot[i] == St::Done {
                    continue;
                }
                sel.set(d.id);
                process_one(d.id).await;
            }
            busy.set_value(false);
        });
    };

    // "2 fax, 1 mobile photo, ..." in first-seen order.
    let mut summary: Vec<(&str, usize)> = Vec::new();
    for x in DOCS.iter() {
        match summary.iter_mut().find(|(k, _)| *k == x.source) {
            Some(e) => e.1 += 1,
            None => summary.push((x.source, 1)),
        }
    }
    let summary = summary.iter().map(|(k, v)| format!("{v} {}", k.to_lowercase())).collect::<Vec<_>>().join(", ");

    let highlight_line = move |line: &'static str| -> AnyView {
        let dd = d();
        if st() != St::Done {
            return line.into_any();
        }
        let Some((label, value, conf)) = dd.fields.iter().find(|f| line.contains(f.1)) else { return line.into_any() };
        let i = line.find(value).unwrap_or(0);
        view! {
            {&line[..i]}
            <span class="rounded-[3px] bg-ocean/10 outline outline-[1.5px] outline-ocean/70" title=format!("{label} · {}%", pct(*conf))>{*value}</span>
            {&line[i + value.len()..]}
        }
        .into_any()
    };

    view! {
        <div class="mx-auto max-w-7xl">
            <ToolHeader
                eyebrow="Clinical AI · OCR / Document AI"
                title="Document AI Inbox"
                description="Every fax, upload and email gets read, classified, matched to the right patient, turned into structured data and routed to the right queue. Staff review exceptions instead of retyping documents."
                tags=&["OCR", "document classification", "key-value extraction", "patient matching", "FHIR write-back"]
                impact=&[("−44 h", "intake hours / week"), ("96%", "straight-through rate")]
            />

            <div class="grid gap-6 xl:grid-cols-[320px_1fr]">
                <Card class="h-fit overflow-hidden">
                    <CardHeader
                        title="Incoming today"
                        subtitle=Signal::derive(move || format!("{} of {} processed · {}", processed(), DOCS.len(), summary))
                        right=move || {
                            view! {
                                <Button size=BtnSize::Sm on_click=Callback::new(move |_| process_all()) disabled=Signal::derive(move || processed() == DOCS.len())>
                                    <Icon name="Wand2" class="size-3.5"/>
                                    " Process all"
                                </Button>
                            }
                        }
                    />
                    <ul class="divide-y divide-line">
                        {DOCS
                            .iter()
                            .enumerate()
                            .map(|(i, x)| {
                                let s = move || status.with(|s| s[i]);
                                view! {
                                    <li>
                                        <button
                                            on:click=move |_| sel.set(x.id)
                                            class=move || {
                                                cx(&[
                                                    "flex w-full items-start gap-3 px-4 py-3.5 text-left transition",
                                                    if sel.get() == x.id { "bg-sky-2 shadow-[inset_3px_0_0_var(--color-ocean)]" } else { "hover:bg-cream" },
                                                ])
                                            }
                                        >
                                            <div class="mt-0.5 grid size-8 shrink-0 place-items-center rounded-lg bg-sand text-muted">
                                                <Icon name=source_icon(x.source) class="size-4"/>
                                            </div>
                                            <div class="min-w-0 flex-1">
                                                <div class="flex items-center justify-between gap-2">
                                                    <span class="truncate text-[13px] font-semibold text-ink">
                                                        {move || if s() == St::Done { x.kind } else { "Unclassified document" }}
                                                    </span>
                                                    {move || match s() {
                                                        St::Queued => view! { <Badge>"Queued"</Badge> }.into_any(),
                                                        St::Reading => view! { <Badge tone=Tone::Ocean><Icon name="Loader2" class="size-3 animate-spin"/> " Reading"</Badge> }.into_any(),
                                                        St::Done => view! { <Badge tone=Tone::Sage>"Routed"</Badge> }.into_any(),
                                                    }}
                                                </div>
                                                <div class="truncate text-[12px] text-muted">{x.from}</div>
                                                <div class="text-[11px] text-faint">{x.source} " · " {x.pages} " pg · " {x.received}</div>
                                            </div>
                                        </button>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                </Card>

                <div class="grid gap-6 lg:grid-cols-[1fr_1fr]">
                    <Card class="overflow-hidden">
                        <CardHeader
                            title=Signal::derive(move || d().id.to_string())
                            subtitle=Signal::derive(move || format!("{} from {}", d().source, d().from))
                            icon=|| view! { <Icon name="FileScan" class="size-4"/> }
                            right=move || {
                                move || {
                                    (st() == St::Queued)
                                        .then(|| {
                                            view! {
                                                <Button size=BtnSize::Sm on_click=Callback::new(move |_| spawn_local(process_one(sel.get_untracked())))>
                                                    <Icon name="ScanLine" class="size-3.5"/>
                                                    " Read"
                                                </Button>
                                            }
                                        })
                                }
                            }
                        />
                        <div class="bg-sand/70 p-5">
                            <div class="grain relative mx-auto max-w-md -rotate-[0.6deg] overflow-hidden rounded-sm bg-[#fdfcf8] p-6 font-mono text-[11.5px] leading-6 text-[#2a2f36] shadow-[0_10px_30px_-12px_rgb(15_30_46/0.35)]">
                                <Show when=move || st() == St::Reading>
                                    <div class="pointer-events-none absolute inset-x-0 h-10 animate-scan bg-gradient-to-b from-transparent via-ocean/25 to-transparent"></div>
                                </Show>
                                <div class="mb-3 flex justify-between text-[9px] uppercase tracking-widest text-[#8a8f96]">
                                    <span>{move || if d().source == "Fax" { format!("FAX · {}", d().received) } else { d().source.to_string() }}</span>
                                    <span>"Page 1/" {move || d().pages}</span>
                                </div>
                                {move || {
                                    d()
                                        .lines
                                        .iter()
                                        .enumerate()
                                        .map(|(i, l)| {
                                            view! { <div class=cx(&["whitespace-pre-wrap", if i == 0 { "mb-2 font-bold" } else { "" }])>{highlight_line(l)}</div> }
                                        })
                                        .collect_view()
                                }}
                            </div>
                        </div>
                    </Card>

                    <Card class="flex flex-col">
                        <CardHeader
                            title=Signal::derive(move || if st() == St::Done { d().kind.to_string() } else { "Awaiting processing".to_string() })
                            subtitle=Signal::derive(move || {
                                if st() == St::Done { format!("Classified with {}% confidence", pct(d().type_conf)) } else { "Run Read or Process all".to_string() }
                            })
                            right=move || {
                                move || (st() == St::Done).then(|| view! { <Segmented value=view_mode options=vec![(ViewMode::Fields, "Fields"), (ViewMode::Fhir, "FHIR")]/> })
                            }
                        />
                        {move || {
                            let s = st();
                            let dd = d();
                            if s != St::Done {
                                view! {
                                    <div class="grid flex-1 place-items-center p-10 text-center text-[13px] text-muted">
                                        {if s == St::Reading {
                                            view! {
                                                <div>
                                                    <Icon name="Loader2" class="mx-auto size-7 animate-spin text-ocean"/>
                                                    <p class="mt-3">"OCR → layout analysis → classification → extraction…"</p>
                                                </div>
                                            }
                                                .into_any()
                                        } else {
                                            view! {
                                                <div>
                                                    <Icon name="ScanLine" class="mx-auto size-8 text-line-2"/>
                                                    <p class="mt-3">"Not processed yet."</p>
                                                </div>
                                            }
                                                .into_any()
                                        }}
                                    </div>
                                }
                                    .into_any()
                            } else if view_mode.get() == ViewMode::Fhir {
                                view! {
                                    <div class="animate-fade-up p-4">
                                        <CodeBlock value=pretty_json(&fhir_of(dd.id)) max_h="max-h-[420px]"/>
                                    </div>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <div class="animate-fade-up flex-1">
                                        <table class="w-full text-[13px]">
                                            <tbody class="divide-y divide-line">
                                                {dd
                                                    .fields
                                                    .iter()
                                                    .map(|(label, value, conf)| {
                                                        view! {
                                                            <tr>
                                                                <td class="w-32 px-5 py-2.5 font-semibold text-muted">{*label}</td>
                                                                <td class="px-2 py-2.5 text-ink">{*value}</td>
                                                                <td class="px-5 py-2.5 text-right">
                                                                    <span class=cx(&["font-mono text-[11px]", if *conf >= 0.95 { "text-sage" } else { "text-warn" }])>{pct(*conf)} "%"</span>
                                                                </td>
                                                            </tr>
                                                        }
                                                    })
                                                    .collect_view()}
                                            </tbody>
                                        </table>
                                        <div class="border-t border-line p-5">
                                            <div class="flex items-center gap-2 text-[13px] font-semibold text-ink">
                                                <Icon name="ArrowRight" class="size-4 text-ocean"/>
                                                " Routed to "
                                                {dd.queue}
                                            </div>
                                            <ul class="mt-3 space-y-2">
                                                {dd
                                                    .actions
                                                    .iter()
                                                    .map(|a| {
                                                        view! {
                                                            <li class="flex gap-2 text-[12.5px] text-muted">
                                                                <Icon name="CheckCircle2" class="mt-0.5 size-3.5 shrink-0 text-sage"/>
                                                                {*a}
                                                            </li>
                                                        }
                                                    })
                                                    .collect_view()}
                                            </ul>
                                        </div>
                                    </div>
                                }
                                    .into_any()
                            }
                        }}
                    </Card>
                </div>
            </div>
        </div>
    }
}
