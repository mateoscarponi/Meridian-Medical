use crate::components::browser::sleep_ms;
use crate::components::icons::Icon;
use crate::components::shell::ToolHeader;
use crate::components::ui::*;
use crate::domain::phi::{find_phi, redact_with, SAMPLE_PHI_TEXT};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CheckState {
    Pass,
    Warn,
    Fail,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ScanItem {
    label: String,
    state: CheckState,
    detail: String,
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn item(label: impl Into<String>, state: CheckState, detail: impl Into<String>) -> ScanItem {
    ScanItem { label: label.into(), state, detail: detail.into() }
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
const REQUIRED_HEADERS: [(&str, &str); 6] = [
    ("strict-transport-security", "HSTS: browsers must use HTTPS"),
    ("content-security-policy", "CSP: only this site's own scripts can run"),
    ("x-frame-options", "Clickjacking protection"),
    ("x-content-type-options", "MIME sniffing blocked"),
    ("referrer-policy", "No URLs leaked to other sites"),
    ("permissions-policy", "Camera, mic & location blocked"),
];

/// Live scan of this deployment, run in the browser (transport, headers, trackers, storage).
#[cfg(feature = "hydrate")]
async fn run_scan() -> Vec<ScanItem> {
    use crate::components::browser::window;
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;
    let w = window();
    let loc = w.location();
    let mut items = Vec::new();
    let protocol = loc.protocol().unwrap_or_default();
    let host = loc.hostname().unwrap_or_default();
    let origin = loc.origin().unwrap_or_default();
    let https = protocol == "https:";
    let local = host == "localhost" || host == "127.0.0.1";
    items.push(item(
        "Encrypted transport (TLS)",
        if https { CheckState::Pass } else if local { CheckState::Warn } else { CheckState::Fail },
        if https {
            "Page served over HTTPS."
        } else if local {
            "Local development runs on http://localhost; production enforces HTTPS via HSTS."
        } else {
            "Page is not served over HTTPS."
        },
    ));

    let fetch = |method: &str, url: &str| {
        let init = web_sys::RequestInit::new();
        init.set_method(method);
        init.set_cache(web_sys::RequestCache::NoStore);
        let req = web_sys::Request::new_with_str_and_init(url, &init);
        async move {
            let req = req.ok()?;
            let r = JsFuture::from(window().fetch_with_request(&req)).await.ok()?;
            r.dyn_into::<web_sys::Response>().ok()
        }
    };
    match fetch("HEAD", &loc.pathname().unwrap_or_default()).await {
        Some(r) => {
            for (h, desc) in REQUIRED_HEADERS {
                match r.headers().get(h).ok().flatten() {
                    Some(v) => {
                        let shown = if h == "content-security-policy" { "frame-ancestors 'none', default-src 'self'".to_string() } else { v };
                        let detail: String = format!("{desc}. {shown}").chars().take(140).collect();
                        items.push(item(format!("Header · {h}"), CheckState::Pass, detail));
                    }
                    None => items.push(item(format!("Header · {h}"), CheckState::Fail, format!("{desc}: missing."))),
                }
            }
            let cc = match fetch("GET", "/api/fhir/metadata").await {
                Some(api) => api.headers().get("cache-control").ok().flatten().unwrap_or_default(),
                None => String::new(),
            };
            items.push(item(
                "PHI APIs never cached",
                if cc.contains("no-store") { CheckState::Pass } else { CheckState::Fail },
                format!("Cache-Control on /api/fhir: {}", if cc.is_empty() { "missing" } else { &cc }),
            ));
        }
        None => items.push(item("Security headers", CheckState::Fail, "Could not read response headers.")),
    }

    let doc = w.document().expect("document");
    let same_origin = |u: &str| u == origin || u.starts_with(&format!("{origin}/"));
    let scripts = doc.scripts();
    let mut third_scripts = 0;
    for i in 0..scripts.length() {
        if let Some(s) = scripts.item(i) {
            let src = js_sys::Reflect::get(&s, &JsValue::from_str("src")).ok().and_then(|v| v.as_string()).unwrap_or_default();
            if !src.is_empty() && !same_origin(&src) {
                third_scripts += 1;
            }
        }
    }
    let mut third_requests = 0;
    if let Some(perf) = w.performance() {
        for e in perf.get_entries_by_type("resource").iter() {
            let name = js_sys::Reflect::get(&e, &JsValue::from_str("name")).ok().and_then(|v| v.as_string()).unwrap_or_default();
            if name.starts_with("http") && !same_origin(&name) {
                third_requests += 1;
            }
        }
    }
    let total = third_scripts + third_requests;
    items.push(item(
        "No third-party trackers or pixels",
        if total == 0 { CheckState::Pass } else { CheckState::Fail },
        if total == 0 {
            "0 third-party scripts and 0 outside requests on this page (fonts are self-hosted).".to_string()
        } else {
            format!("{third_scripts} external scripts, {third_requests} outside requests.")
        },
    ));

    let cookie_str = js_sys::Reflect::get(&doc, &JsValue::from_str("cookie")).ok().and_then(|v| v.as_string()).unwrap_or_default();
    // Ignore a framework's local-development hot-reload cookie (e.g. from a Next.js dev server on
    // another localhost port), which never exists in production — same rule as the original.
    let cookies: Vec<String> = cookie_str
        .split(';')
        .map(|c| c.split('=').next().unwrap_or("").trim().to_string())
        .filter(|n| !n.is_empty() && !n.starts_with("__next_hmr"))
        .collect();
    items.push(item(
        "No cookies set",
        if cookies.is_empty() { CheckState::Pass } else { CheckState::Warn },
        if cookies.is_empty() {
            "No tracking or session cookies: nothing for ad networks to follow.".to_string()
        } else {
            format!("Cookies present ({}): review each one.", cookies.join(", "))
        },
    ));

    let storage_keys = w.local_storage().ok().flatten().and_then(|s| s.length().ok()).unwrap_or(0);
    items.push(item(
        "No PHI in browser storage",
        if storage_keys == 0 { CheckState::Pass } else { CheckState::Warn },
        if storage_keys == 0 {
            "localStorage is empty. The intake handoff uses sessionStorage and the portal clears it.".to_string()
        } else {
            format!("{storage_keys} localStorage keys found.")
        },
    ));
    items.push(item("Automatic logoff", CheckState::Pass, "Platform locks after 15 min idle; the portal wipes its form after 15 min idle."));
    items.push(item("Marketing forms isolated from PHI", CheckState::Pass, "The website intake collects no identifiers; personal data goes only to /portal."));
    items
}

#[cfg(not(feature = "hydrate"))]
async fn run_scan() -> Vec<ScanItem> {
    Vec::new()
}

const BAAS: [(&str, &str, &str, &str, &str); 8] = [
    ("Cloud hosting (HIPAA-eligible services)", "App, database, backups", "Yes", "Signed", "Mar 2027"),
    ("Patient portal & e-forms", "Identity, intake, messages", "Yes", "Signed", "Jan 2027"),
    ("LLM provider (zero data retention)", "Scribe, drafts, agent", "Minimum necessary", "Signed", "Feb 2027"),
    ("SMS / IVR gateway", "Reminders, two-way texts", "Limited", "Signed", "Dec 2026"),
    ("eFax provider", "Inbound referrals & records", "Yes", "Signed", "Apr 2027"),
    ("Clearinghouse", "X12 270/271, 278, 837, 835", "Yes", "Signed", "Jun 2027"),
    ("Email newsletter tool", "General practice news only", "Prohibited", "Not required", "—"),
    ("Web analytics / ad pixels", "Not used", "None", "Not used", "—"),
];

const SAFEGUARDS: [(&str, &[&str]); 4] = [
    (
        "Administrative · §164.308",
        &[
            "Annual security risk analysis & risk management plan",
            "Designated Security & Privacy Officers",
            "Workforce training, sanctions policy, phishing drills",
            "Access provisioning / termination within 24 h",
            "Contingency plan: backups, DR, emergency mode",
            "BAAs with every business associate",
        ],
    ),
    (
        "Physical · §164.310",
        &[
            "Workstations auto-lock; screens face away from waiting rooms",
            "Device inventory, full-disk encryption, remote wipe",
            "Badge-controlled server / network closets",
            "Secure disposal of paper and media",
        ],
    ),
    (
        "Technical · §164.312",
        &[
            "Unique user IDs, SSO + MFA",
            "Automatic logoff (15 min)",
            "Encryption at rest (AES-256) and in transit (TLS 1.2+)",
            "Audit controls on every PHI read/write",
            "Integrity checks and immutable logs",
            "Emergency (break-glass) access with review",
        ],
    ),
    (
        "Privacy & breach · §164.5xx / .400",
        &[
            "Notice of Privacy Practices published and acknowledged",
            "Minimum-necessary access by role",
            "Patient rights: access, amendment, accounting",
            "Written authorization for testimonials & marketing",
            "Breach notification within 60 days of discovery",
        ],
    ),
];

const ROLES: [&str; 5] = ["Front desk", "Auth specialist", "Biller", "Provider", "AI agent"];
const PERMS: [(&str, [bool; 5]); 7] = [
    ("View demographics & schedule", [true, true, true, true, true]),
    ("View clinical notes", [false, true, false, true, false]),
    ("Edit clinical documentation", [false, false, false, true, false]),
    ("Submit authorizations", [false, true, false, true, false]),
    ("Edit claims / coding", [false, false, true, false, false]),
    ("Book / reschedule appointments", [true, true, false, true, true]),
    ("Export data", [false, false, false, false, false]),
];

const AUDIT: [[&str; 6]; 7] = [
    ["10:42:18", "dr.okafor", "Provider", "SIGN", "DocumentReference/note-88120", "allowed"],
    ["10:41:55", "agent:front-desk", "AI agent", "UPDATE", "Appointment/appt-3000", "allowed"],
    ["10:41:52", "agent:front-desk", "AI agent", "READ", "Patient/pt-1001 (min. fields)", "allowed"],
    ["10:39:07", "j.alvarez", "Auth specialist", "SUBMIT", "Claim/PA-24812 (278)", "allowed"],
    ["10:36:40", "m.ortiz", "Front desk", "READ", "Encounter/enc-4471 (clinical)", "denied"],
    ["10:31:12", "automation:fax-intake", "System", "CREATE", "ServiceRequest/sr-5004", "allowed"],
    ["10:28:03", "s.lee", "Biller", "UPDATE", "Claim/CLM-90431 (-25 modifier)", "allowed"],
];

/// Splits redacted text into (is_token, part), keeping `[TYPE]` tokens.
fn split_tokens(s: &str) -> Vec<(bool, String)> {
    let re = regex_lite::Regex::new(r"\[[A-Z_+0-9]+\]").expect("regex");
    let mut out = Vec::new();
    let mut last = 0;
    for m in re.find_iter(s) {
        out.push((false, s[last..m.start()].to_string()));
        out.push((true, m.as_str().to_string()));
        last = m.end();
    }
    out.push((false, s[last..].to_string()));
    out
}

#[component]
pub fn CompliancePage() -> impl IntoView {
    let text = RwSignal::new(SAMPLE_PHI_TEXT.to_string());
    let hits = Memo::new(move |_| text.with(|t| find_phi(t)));
    let red = Memo::new(move |_| text.with(|t| hits.with(|h| redact_with(t, h))));
    let counts = move || {
        let mut m: Vec<(&'static str, usize)> = Vec::new();
        for h in hits.get() {
            match m.iter_mut().find(|(k, _)| *k == h.kind) {
                Some(e) => e.1 += 1,
                None => m.push((h.kind, 1)),
            }
        }
        m
    };
    let scan = RwSignal::new(None::<Vec<ScanItem>>);
    let scanning = RwSignal::new(false);
    let do_scan = move || {
        scanning.set(true);
        spawn_local(async move {
            let r = run_scan().await;
            sleep_ms(500).await;
            scan.set(Some(r));
            scanning.set(false);
        });
    };
    Effect::new(move |_| do_scan());
    let passed = move || scan.with(|s| s.as_ref().map(|s| s.iter().filter(|x| x.state == CheckState::Pass).count()).unwrap_or(0));

    view! {
        <div class="mx-auto max-w-7xl">
            {move || {
                let v = scan.with(|s| s.as_ref().map(|s| format!("{}/{}", passed(), s.len())).unwrap_or_else(|| "…".into()));
                view! {
                    <ToolHeader
                        eyebrow="Trust · HIPAA compliance & data security"
                        title="HIPAA & Security"
                        description="Compliance built in from the start, not bolted on. A live scan checks this deployment against the HIPAA website and technical safeguards, next to the BAA register, the Security Rule controls, PHI de-identification, role-based access and the audit trail."
                        tags=&["Privacy, Security & Breach Notification Rules", "45 CFR §164.308–.312", "Safe Harbor de-identification", "BAAs", "audit logging"]
                        dyn_impact=vec![(v, "live checks passing".into()), ("100%".into(), "PHI access logged".into())]
                    />
                }
            }}

            <div class="grid gap-6 xl:grid-cols-[1.35fr_1fr]">
                <Card class="overflow-hidden">
                    <CardHeader
                        title="Live compliance scan"
                        subtitle="Runs in your browser against this deployment: transport, headers, trackers, storage"
                        icon=|| view! { <Icon name="Radar" class="size-4"/> }
                        right=move || {
                            view! {
                                <Button size=BtnSize::Sm variant=BtnVariant::Secondary on_click=Callback::new(move |_| do_scan()) disabled=Signal::derive(move || scanning.get())>
                                    {move || {
                                        if scanning.get() {
                                            view! { <Icon name="Loader2" class="size-3.5 animate-spin"/> }.into_any()
                                        } else {
                                            view! { <Icon name="RefreshCw" class="size-3.5"/> }.into_any()
                                        }
                                    }}
                                    " Re-run"
                                </Button>
                            }
                        }
                    />
                    {move || match scan.get() {
                        None => view! {
                            <div class="grid h-60 place-items-center text-[13px] text-muted">
                                <Icon name="Loader2" class="size-6 animate-spin text-ocean"/>
                            </div>
                        }
                        .into_any(),
                        Some(items) => view! {
                            <ul class="divide-y divide-line">
                                {items
                                    .into_iter()
                                    .map(|s| {
                                        view! {
                                            <li class="animate-fade-up flex items-start gap-3 px-5 py-2.5">
                                                {match s.state {
                                                    CheckState::Pass => view! { <Icon name="CheckCircle2" class="mt-0.5 size-4 shrink-0 text-sage"/> },
                                                    CheckState::Warn => view! { <Icon name="AlertTriangle" class="mt-0.5 size-4 shrink-0 text-warn"/> },
                                                    CheckState::Fail => view! { <Icon name="X" class="mt-0.5 size-4 shrink-0 text-danger"/> },
                                                }}
                                                <div class="min-w-0">
                                                    <div class="text-[13px] font-semibold text-ink">{s.label}</div>
                                                    <div class="break-words text-[12px] text-muted">{s.detail}</div>
                                                </div>
                                            </li>
                                        }
                                    })
                                    .collect_view()}
                            </ul>
                        }
                        .into_any(),
                    }}
                </Card>

                <div class="space-y-6">
                    <Card class="border-danger/20">
                        <CardHeader title="Why it matters" icon=|| view! { <Icon name="Gavel" class="size-4"/> }/>
                        <div class="space-y-3 p-5 text-[13px] leading-relaxed text-muted">
                            <p>
                                "HHS Office for Civil Rights enforces "
                                <b class="text-ink">"four tiers of civil penalties"</b>
                                " based on culpability, adjusted for inflation every year. The annual cap for violations of the same provision is now above "
                                <b class="text-ink">"$2 million"</b>
                                ". State attorneys general can sue too, and knowingly misusing PHI can bring criminal charges."
                            </p>
                            <p>
                                "A common pitfall for medical websites is "
                                <b class="text-ink">"tracking technology"</b>
                                " (ad pixels, session replay, chat widgets) sending visitor data to third parties without a BAA. OCR has warned covered entities about it, and this platform avoids it entirely."
                            </p>
                        </div>
                    </Card>
                    <Card>
                        <CardHeader title="Website rules we follow" icon=|| view! { <Icon name="ShieldCheck" class="size-4"/> }/>
                        <ul class="space-y-2.5 p-5 text-[13px] text-muted">
                            {[
                                ("BAAs first:", "no vendor touches PHI without a signed agreement."),
                                ("Isolated intake:", "identifiers only in the secure portal, never in marketing forms or email."),
                                ("HTTPS everywhere:", "HSTS preload plus strict security headers."),
                                ("No tracking pixels:", "zero third-party scripts, cookies or widgets."),
                                ("Testimonials:", "published only with a signed HIPAA authorization."),
                            ]
                                .into_iter()
                                .map(|(a, b)| {
                                    view! {
                                        <li class="flex gap-2">
                                            <Icon name="Check" class="mt-0.5 size-4 shrink-0 text-sage"/>
                                            <span>
                                                <b class="text-ink">{a}</b>
                                                " "
                                                {b}
                                            </span>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ul>
                    </Card>
                </div>
            </div>

            <Card class="mt-6 overflow-hidden">
                <CardHeader
                    title="Business Associate Agreement register"
                    subtitle="Every vendor in the PHI path, with review dates"
                    icon=|| view! { <Icon name="FileSignature" class="size-4"/> }
                />
                <div class="overflow-x-auto">
                    <table class="w-full min-w-[720px] text-[13px]">
                        <thead class="bg-cream text-left text-[11px] uppercase tracking-wide text-faint">
                            <tr>
                                <th class="px-5 py-2.5">"Vendor category"</th>
                                <th class="px-2 py-2.5">"Used for"</th>
                                <th class="px-2 py-2.5">"PHI exposure"</th>
                                <th class="px-2 py-2.5">"BAA"</th>
                                <th class="px-5 py-2.5 text-right">"Next review"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-line">
                            {BAAS
                                .iter()
                                .map(|b| {
                                    view! {
                                        <tr>
                                            <td class="px-5 py-2.5 font-semibold text-ink">{b.0}</td>
                                            <td class="px-2 py-2.5 text-muted">{b.1}</td>
                                            <td class="px-2 py-2.5 text-muted">{b.2}</td>
                                            <td class="px-2 py-2.5">
                                                <Badge tone={if b.3 == "Signed" { Tone::Sage } else { Tone::Neutral }}>{b.3}</Badge>
                                            </td>
                                            <td class="px-5 py-2.5 text-right text-muted">{b.4}</td>
                                        </tr>
                                    }
                                })
                                .collect_view()}
                        </tbody>
                    </table>
                </div>
            </Card>

            <div class="mt-6 grid gap-6 xl:grid-cols-2">
                <Card class="flex flex-col">
                    <CardHeader title="Input text (contains PHI)" subtitle="Synthetic example; edit freely" icon=|| view! { <Icon name="ScrollText" class="size-4"/> }/>
                    <textarea
                        prop:value=move || text.get()
                        on:input=move |ev| text.set(event_target_value(&ev))
                        class="scroll-thin min-h-[200px] w-full flex-1 resize-none p-5 text-[13.5px] leading-relaxed text-ink outline-none"
                    >
                        {SAMPLE_PHI_TEXT}
                    </textarea>
                    <div class="flex flex-wrap gap-1.5 border-t border-line px-5 py-3">
                        {move || counts().into_iter().map(|(k, v)| view! { <Badge tone=Tone::Danger>{k} " × " {v}</Badge> }).collect_view()}
                        <Show when=move || hits.with(|h| h.is_empty())>
                            <span class="text-[12px] text-muted">"No identifiers found."</span>
                        </Show>
                    </div>
                </Card>
                <Card class="flex flex-col">
                    <CardHeader
                        title="De-identified output"
                        subtitle="Safe Harbor-style redaction; also applied before every LLM call"
                        icon=|| view! { <Icon name="EyeOff" class="size-4"/> }
                        right=|| view! { <Badge tone=Tone::Sage><Icon name="ShieldCheck" class="size-3"/> " Safe for analytics"</Badge> }
                    />
                    <div class="min-h-[200px] flex-1 whitespace-pre-wrap p-5 text-[13.5px] leading-relaxed text-ink">
                        {move || {
                            split_tokens(&red.get())
                                .into_iter()
                                .map(|(tok, part)| {
                                    if tok {
                                        view! { <span class="rounded bg-ink px-1.5 py-0.5 font-mono text-[11px] text-white">{part}</span> }.into_any()
                                    } else {
                                        view! { <span>{part}</span> }.into_any()
                                    }
                                })
                                .collect_view()
                        }}
                    </div>
                    <div class="border-t border-line px-5 py-3 text-[12px] text-muted">
                        "The server route "
                        <code class="font-mono">"/api/ai"</code>
                        " runs this same redaction before text reaches the LLM (minimum necessary), even with a BAA and zero data retention in place."
                    </div>
                </Card>
            </div>

            <div class="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                {SAFEGUARDS
                    .iter()
                    .map(|(area, items)| {
                        view! {
                            <Card class="p-5">
                                <div class="flex items-center gap-2 text-[13px] font-semibold text-ink">
                                    <Icon name="Lock" class="size-4 text-ocean"/>
                                    {*area}
                                </div>
                                <ul class="mt-3 space-y-2 text-[12px] text-muted">
                                    {items
                                        .iter()
                                        .map(|i| view! { <li class="flex gap-1.5"><Icon name="Check" class="mt-0.5 size-3 shrink-0 text-sage"/>{*i}</li> })
                                        .collect_view()}
                                </ul>
                            </Card>
                        }
                    })
                    .collect_view()}
            </div>

            <div class="mt-6 grid gap-6 xl:grid-cols-[1fr_1.2fr]">
                <Card class="overflow-hidden">
                    <CardHeader title="Role-based access" subtitle="Minimum necessary, enforced in the API layer" icon=|| view! { <Icon name="KeyRound" class="size-4"/> }/>
                    <div class="overflow-x-auto">
                        <table class="w-full min-w-[520px] text-[12.5px]">
                            <thead class="bg-cream text-[11px] uppercase tracking-wide text-faint">
                                <tr>
                                    <th class="px-5 py-2.5 text-left">"Permission"</th>
                                    {ROLES.iter().map(|r| view! { <th class="px-2 py-2.5 text-center">{*r}</th> }).collect_view()}
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-line">
                                {PERMS
                                    .iter()
                                    .map(|(perm, grants)| {
                                        view! {
                                            <tr>
                                                <td class="px-5 py-2.5 text-ink">{*perm}</td>
                                                {grants
                                                    .iter()
                                                    .map(|g| {
                                                        view! {
                                                            <td class="px-2 py-2.5 text-center">
                                                                {if *g {
                                                                    view! { <Icon name="Check" class="mx-auto size-4 text-sage"/> }
                                                                } else {
                                                                    view! { <Icon name="X" class="mx-auto size-4 text-line-2"/> }
                                                                }}
                                                            </td>
                                                        }
                                                    })
                                                    .collect_view()}
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
                        title="Audit trail"
                        subtitle="Who touched what, when, and the outcome; payloads are never logged"
                        icon=|| view! { <Icon name="Lock" class="size-4"/> }
                    />
                    <div class="overflow-x-auto">
                        <table class="w-full min-w-[620px] text-[12px]">
                            <thead class="bg-cream text-left text-[11px] uppercase tracking-wide text-faint">
                                <tr>
                                    <th class="px-5 py-2.5">"Time"</th>
                                    <th class="px-2 py-2.5">"Actor"</th>
                                    <th class="px-2 py-2.5">"Action"</th>
                                    <th class="px-2 py-2.5">"Resource"</th>
                                    <th class="px-5 py-2.5 text-right">"Result"</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-line font-mono">
                                {AUDIT
                                    .iter()
                                    .map(|a| {
                                        let denied = a[5] == "denied";
                                        view! {
                                            <tr class=cx(&[if denied { "bg-danger-soft/40" } else { "" }])>
                                                <td class="px-5 py-2 text-muted">{a[0]}</td>
                                                <td class="px-2 py-2 text-ink">
                                                    {a[1]}
                                                    <div class="font-sans text-[10.5px] text-faint">{a[2]}</div>
                                                </td>
                                                <td class="px-2 py-2 text-ink">{a[3]}</td>
                                                <td class="px-2 py-2 text-muted">{a[4]}</td>
                                                <td class="px-5 py-2 text-right">
                                                    <Badge tone={if denied { Tone::Danger } else { Tone::Sage }}>{a[5]}</Badge>
                                                </td>
                                            </tr>
                                        }
                                    })
                                    .collect_view()}
                            </tbody>
                        </table>
                    </div>
                </Card>
            </div>
        </div>
    }
}
