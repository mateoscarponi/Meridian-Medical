use crate::components::icons::Icon;
use crate::components::shell::ToolHeader;
use crate::components::ui::*;
use crate::domain::util::{js_round, locale, num};
use leptos::prelude::*;

// Indices into the inputs array (keys of DEFAULTS in the original).
const PROVIDERS: usize = 0;
const ADMIN_STAFF: usize = 1;
const VISITS_WEEK: usize = 2;
const AUTHS_WEEK: usize = 3;
const MIN_PER_AUTH: usize = 4;
const DOCS_WEEK: usize = 5;
const CALLS_WEEK: usize = 6;
const CLAIMS_MONTH: usize = 7;
const DENIAL_RATE: usize = 8;
const AVG_CLAIM: usize = 9;
const NO_SHOW_RATE: usize = 10;
const REVENUE_PER_VISIT: usize = 11;
const STAFF_HOURLY: usize = 12;
const DOC_MIN_PER_VISIT: usize = 13;
const AUTH_REDUCTION: usize = 14;
const INTAKE_AUTOMATION: usize = 15;
const CALL_DEFLECTION: usize = 16;
const DENIAL_REDUCTION: usize = 17;
const NO_SHOW_REDUCTION: usize = 18;
const SCRIBE_REDUCTION: usize = 19;

const DEFAULTS: [f64; 20] = [4.0, 12.0, 620.0, 70.0, 45.0, 520.0, 1200.0, 2600.0, 11.0, 240.0, 13.0, 185.0, 24.0, 10.0, 70.0, 75.0, 30.0, 40.0, 35.0, 60.0];

/// (key, label, min, max, step, unit)
const OPS: [(usize, &str, f64, f64, f64, &str); 14] = [
    (PROVIDERS, "Providers", 1.0, 20.0, 1.0, ""),
    (ADMIN_STAFF, "Admin / front-desk staff", 2.0, 60.0, 1.0, ""),
    (VISITS_WEEK, "Patient visits per week", 100.0, 3000.0, 10.0, ""),
    (AUTHS_WEEK, "Prior auths per week", 5.0, 300.0, 1.0, ""),
    (MIN_PER_AUTH, "Minutes per manual auth", 10.0, 90.0, 1.0, " min"),
    (DOCS_WEEK, "Faxes / documents per week", 50.0, 2000.0, 10.0, ""),
    (CALLS_WEEK, "Calls & messages per week", 100.0, 4000.0, 50.0, ""),
    (CLAIMS_MONTH, "Claims per month", 300.0, 12000.0, 100.0, ""),
    (DENIAL_RATE, "Initial denial rate", 2.0, 25.0, 1.0, "%"),
    (AVG_CLAIM, "Average claim value", 80.0, 1500.0, 10.0, "$"),
    (NO_SHOW_RATE, "No-show rate", 2.0, 30.0, 1.0, "%"),
    (REVENUE_PER_VISIT, "Revenue per visit", 60.0, 600.0, 5.0, "$"),
    (STAFF_HOURLY, "Loaded staff cost / hour", 15.0, 60.0, 1.0, "$"),
    (DOC_MIN_PER_VISIT, "Provider documentation / visit", 4.0, 30.0, 1.0, " min"),
];

const LEVERS: [(usize, &str, &str); 6] = [
    (AUTH_REDUCTION, "Auth staff time reduced", "Prior Auth Copilot"),
    (INTAKE_AUTOMATION, "Document intake automated", "Document AI"),
    (CALL_DEFLECTION, "Calls & messages resolved by AI", "Agent + Messages"),
    (DENIAL_REDUCTION, "Denials prevented", "Claim Scrubber"),
    (NO_SHOW_REDUCTION, "No-shows prevented", "Smart Scheduling"),
    (SCRIBE_REDUCTION, "Documentation time saved", "Clinical Scribe"),
];

