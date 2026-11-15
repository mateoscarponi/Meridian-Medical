import Link from "next/link";
import {
  Activity, ArrowRight, ArrowUpRight, Bone, Brain, CalendarClock, CheckCircle2, ClipboardCheck, FileScan, FileText,
  HeartPulse, MapPin, Network, Receipt, ShieldCheck, Sparkles, Stethoscope, Syringe, Timer, Workflow, Clock, Bot, Briefcase, Gauge,
} from "lucide-react";
import { SiteHeader } from "@/components/site/header";
import { SmartIntake } from "@/components/site/smart-intake";
import { Reveal } from "@/components/site/reveal";
import { SiteFooter } from "@/components/site/footer";
import { Eyebrow } from "@/components/ui";
import { CARE_PATH, CONDITIONS, INSURANCE, LOCATIONS, PRACTICE, SPECIALTIES, TEAM, VISIT_CHECKLIST } from "@/lib/practice";

const SPECIALTY_ICONS = [Bone, Syringe, Stethoscope, Activity];

const OUTCOMES = [
  { metric: "−41%", label: "Administrative workload", sub: "staff hours on manual tasks", icon: Timer },
  { metric: "10 → 2.6d", label: "Authorization turnaround", sub: "median, diagnostics & procedures", icon: ClipboardCheck },
  { metric: "9 systems", label: "Interoperability", sub: "EMR, payers, imaging, fax, PT", icon: Network },
  { metric: "71%", label: "Tasks automated", sub: "intake, reminders, scrubbing, routing", icon: Workflow },
  { metric: "Measured", label: "AI that proves itself", sub: "every tool reports time saved", icon: Brain },
];

const MODULES = [
  { href: "/platform/prior-auth", title: "Prior Auth Copilot", text: "Reads the chart, checks payer criteria, drafts the letter and submits.", icon: ClipboardCheck },
  { href: "/platform/documents", title: "Document AI Inbox", text: "OCR + classification for faxes, referrals, imaging and insurance cards.", icon: FileScan },
  { href: "/platform/scribe", title: "Clinical Scribe", text: "Visit transcript to a structured SOAP note with ICD-10 suggestions.", icon: FileText },
  { href: "/platform/claims", title: "Claim Scrubber", text: "Catches bundling, modifier and coding errors before the payer does.", icon: Receipt },
  { href: "/platform/scheduling", title: "Smart Scheduling", text: "A no-show model trained in Python scores every appointment.", icon: CalendarClock },
  { href: "/platform/interop", title: "Interoperability Lab", text: "Live HL7 v2 → FHIR R4 conversion and a working FHIR API.", icon: Network },
  { href: "/platform/agent", title: "AI Front-Desk Agent", text: "Reschedules, answers auth questions in English or Spanish, and escalates red flags.", icon: Bot },
  { href: "/platform/wc-nofault", title: "WC & No-Fault Desk", text: "Tracks every NY filing deadline and drafts the forms from the chart.", icon: Briefcase },
  { href: "/platform/impact", title: "Impact Calculator", text: "Enter the practice's own volumes to see hours and dollars returned.", icon: Gauge },
];

