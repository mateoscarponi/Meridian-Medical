use crate::components::footer::SiteFooter;
use crate::components::header::SiteHeader;
use crate::components::icons::Icon;
use crate::components::reveal::Reveal;
use crate::components::smart_intake::SmartIntake;
use crate::components::spine3d::HeroSpine;
use crate::components::ui::Eyebrow;
use crate::domain::practice::*;
use leptos::prelude::*;

const SPECIALTY_ICONS: [&str; 4] = ["Bone", "Syringe", "Stethoscope", "Activity"];

struct Outcome {
    metric: &'static str,
    label: &'static str,
    sub: &'static str,
    icon: &'static str,
}

const OUTCOMES: [Outcome; 5] = [
    Outcome { metric: "−41%", label: "Administrative workload", sub: "staff hours on manual tasks", icon: "Timer" },
    Outcome { metric: "10 → 2.6d", label: "Authorization turnaround", sub: "median, diagnostics & procedures", icon: "ClipboardCheck" },
    Outcome { metric: "9 systems", label: "Interoperability", sub: "EMR, payers, imaging, fax, PT", icon: "Network" },
    Outcome { metric: "71%", label: "Tasks automated", sub: "intake, reminders, scrubbing, routing", icon: "Workflow" },
    Outcome { metric: "Measured", label: "AI that proves itself", sub: "every tool reports time saved", icon: "Brain" },
];

struct Module {
    href: &'static str,
    title: &'static str,
    text: &'static str,
    icon: &'static str,
}

const MODULES: [Module; 9] = [
    Module { href: "/platform/prior-auth", title: "Prior Auth Copilot", text: "Reads the chart, checks payer criteria, drafts the letter and submits.", icon: "ClipboardCheck" },
    Module { href: "/platform/documents", title: "Document AI Inbox", text: "OCR + classification for faxes, referrals, imaging and insurance cards.", icon: "FileScan" },
    Module { href: "/platform/scribe", title: "Clinical Scribe", text: "Visit transcript to a structured SOAP note with ICD-10 suggestions.", icon: "FileText" },
    Module { href: "/platform/claims", title: "Claim Scrubber", text: "Catches bundling, modifier and coding errors before the payer does.", icon: "Receipt" },
    Module { href: "/platform/scheduling", title: "Smart Scheduling", text: "A no-show model trained in Rust scores every appointment.", icon: "CalendarClock" },
    Module { href: "/platform/interop", title: "Interoperability Lab", text: "Live HL7 v2 → FHIR R4 conversion and a working FHIR API.", icon: "Network" },
    Module { href: "/platform/agent", title: "AI Front-Desk Agent", text: "Reschedules, answers auth questions in English or Spanish, and escalates red flags.", icon: "Bot" },
    Module { href: "/platform/wc-nofault", title: "WC & No-Fault Desk", text: "Tracks every NY filing deadline and drafts the forms from the chart.", icon: "Briefcase" },
    Module { href: "/platform/impact", title: "Impact Calculator", text: "Enter the practice's own volumes to see hours and dollars returned.", icon: "Gauge" },
];

const HERO_STATS: [(&str, &str); 4] = [
    ("Since 2011", "Treating Long Island families"),
    ("4 offices", "Across Nassau & Suffolk"),
    ("Same week", "New-patient visits"),
    ("Non-surgical", "Conservative care first; surgery as a last resort"),
];