#[derive(Clone, PartialEq)]
struct Results {
    breakdown: [(&'static str, f64); 5],
    staff_h: f64,
    provider_h: f64,
    fte: f64,
    labor_year: f64,
    no_show_revenue: f64,
    denial_revenue: f64,
    recovered_visits_week: f64,
    auth_days: f64,
    admin_reduction: f64,
    automation_share: f64,
    total: f64,
}

fn compute(v: &[f64; 20]) -> Results {
    let pct = |k: usize| v[k] / 100.0;
    let auth_h = v[AUTHS_WEEK] * v[MIN_PER_AUTH] * pct(AUTH_REDUCTION) / 60.0;
    let intake_h = v[DOCS_WEEK] * 6.0 * pct(INTAKE_AUTOMATION) / 60.0;
    let calls_h = v[CALLS_WEEK] * 4.0 * pct(CALL_DEFLECTION) / 60.0;
    let claims_week = v[CLAIMS_MONTH] / 4.33;
    let denied_week = claims_week * pct(DENIAL_RATE);
    let claims_h = (denied_week * pct(DENIAL_REDUCTION) * 30.0 + claims_week * 0.5) / 60.0;
    let referral_h = v[VISITS_WEEK] * 0.12 * 8.0 * 0.55 / 60.0;
    let staff_h = auth_h + intake_h + calls_h + claims_h + referral_h;
    let provider_h = v[VISITS_WEEK] * v[DOC_MIN_PER_VISIT] * pct(SCRIBE_REDUCTION) / 60.0;
    let fte = staff_h / 38.0;
    let labor_year = staff_h * v[STAFF_HOURLY] * 50.0;
    let recovered_visits_week = v[VISITS_WEEK] * pct(NO_SHOW_RATE) * pct(NO_SHOW_REDUCTION);
    let no_show_revenue = recovered_visits_week * v[REVENUE_PER_VISIT] * 50.0;
    let denial_revenue = v[CLAIMS_MONTH] * 12.0 * pct(DENIAL_RATE) * pct(DENIAL_REDUCTION) * v[AVG_CLAIM] * 0.35;
    let auth_days = (10.0 * (1.0 - pct(AUTH_REDUCTION) * 1.06)).max(1.5);
    let total_staff_week = v[ADMIN_STAFF] * 38.0;
    let admin_reduction = (staff_h / (total_staff_week * 0.7).max(1.0)).min(0.95);
    let automation_share = (0.08 + (pct(INTAKE_AUTOMATION) + pct(CALL_DEFLECTION) + pct(AUTH_REDUCTION) + pct(DENIAL_REDUCTION)) / 4.0 * 0.85).min(0.95);
    Results {
        breakdown: [
            ("Prior authorizations", auth_h),
            ("Document intake", intake_h),
            ("Calls & messages", calls_h),
            ("Claims & denials", claims_h),
            ("Referral coordination", referral_h),
        ],
        staff_h,
        provider_h,
        fte,
        labor_year,
        no_show_revenue,
        denial_revenue,
        recovered_visits_week,
        auth_days,
        admin_reduction,
        automation_share,
        total: labor_year + no_show_revenue + denial_revenue,
    }
}

#[component]
fn Slider(
    label: &'static str,
    #[prop(optional)] hint: Option<&'static str>,
    value: Signal<f64>,
    min: f64,
    max: f64,
    #[prop(default = 1.0)] step: f64,
    #[prop(optional)] unit: &'static str,
    on_change: impl Fn(f64) + 'static,
) -> impl IntoView {
    let display = move || {
        let v = value.get();
        if unit == "$" {
            format!("${}", locale(v))
        } else {
            format!("{}{unit}", locale(v))
        }
    };
    view! {
        <label class="block">
            <div class="flex items-baseline justify-between gap-2 text-[12px]">
                <span class="font-semibold text-muted">
                    {label}
                    {hint.map(|h| view! { <span class="ml-1.5 font-normal text-faint">"· " {h}</span> })}
                </span>
                <span class="font-mono font-semibold text-ink">{display}</span>
            </div>
            <input
                type="range"
                min=num(min)
                max=num(max)
                step=num(step)
                value=move || num(value.get())
                prop:value=move || num(value.get())
                on:input=move |ev| on_change(event_target_value(&ev).parse().unwrap_or(0.0))
                class="mt-1.5 w-full accent-[#1f5c8f]"
            />
        </label>
    }
}

#[component]
fn Big(label: &'static str, value: Signal<String>, sub: Signal<String>, #[prop(optional)] accent: bool) -> impl IntoView {
    view! {
        <Card class=cx(&["p-4", if accent { "border-sage/30 bg-gradient-to-br from-sage-soft to-white" } else { "" }])>
            <div class="text-[11.5px] font-medium uppercase tracking-wide text-faint">{label}</div>
            <div class=cx(&["mt-1.5 font-display text-[26px] leading-none font-semibold tabular-nums", if accent { "text-sage" } else { "text-ink" }])>
                {move || value.get()}
            </div>
            <div class="mt-1.5 text-[12px] text-muted">{move || sub.get()}</div>
        </Card>
    }
}

#[component]
fn Mini(label: &'static str, value: Signal<String>) -> impl IntoView {
    view! {
        <div class="bg-white px-3 py-3">
            <div class="text-[13px] font-semibold tabular-nums text-ink">{move || value.get()}</div>
            <div class="text-[10.5px] text-muted">{label}</div>
        </div>
    }
}

#[component]
pub fn ImpactPage() -> impl IntoView {
    let v = RwSignal::new(DEFAULTS);
    let r = Memo::new(move |_| compute(&v.get()));
    let slider_for = move |k: usize| Signal::derive(move || v.with(|s| s[k]));
    let set = move |k: usize| move |n: f64| v.update(|s| s[k] = n);
    let max_h = move || r.with(|r| r.breakdown.iter().map(|b| b.1).fold(f64::MIN, f64::max));

    view! {
        <div class="mx-auto max-w-7xl">
            <ToolHeader
                eyebrow="Overview · What success looks like"
                title="Impact & ROI Calculator"
                description="Enter the practice's own volumes and see the time and money each tool returns. Every lever links to the outcome it moves, so success is measured from day one instead of assumed."
                tags=&["transparent formulas", "editable assumptions", "baseline → target", "per-workflow attribution"]
            />

            <div class="grid gap-6 xl:grid-cols-[380px_1fr]">
                <div class="space-y-6">
                    <Card>
                        <CardHeader
                            title="Practice volumes"
                            icon=|| view! { <Icon name="Calculator" class="size-4"/> }
                            right=move || {
                                view! {
                                    <button on:click=move |_| v.set(DEFAULTS) class="inline-flex items-center gap-1 text-[12px] font-semibold text-muted hover:text-ink">
                                        <Icon name="RotateCcw" class="size-3"/>
                                        " Reset"
                                    </button>
                                }
                            }
                        />
                        <div class="space-y-4 p-5">
                            {OPS
                                .iter()
                                .map(|(k, label, min, max, step, unit)| {
                                    view! { <Slider label=label value=slider_for(*k) min=*min max=*max step=*step unit=unit on_change=set(*k)/> }
                                })
                                .collect_view()}
                        </div>
                    </Card>
                </div>

                <div class="space-y-6">
                    <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                        <Big
                            label="Staff hours returned / week"
                            value=Signal::derive(move || format!("{} h", num(js_round(r.with(|r| r.staff_h)))))
                            sub=Signal::derive(move || format!("≈ {:.1} FTE of capacity", r.with(|r| r.fte)))
                        />
                        <Big
                            label="Provider hours returned / week"
                            value=Signal::derive(move || format!("{} h", num(js_round(r.with(|r| r.provider_h)))))
                            sub=Signal::derive(|| "less after-hours charting".to_string())
                        />
                        <Big
                            label="Visits recovered / week"
                            value=Signal::derive(move || format!("+{}", num(js_round(r.with(|r| r.recovered_visits_week)))))
                            sub=Signal::derive(|| "from prevented no-shows".to_string())
                        />
                        <Big
                            label="Estimated annual impact"
                            value=Signal::derive(move || money(r.with(|r| r.total), 0))
                            sub=Signal::derive(|| "labor + recovered revenue".to_string())
                            accent=true
                        />
                    </div>

                    <Card>
                        <CardHeader title="Automation levers" subtitle="Conservative defaults; adjust to match your pilot results"/>
                        <div class="grid gap-x-8 gap-y-4 p-5 sm:grid-cols-2">
                            {LEVERS
                                .iter()
                                .map(|(k, label, tool)| view! { <Slider label=label hint=tool value=slider_for(*k) min=0.0 max=90.0 unit="%" on_change=set(*k)/> })
                                .collect_view()}
                        </div>
                    </Card>

                    <div class="grid gap-6 lg:grid-cols-[1fr_1fr]">
                        <Card>
                            <CardHeader title="Where the staff hours come from" subtitle="Hours returned per week, by workflow"/>
                            <div class="space-y-3 p-5">
                                {(0..5)
                                    .map(|i| {
                                        let b = move || r.with(|r| r.breakdown[i]);
                                        view! {
                                            <div class="grid grid-cols-[140px_1fr_48px] items-center gap-3 text-[12.5px]">
                                                <span class="text-ink">{move || b().0}</span>
                                                <div class="h-2.5 rounded-r-[4px] bg-sand">
                                                    <div
                                                        class="h-2.5 rounded-r-[4px] bg-[#1F5C8F] transition-all duration-500"
                                                        style:width=move || format!("{}%", num(b().1 / max_h() * 100.0))
                                                    ></div>
                                                </div>
                                                <span class="text-right font-semibold tabular-nums text-ink">{move || format!("{:.0} h", b().1)}</span>
                                            </div>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                            <div class="grid grid-cols-3 gap-px border-t border-line bg-line text-center">
                                <Mini label="Labor value / yr" value=Signal::derive(move || money(r.with(|r| r.labor_year), 0))/>
                                <Mini label="No-show revenue / yr" value=Signal::derive(move || money(r.with(|r| r.no_show_revenue), 0))/>
                                <Mini label="Denial revenue / yr" value=Signal::derive(move || money(r.with(|r| r.denial_revenue), 0))/>
                            </div>
                        </Card>

                        <Card>
                            <CardHeader title="Mapped to “What success looks like”" icon=|| view! { <Icon name="Target" class="size-4"/> }/>
                            <ul class="divide-y divide-line">
                                {move || {
                                    let r = r.get();
                                    [
                                        (
                                            "Reduced administrative workload for staff and providers",
                                            format!("−{}% admin hours · {} provider h/wk returned", num(js_round(r.admin_reduction * 100.0)), num(js_round(r.provider_h))),
                                        ),
                                        (
                                            "Faster processing of authorizations and documentation",
                                            format!("Auth turnaround 10 → {:.1} business days · notes closed same day", r.auth_days),
                                        ),
                                        ("Improved interoperability between systems", "EMR, payers, imaging, fax, SMS, PT connected via HL7 / FHIR / X12".to_string()),
                                        ("Increased operational efficiency through automation", format!("{}% of repeatable tasks automated", num(js_round(r.automation_share * 100.0)))),
                                        ("AI-driven tools that measurably improve workflows", "Every tool logs time saved & outcome KPIs, reviewed monthly".to_string()),
                                    ]
                                        .into_iter()
                                        .map(|(g, k)| {
                                            view! {
                                                <li class="flex gap-3 px-5 py-3.5">
                                                    <Icon name="CheckCircle2" class="mt-0.5 size-4 shrink-0 text-sage"/>
                                                    <div>
                                                        <div class="text-[13px] font-semibold text-ink">{g}</div>
                                                        <div class="text-[12.5px] text-muted">{k}</div>
                                                    </div>
                                                </li>
                                            }
                                        })
                                        .collect_view()
                                }}
                            </ul>
                        </Card>
                    </div>

                    <div class="flex items-start gap-2 rounded-xl bg-sand p-4 text-[12px] text-muted">
                        <Icon name="Info" class="mt-0.5 size-4 shrink-0"/>
                        <span>
                            "Assumptions: 6 min of manual handling per document, 4 min per call/message, 30 min of rework per denial, 35% of prevented denials representing revenue that would otherwise be lost, 12% of visits starting from referrals (8 min each), 38-hour FTE, 50 working weeks. All inputs are editable; replace them with baseline measurements from a 2-week time study."
                        </span>
                    </div>
                </div>
            </div>
        </div>
    }
}