export default function Home() {
  return (
    <>
      <SiteHeader />
      <main>
        {/* HERO */}
        <section className="relative overflow-hidden">
          <div className="grain pointer-events-none absolute inset-0" />
          <div className="pointer-events-none absolute -left-40 -top-40 size-[520px] rounded-full bg-sky/70 blur-3xl" />
          <div className="pointer-events-none absolute -right-32 top-40 size-[420px] rounded-full bg-gold-soft blur-3xl" />
          <div className="relative mx-auto grid max-w-7xl gap-12 px-4 pb-20 pt-14 sm:px-6 lg:grid-cols-[1.05fr_0.95fr] lg:pt-20">
            <div className="animate-fade-up">
              <Eyebrow>Long Island · Queens</Eyebrow>
              <h1 className="mt-4 font-display text-[44px] leading-[1.04] font-semibold tracking-tight text-ink sm:text-6xl">
                Orthopedic &amp; spine care,
                <span className="block text-ocean">without the wait.</span>
              </h1>
              <p className="mt-6 max-w-xl text-[17px] leading-relaxed text-muted">
                Fellowship-trained spine surgery, interventional pain management, orthopedics and physical therapy under one roof.
                We start with the least invasive path, and our care team handles referrals, authorizations and paperwork for you.
              </p>
              <div className="mt-8 flex flex-wrap gap-3">
                <a href="#request" className="inline-flex items-center gap-2 rounded-xl bg-navy px-5 py-3 text-[15px] font-semibold text-white shadow-sm transition hover:bg-ocean">
                  Request an appointment <ArrowRight className="size-4" />
                </a>
                <a href="#locations" className="inline-flex items-center gap-2 rounded-xl bg-white px-5 py-3 text-[15px] font-semibold text-ink ring-1 ring-inset ring-line-2 transition hover:bg-sand">
                  <MapPin className="size-4" /> Find a location
                </a>
              </div>
              <dl className="mt-12 grid max-w-xl grid-cols-2 gap-6 border-t border-line pt-8 sm:grid-cols-4">
                {[
                  ["Since 2018", "caring for the community"],
                  ["4 offices", "Nassau, Suffolk & Queens"],
                  ["Same week", "new-patient visits"],
                  ["Non-surgical", "first, always"],
                ].map(([a, b]) => (
                  <div key={a}>
                    <dt className="font-display text-[19px] font-semibold text-ink">{a}</dt>
                    <dd className="mt-1 text-[13px] text-muted">{b}</dd>
                  </div>
                ))}
              </dl>
            </div>
            <div className="animate-fade-up [animation-delay:150ms]">
              <SmartIntake />
            </div>
          </div>
        </section>

        {/* SPECIALTIES */}
        <section id="specialties" className="scroll-mt-28 bg-white py-24">
          <div className="mx-auto max-w-7xl px-4 sm:px-6">
            <Reveal className="max-w-2xl">
              <Eyebrow>Specialties</Eyebrow>
              <h2 className="mt-3 font-display text-4xl font-semibold tracking-tight text-ink">Four disciplines, one coordinated plan.</h2>
              <p className="mt-4 text-[16px] leading-relaxed text-muted">Your surgeon, pain physician and therapist share one chart and one plan, so nothing gets lost between visits.</p>
            </Reveal>
            <div className="mt-14 grid gap-5 md:grid-cols-2 lg:grid-cols-4">
              {SPECIALTIES.map((s, i) => {
                const Icon = SPECIALTY_ICONS[i];
                return (
                  <Reveal key={s.slug} delay={i * 80} className="group relative flex flex-col rounded-2xl border border-line bg-cream p-6 transition-all duration-300 hover:-translate-y-1 hover:bg-white hover:shadow-[var(--shadow-lift)]">
                    <div className="grid size-11 place-items-center rounded-xl bg-navy text-white transition-colors group-hover:bg-ocean">
                      <Icon className="size-5" />
                    </div>
                    <h3 className="mt-5 font-display text-xl font-semibold text-ink">{s.title}</h3>
                    <p className="mt-2 text-[14px] leading-relaxed text-muted">{s.blurb}</p>
                    <ul className="mt-5 space-y-2 border-t border-line pt-4 text-[13px] text-ink">
                      {s.items.map((it) => (
                        <li key={it} className="flex gap-2"><span className="mt-[7px] size-1.5 shrink-0 rounded-full bg-gold" />{it}</li>
                      ))}
                    </ul>
                  </Reveal>
                );
              })}
            </div>
          </div>
        </section>

        {/* CARE PATH */}
        <section className="py-24">
          <div className="mx-auto max-w-7xl px-4 sm:px-6">
            <div className="grid gap-12 lg:grid-cols-[0.8fr_1.2fr] lg:items-center">
              <Reveal>
                <Eyebrow>Our philosophy</Eyebrow>
                <h2 className="mt-3 font-display text-4xl font-semibold tracking-tight text-ink">Surgery is the last step, not the first.</h2>
                <p className="mt-4 text-[16px] leading-relaxed text-muted">
                  Most neck and back pain improves with the right conservative care. When it doesn&apos;t, we escalate thoughtfully and involve you and your family in every decision.
                </p>
                <p className="mt-6 font-display text-lg italic text-gold">{PRACTICE.tagline}</p>
              </Reveal>
              <div className="relative grid gap-4 sm:grid-cols-2">
                {CARE_PATH.map((c, i) => (
                  <Reveal key={c.step} delay={i * 90} className="rounded-2xl border border-line bg-white p-6 shadow-[var(--shadow-soft)]">
                    <div className="font-display text-3xl font-semibold text-sky">{c.step}</div>
                    <h3 className="mt-3 text-[16px] font-semibold text-ink">{c.title}</h3>
                    <p className="mt-1.5 text-[14px] leading-relaxed text-muted">{c.text}</p>
                  </Reveal>
                ))}
              </div>
            </div>
          </div>
        </section>

        {/* CONDITIONS */}
        <section id="conditions" className="scroll-mt-28 bg-sand/60 py-24">
          <div className="mx-auto max-w-7xl px-4 sm:px-6">
            <Reveal className="flex flex-wrap items-end justify-between gap-6">
              <div className="max-w-2xl">
                <Eyebrow>Conditions we treat</Eyebrow>
                <h2 className="mt-3 font-display text-4xl font-semibold tracking-tight text-ink">Expert care for the spine and joints.</h2>
              </div>
              <p className="max-w-md text-[15px] text-muted">Including osteoarthritis, rheumatoid arthritis and trauma-related disorders of the joints and spine.</p>
            </Reveal>
            <div className="mt-12 grid gap-px overflow-hidden rounded-2xl border border-line bg-line sm:grid-cols-2 lg:grid-cols-4">
              {CONDITIONS.map((c, i) => (
                <Reveal key={c.title} delay={(i % 4) * 60} className="group bg-cream p-6 transition-colors hover:bg-white">
                  <div className="flex items-center justify-between">
                    <h3 className="text-[16px] font-semibold text-ink">{c.title}</h3>
                    <ArrowUpRight className="size-4 text-faint transition-all group-hover:-translate-y-0.5 group-hover:translate-x-0.5 group-hover:text-ocean" />
                  </div>
                  <p className="mt-2 text-[14px] leading-relaxed text-muted">{c.text}</p>
                </Reveal>
              ))}
            </div>
          </div>
        </section>

        {/* TEAM */}
        <section id="team" className="scroll-mt-28 py-24">
          <div className="mx-auto max-w-7xl px-4 sm:px-6">
            <Reveal className="max-w-2xl">
              <Eyebrow>Our team</Eyebrow>
              <h2 className="mt-3 font-display text-4xl font-semibold tracking-tight text-ink">Physicians who listen first.</h2>
            </Reveal>
            <div className="mt-12 grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
              {TEAM.map((t, i) => (
                <Reveal key={t.name} delay={i * 80} className="rounded-2xl border border-line bg-white p-6 shadow-[var(--shadow-soft)]">
                  <div className="grid size-16 place-items-center rounded-2xl bg-gradient-to-br from-navy to-ocean font-display text-xl font-semibold text-white">{t.initials}</div>
                  <h3 className="mt-5 text-[16px] font-semibold text-ink">{t.name}</h3>
                  <div className="mt-0.5 text-[13px] font-semibold text-ocean">{t.role}</div>
                  <p className="mt-3 text-[14px] leading-relaxed text-muted">{t.detail}</p>
                </Reveal>
              ))}
            </div>
          </div>
        </section>

        {/* LOCATIONS */}
        <section id="locations" className="scroll-mt-28 bg-white py-24">
          <div className="mx-auto max-w-7xl px-4 sm:px-6">
            <Reveal className="max-w-2xl">
              <Eyebrow>Locations</Eyebrow>
              <h2 className="mt-3 font-display text-4xl font-semibold tracking-tight text-ink">Four offices across Long Island &amp; Queens.</h2>
            </Reveal>
            <div className="mt-12 grid gap-5 md:grid-cols-2 lg:grid-cols-4">
              {LOCATIONS.map((l, i) => (
                <Reveal key={l.town} delay={i * 80} className="flex flex-col rounded-2xl border border-line bg-cream p-6">
                  <div className="flex items-center gap-2 text-[12px] font-semibold uppercase tracking-wide text-gold"><MapPin className="size-3.5" /> {l.county}</div>
                  <h3 className="mt-3 font-display text-2xl font-semibold text-ink">{l.town}</h3>
                  <div className="mt-1 text-[14px] font-semibold text-ocean">{l.kind}</div>
                  <p className="mt-3 flex-1 text-[14px] text-muted">{l.note}</p>
                  <div className="mt-5 border-t border-line pt-4 text-[13px] text-muted">{PRACTICE.phone}</div>
                </Reveal>
              ))}
            </div>
          </div>
        </section>

        {/* FOR PATIENTS */}
        <section id="patients" className="scroll-mt-28 py-24">
          <div className="mx-auto grid max-w-7xl gap-10 px-4 sm:px-6 lg:grid-cols-3">
            <Reveal className="lg:col-span-1">
              <Eyebrow>For your visit</Eyebrow>
              <h2 className="mt-3 font-display text-4xl font-semibold tracking-tight text-ink">Come prepared, leave with a plan.</h2>
              <p className="mt-4 text-[15px] leading-relaxed text-muted">New patients: please arrive 15 minutes early. Forms are available digitally in English and Spanish and can be completed from your phone.</p>
            </Reveal>
            <Reveal delay={80} className="rounded-2xl border border-line bg-white p-6 shadow-[var(--shadow-soft)]">
              <h3 className="flex items-center gap-2 text-[16px] font-semibold text-ink"><ClipboardCheck className="size-5 text-sage" /> What to bring</h3>
              <ul className="mt-4 space-y-2.5 text-[14px] text-muted">
                {VISIT_CHECKLIST.map((v) => <li key={v} className="flex gap-2"><CheckCircle2 className="mt-0.5 size-4 shrink-0 text-sage" />{v}</li>)}
              </ul>
            </Reveal>
            <Reveal delay={160} className="flex flex-col gap-5">
              <div className="rounded-2xl border border-line bg-white p-6 shadow-[var(--shadow-soft)]">
                <h3 className="flex items-center gap-2 text-[16px] font-semibold text-ink"><ShieldCheck className="size-5 text-ocean" /> Insurance</h3>
                <ul className="mt-4 divide-y divide-line text-[14px]">
                  {INSURANCE.map((i) => (
                    <li key={i.name} className="flex justify-between gap-4 py-2"><span className="font-medium text-ink">{i.name}</span><span className="text-right text-muted">{i.note}</span></li>
                  ))}
                </ul>
              </div>
              <div className="rounded-2xl bg-navy p-6 text-white">
                <h3 className="flex items-center gap-2 text-[16px] font-semibold"><Clock className="size-5 text-gold-2" /> Authorizations, faster</h3>
                <p className="mt-2 text-[14px] leading-relaxed text-white/75">
                  Insurers can take up to 10 business days for diagnostic tests and 25 for surgery. Our authorization desk submits complete requests the same day, so most decisions come back within days.
                </p>
              </div>
            </Reveal>
          </div>
        </section>

        {/* PRIVACY */}
        <section className="pb-24">
          <div className="mx-auto max-w-7xl px-4 sm:px-6">
            <Reveal className="grid gap-6 rounded-3xl border border-line bg-white p-6 shadow-[var(--shadow-soft)] sm:p-8 lg:grid-cols-[0.9fr_1.6fr] lg:items-center">
              <div>
                <div className="flex items-center gap-2 text-[12px] font-semibold uppercase tracking-[0.16em] text-sage"><ShieldCheck className="size-4" /> Privacy by design</div>
                <h2 className="mt-2 font-display text-2xl font-semibold text-ink">HIPAA-compliant from the first click.</h2>
                <Link href="/privacy" className="mt-3 inline-flex items-center gap-1 text-[14px] font-semibold text-ocean hover:underline">How we protect your information <ArrowRight className="size-4" /></Link>
              </div>
              <ul className="grid gap-4 sm:grid-cols-2">
                {[
                  ["No personal data on this site", "Identifiable details are only entered in the encrypted patient portal."],
                  ["No ad trackers, pixels or cookies", "Your browsing is never shared with ad networks."],
                  ["BAAs with every vendor", "Hosting, portal, SMS, fax and AI services are contractually bound."],
                  ["Stories only with written consent", "No patient story is published without a signed HIPAA authorization."],
                ].map(([t, d]) => (
                  <li key={t} className="flex gap-3">
                    <CheckCircle2 className="mt-0.5 size-5 shrink-0 text-sage" />
                    <div><div className="text-[14px] font-semibold text-ink">{t}</div><div className="text-[13px] text-muted">{d}</div></div>
                  </li>
                ))}
              </ul>
            </Reveal>
          </div>
        </section>

        {/* BEHIND THE FRONT DESK */}
        <section className="relative overflow-hidden bg-ink py-24 text-white">
          <div className="pointer-events-none absolute -right-40 -top-40 size-[560px] rounded-full bg-ocean/30 blur-3xl" />
          <div className="pointer-events-none absolute -bottom-48 -left-24 size-[460px] rounded-full bg-sage/20 blur-3xl" />
          <div className="relative mx-auto max-w-7xl px-4 sm:px-6">
            <Reveal className="flex flex-wrap items-end justify-between gap-8">
              <div className="max-w-2xl">
                <div className="flex items-center gap-2 text-[12px] font-semibold uppercase tracking-[0.16em] text-gold-2"><Sparkles className="size-3.5" /> Behind the front desk</div>
                <h2 className="mt-3 font-display text-4xl font-semibold tracking-tight sm:text-5xl">A care operations platform that gives staff their time back.</h2>
                <p className="mt-4 text-[16px] leading-relaxed text-white/70">
                  AI and automation across authorizations, documents, claims, scheduling and patient messages, connected to the EMR through HL7 and FHIR. Every tool reports the time it saves.
                </p>
              </div>
              <Link href="/platform" className="inline-flex items-center gap-2 rounded-xl bg-white px-5 py-3 text-[15px] font-semibold text-ink transition hover:bg-sky">
                Open the platform <ArrowRight className="size-4" />
              </Link>
            </Reveal>

            <div className="mt-14 grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
              {OUTCOMES.map((o, i) => (
                <Reveal key={o.label} delay={i * 70} className="rounded-2xl border border-white/10 bg-white/[0.04] p-5 backdrop-blur transition hover:bg-white/[0.07]">
                  <o.icon className="size-5 text-gold-2" />
                  <div className="mt-4 font-display text-[28px] leading-none font-semibold">{o.metric}</div>
                  <div className="mt-2 text-[14px] font-semibold">{o.label}</div>
                  <div className="mt-1 text-[12px] text-white/55">{o.sub}</div>
                </Reveal>
              ))}
            </div>

            <div className="mt-6 grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {MODULES.map((m, i) => (
                <Reveal key={m.href} delay={i * 60}>
                  <Link href={m.href} className="group flex h-full items-start gap-4 rounded-2xl border border-white/10 bg-white/[0.03] p-5 transition hover:border-white/25 hover:bg-white/[0.07]">
                    <div className="grid size-10 shrink-0 place-items-center rounded-xl bg-white/10 text-sky"><m.icon className="size-5" /></div>
                    <div>
                      <div className="flex items-center gap-1.5 text-[15px] font-semibold">{m.title}<ArrowUpRight className="size-3.5 opacity-50 transition group-hover:translate-x-0.5 group-hover:-translate-y-0.5 group-hover:opacity-100" /></div>
                      <p className="mt-1 text-[13px] leading-relaxed text-white/60">{m.text}</p>
                    </div>
                  </Link>
                </Reveal>
              ))}
            </div>
            <p className="mt-8 flex items-center gap-2 text-[12px] text-white/45"><HeartPulse className="size-3.5" /> HIPAA-minded by design: synthetic data only in this demo, PHI redaction, audit logging and role-based access.</p>
          </div>
        </section>
      </main>

      <SiteFooter />
    </>
  );
}