const PRIVACY_POINTS: [(&str, &str); 4] = [
    ("No personal data on this site", "Identifiable details are only entered in the encrypted patient portal."),
    ("No ad trackers, pixels or cookies", "Your browsing is never shared with ad networks."),
    ("BAAs with every vendor", "Hosting, portal, SMS, fax and AI services are contractually bound."),
    ("Stories only with written consent", "No patient story is published without a signed HIPAA authorization."),
];

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <SiteHeader/>
        <main>
            // HERO
            <section class="relative overflow-hidden">
                <div class="grain pointer-events-none absolute inset-0"></div>
                <div class="pointer-events-none absolute -left-40 -top-40 size-[520px] rounded-full bg-sky/70 blur-3xl"></div>
                <div class="pointer-events-none absolute -right-32 top-40 size-[420px] rounded-full bg-gold-soft blur-3xl"></div>
                <div class="relative mx-auto grid max-w-7xl gap-12 px-4 pb-20 pt-14 sm:px-6 lg:grid-cols-[1.05fr_0.95fr] lg:pt-20">
                    <div class="animate-fade-up">
                        <Eyebrow>"Nassau · Suffolk"</Eyebrow>
                        <h1 class="mt-4 font-display text-[44px] leading-[1.04] font-semibold tracking-tight text-ink sm:text-6xl">
                            "Orthopedic & spine care,"
                            <span class="block text-ocean">"without the wait."</span>
                        </h1>
                        <p class="mt-6 max-w-xl text-[17px] leading-relaxed text-muted">
                            "Spine and neck surgery, interventional spine care, joint and sports medicine, and in-house rehab, all coordinated by one team. We start with the least invasive path, and our care team handles referrals, authorizations and paperwork for you."
                        </p>
                        <div class="mt-8 flex flex-wrap gap-3">
                            <a
                                href="#request"
                                class="inline-flex items-center gap-2 rounded-xl bg-navy px-5 py-3 text-[15px] font-semibold text-white shadow-sm transition hover:bg-ocean"
                            >
                                "Request an appointment "
                                <Icon name="ArrowRight" class="size-4"/>
                            </a>
                            <a
                                href="#locations"
                                class="inline-flex items-center gap-2 rounded-xl bg-white px-5 py-3 text-[15px] font-semibold text-ink ring-1 ring-inset ring-line-2 transition hover:bg-sand"
                            >
                                <Icon name="MapPin" class="size-4"/>
                                " Find a location"
                            </a>
                        </div>
                        <dl class="mt-12 grid max-w-xl grid-cols-2 gap-x-5 gap-y-6 border-t border-line pt-8 sm:grid-cols-4">
                            {HERO_STATS
                                .iter()
                                .map(|(a, b)| {
                                    view! {
                                        <div>
                                            <dt class="whitespace-nowrap font-display text-[17px] font-semibold text-gold xl:text-[19px]">{*a}</dt>
                                            <dd class="mt-1 text-[13px] leading-snug text-taupe">{*b}</dd>
                                        </div>
                                    }
                                })
                                .collect_view()}
                        </dl>
                    </div>
                    <div class="animate-fade-up [animation-delay:150ms] lg:-mt-8">
                        <SmartIntake/>
                        <HeroSpine/>
                    </div>
                </div>
            </section>

            // SPECIALTIES
            <section id="specialties" class="scroll-mt-28 bg-white py-24">
                <div class="mx-auto max-w-7xl px-4 sm:px-6">
                    <Reveal class="max-w-2xl">
                        <Eyebrow>"Specialties"</Eyebrow>
                        <h2 class="mt-3 font-display text-4xl font-semibold tracking-tight text-ink">"Four disciplines, one coordinated plan."</h2>
                        <p class="mt-4 text-[16px] leading-relaxed text-muted">
                            "Your surgeon, pain physician and therapist share one chart and one plan, so nothing gets lost between visits."
                        </p>
                    </Reveal>
                    <div class="mt-14 grid gap-5 md:grid-cols-2 lg:grid-cols-4">
                        {SPECIALTIES
                            .iter()
                            .enumerate()
                            .map(|(i, s)| {
                                view! {
                                    <Reveal
                                        delay=(i as u32) * 80
                                        class="group relative flex flex-col rounded-2xl border border-line bg-cream p-6 transition-all duration-300 hover:-translate-y-1 hover:bg-white hover:shadow-[var(--shadow-lift)]"
                                    >
                                        <div class="grid size-11 place-items-center rounded-xl bg-navy text-white transition-colors group-hover:bg-ocean">
                                            <Icon name=SPECIALTY_ICONS[i] class="size-5"/>
                                        </div>
                                        <h3 class="mt-5 font-display text-xl font-semibold text-ink">{s.title}</h3>
                                        <p class="mt-2 text-[14px] leading-relaxed text-muted">{s.blurb}</p>
                                        <ul class="mt-5 space-y-2 border-t border-line pt-4 text-[13px] text-ink">
                                            {s
                                                .items
                                                .iter()
                                                .map(|it| {
                                                    view! {
                                                        <li class="flex gap-2">
                                                            <span class="mt-[7px] size-1.5 shrink-0 rounded-full bg-gold"></span>
                                                            {*it}
                                                        </li>
                                                    }
                                                })
                                                .collect_view()}
                                        </ul>
                                    </Reveal>
                                }
                            })
                            .collect_view()}
                    </div>
                </div>
            </section>

            // CARE PATH
            <section class="py-24">
                <div class="mx-auto max-w-7xl px-4 sm:px-6">
                    <div class="grid gap-12 lg:grid-cols-[0.8fr_1.2fr] lg:items-center">
                        <Reveal>
                            <Eyebrow>"How we treat"</Eyebrow>
                            <h2 class="mt-3 font-display text-4xl font-semibold tracking-tight text-ink">"Surgery is the last step, not the first."</h2>
                            <p class="mt-4 text-[16px] leading-relaxed text-muted">
                                "Most neck and back pain improves with the right conservative care. When it doesn't, we escalate thoughtfully and involve you and your family in every decision."
                            </p>
                            <p class="mt-6 font-display text-lg italic text-gold">{PRACTICE.tagline}</p>
                        </Reveal>
                        <div class="relative grid gap-4 sm:grid-cols-2">
                            {CARE_PATH
                                .iter()
                                .enumerate()
                                .map(|(i, c)| {
                                    view! {
                                        <Reveal delay=(i as u32) * 90 class="rounded-2xl border border-line bg-white p-6 shadow-[var(--shadow-soft)]">
                                            <div class="font-display text-3xl font-semibold text-sky">{c.step}</div>
                                            <h3 class="mt-3 text-[16px] font-semibold text-ink">{c.title}</h3>
                                            <p class="mt-1.5 text-[14px] leading-relaxed text-muted">{c.text}</p>
                                        </Reveal>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </div>
                </div>
            </section>

            // CONDITIONS
            <section id="conditions" class="scroll-mt-28 bg-sand/60 py-24">
                <div class="mx-auto max-w-7xl px-4 sm:px-6">
                    <Reveal class="flex flex-wrap items-end justify-between gap-6">
                        <div class="max-w-2xl">
                            <Eyebrow>"Conditions we treat"</Eyebrow>
                            <h2 class="mt-3 font-display text-4xl font-semibold tracking-tight text-ink">"Expert care for the spine and joints."</h2>
                        </div>
                        <p class="max-w-md text-[15px] text-muted">
                            "Including osteoarthritis, rheumatoid arthritis and trauma-related disorders of the joints and spine."
                        </p>
                    </Reveal>
                    <div class="mt-12 grid gap-px overflow-hidden rounded-2xl border border-line bg-line sm:grid-cols-2 lg:grid-cols-4">
                        {CONDITIONS
                            .iter()
                            .enumerate()
                            .map(|(i, c)| {
                                view! {
                                    <Reveal delay=((i % 4) as u32) * 60 class="group bg-cream p-6 transition-colors hover:bg-white">
                                        <div class="flex items-center justify-between">
                                            <h3 class="text-[16px] font-semibold text-ink">{c.title}</h3>
                                            <Icon
                                                name="ArrowUpRight"
                                                class="size-4 text-faint transition-all group-hover:-translate-y-0.5 group-hover:translate-x-0.5 group-hover:text-ocean"
                                            />
                                        </div>
                                        <p class="mt-2 text-[14px] leading-relaxed text-muted">{c.text}</p>
                                    </Reveal>
                                }
                            })
                            .collect_view()}
                    </div>
                </div>
            </section>

            // TEAM
            <section id="team" class="scroll-mt-28 py-24">
                <div class="mx-auto max-w-7xl px-4 sm:px-6">
                    <Reveal class="max-w-2xl">
                        <Eyebrow>"Our team"</Eyebrow>
                        <h2 class="mt-3 font-display text-4xl font-semibold tracking-tight text-ink">"Physicians who listen first."</h2>
                    </Reveal>
                    <div class="mt-12 grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
                        {TEAM
                            .iter()
                            .enumerate()
                            .map(|(i, t)| {
                                view! {
                                    <Reveal delay=(i as u32) * 80 class="rounded-2xl border border-line bg-white p-6 shadow-[var(--shadow-soft)]">
                                        <div class="grid size-16 place-items-center rounded-2xl bg-gradient-to-br from-navy to-ocean font-display text-xl font-semibold text-white">
                                            {t.initials}
                                        </div>
                                        <h3 class="mt-5 text-[16px] font-semibold text-ink">{t.name}</h3>
                                        <div class="mt-0.5 text-[13px] font-semibold text-ocean">{t.role}</div>
                                        <p class="mt-3 text-[14px] leading-relaxed text-muted">{t.detail}</p>
                                    </Reveal>
                                }
                            })
                            .collect_view()}
                    </div>
                </div>
            </section>

            // LOCATIONS
            <section id="locations" class="scroll-mt-28 bg-white py-24">
                <div class="mx-auto max-w-7xl px-4 sm:px-6">
                    <Reveal class="max-w-2xl">
                        <Eyebrow>"Locations"</Eyebrow>
                        <h2 class="mt-3 font-display text-4xl font-semibold tracking-tight text-ink">"Four offices across Nassau & Suffolk."</h2>
                    </Reveal>
                    <div class="mt-12 grid gap-5 md:grid-cols-2 lg:grid-cols-4">
                        {LOCATIONS
                            .iter()
                            .enumerate()
                            .map(|(i, l)| {
                                view! {
                                    <Reveal delay=(i as u32) * 80 class="flex flex-col rounded-2xl border border-line bg-cream p-6">
                                        <div class="flex items-center gap-2 text-[12px] font-semibold uppercase tracking-wide text-gold">
                                            <Icon name="MapPin" class="size-3.5"/>
                                            " "
                                            {l.county}
                                        </div>
                                        <h3 class="mt-3 font-display text-2xl font-semibold text-ink">{l.town}</h3>
                                        <div class="mt-1 text-[14px] font-semibold text-ocean">{l.kind}</div>
                                        <p class="mt-3 flex-1 text-[14px] text-muted">{l.note}</p>
                                        <div class="mt-5 border-t border-line pt-4 text-[13px] text-muted">{PRACTICE.phone}</div>
                                    </Reveal>
                                }
                            })
                            .collect_view()}
                    </div>
                </div>
            </section>

            // FOR PATIENTS
            <section id="patients" class="scroll-mt-28 py-24">
                <div class="mx-auto grid max-w-7xl gap-10 px-4 sm:px-6 lg:grid-cols-3">
                    <Reveal class="lg:col-span-1">
                        <Eyebrow>"For your visit"</Eyebrow>
                        <h2 class="mt-3 font-display text-4xl font-semibold tracking-tight text-ink">"Come prepared, leave with a plan."</h2>
                        <p class="mt-4 text-[15px] leading-relaxed text-muted">
                            "New patients: please arrive 15 minutes early. Forms are available digitally in English and Spanish and can be completed from your phone."
                        </p>
                    </Reveal>
                    <Reveal delay=80 class="rounded-2xl border border-line bg-white p-6 shadow-[var(--shadow-soft)]">
                        <h3 class="flex items-center gap-2 text-[16px] font-semibold text-ink">
                            <Icon name="ClipboardCheck" class="size-5 text-sage"/>
                            " What to bring"
                        </h3>
                        <ul class="mt-4 space-y-2.5 text-[14px] text-muted">
                            {VISIT_CHECKLIST
                                .iter()
                                .map(|v| {
                                    view! {
                                        <li class="flex gap-2">
                                            <Icon name="CheckCircle2" class="mt-0.5 size-4 shrink-0 text-sage"/>
                                            {*v}
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ul>
                    </Reveal>
                    <Reveal delay=160 class="flex flex-col gap-5">
                        <div class="rounded-2xl border border-line bg-white p-6 shadow-[var(--shadow-soft)]">
                            <h3 class="flex items-center gap-2 text-[16px] font-semibold text-ink">
                                <Icon name="ShieldCheck" class="size-5 text-ocean"/>
                                " Insurance"
                            </h3>
                            <ul class="mt-4 divide-y divide-line text-[14px]">
                                {INSURANCE
                                    .iter()
                                    .map(|i| {
                                        view! {
                                            <li class="flex justify-between gap-4 py-2">
                                                <span class="font-medium text-ink">{i.name}</span>
                                                <span class="text-right text-muted">{i.note}</span>
                                            </li>
                                        }
                                    })
                                    .collect_view()}
                            </ul>
                        </div>
                        <div class="rounded-2xl bg-navy p-6 text-white">
                            <h3 class="flex items-center gap-2 text-[16px] font-semibold">
                                <Icon name="Clock" class="size-5 text-gold-2"/>
                                " Authorizations, faster"
                            </h3>
                            <p class="mt-2 text-[14px] leading-relaxed text-white/75">
                                "Insurers can take up to 10 business days for diagnostic tests and 25 for surgery. Our authorization desk submits complete requests the same day, so most decisions come back within days."
                            </p>
                        </div>
                    </Reveal>
                </div>
            </section>

            // PRIVACY
            <section class="pb-24">
                <div class="mx-auto max-w-7xl px-4 sm:px-6">
                    <Reveal class="grid gap-6 rounded-3xl border border-line bg-white p-6 shadow-[var(--shadow-soft)] sm:p-8 lg:grid-cols-[0.9fr_1.6fr] lg:items-center">
                        <div>
                            <div class="flex items-center gap-2 text-[12px] font-semibold uppercase tracking-[0.16em] text-sage">
                                <Icon name="ShieldCheck" class="size-4"/>
                                " Privacy by design"
                            </div>
                            <h2 class="mt-2 font-display text-2xl font-semibold text-ink">"HIPAA-compliant from the first click."</h2>
                            <a href="/privacy" class="mt-3 inline-flex items-center gap-1 text-[14px] font-semibold text-ocean hover:underline">
                                "How we protect your information "
                                <Icon name="ArrowRight" class="size-4"/>
                            </a>
                        </div>
                        <ul class="grid gap-4 sm:grid-cols-2">
                            {PRIVACY_POINTS
                                .iter()
                                .map(|(t, d)| {
                                    view! {
                                        <li class="flex gap-3">
                                            <Icon name="CheckCircle2" class="mt-0.5 size-5 shrink-0 text-sage"/>
                                            <div>
                                                <div class="text-[14px] font-semibold text-ink">{*t}</div>
                                                <div class="text-[13px] text-muted">{*d}</div>
                                            </div>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ul>
                    </Reveal>
                </div>
            </section>

            // BEHIND THE FRONT DESK
            <section class="relative overflow-hidden bg-ink py-24 text-white">
                <div class="pointer-events-none absolute -right-40 -top-40 size-[560px] rounded-full bg-ocean/30 blur-3xl"></div>
                <div class="pointer-events-none absolute -bottom-48 -left-24 size-[460px] rounded-full bg-sage/20 blur-3xl"></div>
                <div class="relative mx-auto max-w-7xl px-4 sm:px-6">
                    <Reveal class="flex flex-wrap items-end justify-between gap-8">
                        <div class="max-w-2xl">
                            <div class="flex items-center gap-2 text-[12px] font-semibold uppercase tracking-[0.16em] text-gold-2">
                                <Icon name="Sparkles" class="size-3.5"/>
                                " Behind the front desk"
                            </div>
                            <h2 class="mt-3 font-display text-4xl font-semibold tracking-tight sm:text-5xl">
                                "A care operations platform that gives staff their time back."
                            </h2>
                            <p class="mt-4 text-[16px] leading-relaxed text-white/70">
                                "AI and automation across authorizations, documents, claims, scheduling and patient messages, connected to the EMR through HL7 and FHIR. Every tool reports the time it saves."
                            </p>
                        </div>
                        <a
                            href="/platform"
                            class="inline-flex items-center gap-2 rounded-xl bg-white px-5 py-3 text-[15px] font-semibold text-ink transition hover:bg-sky"
                        >
                            "Open the platform "
                            <Icon name="ArrowRight" class="size-4"/>
                        </a>
                    </Reveal>

                    <div class="mt-14 grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
                        {OUTCOMES
                            .iter()
                            .enumerate()
                            .map(|(i, o)| {
                                view! {
                                    <Reveal
                                        delay=(i as u32) * 70
                                        class="rounded-2xl border border-white/10 bg-white/[0.04] p-5 backdrop-blur transition hover:bg-white/[0.07]"
                                    >
                                        <Icon name=o.icon class="size-5 text-gold-2"/>
                                        <div class="mt-4 font-display text-[28px] leading-none font-semibold">{o.metric}</div>
                                        <div class="mt-2 text-[14px] font-semibold">{o.label}</div>
                                        <div class="mt-1 text-[12px] text-white/55">{o.sub}</div>
                                    </Reveal>
                                }
                            })
                            .collect_view()}
                    </div>

                    <div class="mt-6 grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                        {MODULES
                            .iter()
                            .enumerate()
                            .map(|(i, m)| {
                                view! {
                                    <Reveal delay=(i as u32) * 60>
                                        <a
                                            href=m.href
                                            class="group flex h-full items-start gap-4 rounded-2xl border border-white/10 bg-white/[0.03] p-5 transition hover:border-white/25 hover:bg-white/[0.07]"
                                        >
                                            <div class="grid size-10 shrink-0 place-items-center rounded-xl bg-white/10 text-sky">
                                                <Icon name=m.icon class="size-5"/>
                                            </div>
                                            <div>
                                                <div class="flex items-center gap-1.5 text-[15px] font-semibold">
                                                    {m.title}
                                                    <Icon
                                                        name="ArrowUpRight"
                                                        class="size-3.5 opacity-50 transition group-hover:translate-x-0.5 group-hover:-translate-y-0.5 group-hover:opacity-100"
                                                    />
                                                </div>
                                                <p class="mt-1 text-[13px] leading-relaxed text-white/60">{m.text}</p>
                                            </div>
                                        </a>
                                    </Reveal>
                                }
                            })
                            .collect_view()}
                    </div>
                    <p class="mt-8 flex items-center gap-2 text-[12px] text-white/45">
                        <Icon name="HeartPulse" class="size-3.5"/>
                        " HIPAA-minded by design: synthetic data only in this demo, PHI redaction, audit logging and role-based access."
                    </p>
                </div>
            </section>
        </main>
        <SiteFooter/>
    }
}
