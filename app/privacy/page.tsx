import Link from "next/link";
import { BadgeCheck, Ban, Cookie, FileLock2, Lock, Mail, MessageSquareQuote, ShieldCheck, UserRoundCheck } from "lucide-react";
import { SiteHeader } from "@/components/site/header";
import { SiteFooter } from "@/components/site/footer";
import { Eyebrow } from "@/components/ui";

export const metadata = { title: "Privacy & HIPAA" };

const WEBSITE = [
  { icon: Ban, title: "No personal data on the website", text: "The marketing site never asks for your name, contact details, symptoms tied to your identity, or insurance IDs. The smart appointment request runs entirely in your browser." },
  { icon: FileLock2, title: "PHI only in the secure portal", text: "Anything that identifies you is entered only in our patient portal, which is encrypted, never cached, clears itself after 15 minutes idle, and is run by a vendor under a Business Associate Agreement." },
  { icon: Cookie, title: "No trackers, pixels or cookies", text: "We do not use advertising pixels, session-recording tools, third-party chat widgets or tracking cookies. Fonts are served from our own domain, so your visit isn't shared with outside networks." },
  { icon: Lock, title: "Encrypted everywhere", text: "HTTPS is enforced on every page (HSTS), with strict security headers that block framing, content injection and access to your camera, microphone and location." },
  { icon: BadgeCheck, title: "Business Associate Agreements", text: "Every vendor that hosts, transmits or processes patient information (hosting, portal, SMS, fax, clearinghouse, AI services) signs a BAA before it touches any data." },
  { icon: MessageSquareQuote, title: "Testimonials need written authorization", text: "We never publish identifiable patient stories, photos or outcomes without a signed, specific HIPAA authorization from the patient, which can be revoked at any time." },
];

const RIGHTS = [
  ["See and get a copy", "of your medical and billing records, generally within 30 days, in paper or electronic form."],
  ["Ask us to correct", "information you believe is incorrect or incomplete."],
  ["Get a list of disclosures", "showing who we shared your information with, for most purposes other than treatment, payment and operations."],
  ["Request restrictions", "on what we share, including with your health plan when you pay out of pocket in full."],
  ["Choose how we contact you", "for example, only at a specific phone number or by secure portal message."],
  ["File a complaint", "with our Privacy Officer or with the U.S. Department of Health and Human Services, Office for Civil Rights. We will never retaliate."],
];

export default function PrivacyPage() {
  return (
    <>
      <SiteHeader />
      <main>
        <section className="relative overflow-hidden border-b border-line">
          <div className="grain pointer-events-none absolute inset-0" />
          <div className="relative mx-auto max-w-4xl px-4 py-16 sm:px-6">
            <Eyebrow>Privacy &amp; HIPAA</Eyebrow>
            <h1 className="mt-3 font-display text-5xl font-semibold tracking-tight text-ink">Your health information stays yours.</h1>
            <p className="mt-5 max-w-2xl text-[17px] leading-relaxed text-muted">
              We follow the HIPAA Privacy, Security and Breach Notification Rules across our website, patient portal and operations platform. This page explains, in plain language, how we protect you.
            </p>
            <div className="mt-6 flex flex-wrap gap-2 text-[12px] font-semibold">
              <Link href="/portal" className="inline-flex items-center gap-1.5 rounded-xl bg-navy px-4 py-2.5 text-white hover:bg-ocean"><Lock className="size-3.5" /> Go to the secure portal</Link>
              <a href="#rights" className="inline-flex items-center gap-1.5 rounded-xl bg-white px-4 py-2.5 text-ink ring-1 ring-line-2 hover:bg-sand"><UserRoundCheck className="size-3.5" /> Your rights</a>
            </div>
          </div>
        </section>

        <section className="mx-auto max-w-6xl px-4 py-16 sm:px-6">
          <h2 className="font-display text-3xl font-semibold text-ink">How this website protects you</h2>
          <div className="mt-8 grid gap-5 md:grid-cols-2 lg:grid-cols-3">
            {WEBSITE.map((w) => (
              <div key={w.title} className="rounded-2xl border border-line bg-white p-6 shadow-[var(--shadow-soft)]">
                <div className="grid size-10 place-items-center rounded-xl bg-sage-soft text-sage"><w.icon className="size-5" /></div>
                <h3 className="mt-4 text-[16px] font-semibold text-ink">{w.title}</h3>
                <p className="mt-1.5 text-[14px] leading-relaxed text-muted">{w.text}</p>
              </div>
            ))}
          </div>
        </section>

        <section id="notice" className="scroll-mt-28 bg-white py-16">
          <div className="mx-auto grid max-w-6xl gap-10 px-4 sm:px-6 lg:grid-cols-[0.9fr_1.1fr]">
            <div>
              <Eyebrow>Notice of Privacy Practices · summary</Eyebrow>
              <h2 className="mt-3 font-display text-3xl font-semibold text-ink">How we use and share your information</h2>
              <p className="mt-4 text-[15px] leading-relaxed text-muted">We use the minimum information necessary for each purpose, and staff can only see what their role requires. Every access is logged.</p>
            </div>
            <ul className="space-y-4 text-[14px] leading-relaxed text-muted">
              <li><b className="text-ink">Treatment:</b> sharing with the clinicians, therapists, imaging centers and hospitals involved in your care.</li>
              <li><b className="text-ink">Payment:</b> verifying coverage, requesting authorizations and billing your health plan, workers&apos; compensation carrier or no-fault insurer.</li>
              <li><b className="text-ink">Health care operations:</b> quality improvement, staff training and running the practice, including AI tools that operate under a BAA and never use your data to train public models.</li>
              <li><b className="text-ink">Only with your written authorization:</b> marketing, testimonials, sale of information, and most uses not listed here.</li>
              <li><b className="text-ink">If a breach happens:</b> we notify affected patients without unreasonable delay and no later than 60 days after discovery, plus HHS and, when required, the media.</li>
              <li><b className="text-ink">Email and text:</b> please don&apos;t send medical details by regular email. Use the portal; our texts never include diagnoses or results.</li>
            </ul>
          </div>
        </section>

        <section id="rights" className="scroll-mt-28 mx-auto max-w-6xl px-4 py-16 sm:px-6">
          <h2 className="font-display text-3xl font-semibold text-ink">Your rights</h2>
          <div className="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {RIGHTS.map(([t, d]) => (
              <div key={t} className="rounded-2xl border border-line bg-cream p-5">
                <div className="flex items-center gap-2 text-[15px] font-semibold text-ink"><ShieldCheck className="size-4 text-sage" />{t}</div>
                <p className="mt-1.5 text-[13.5px] text-muted">{d}</p>
              </div>
            ))}
          </div>
          <div className="mt-8 flex flex-wrap items-center justify-between gap-4 rounded-2xl bg-navy p-6 text-white">
            <div>
              <div className="text-[15px] font-semibold">Privacy Officer</div>
              <div className="mt-1 text-[13.5px] text-white/70">Questions, requests or complaints about your health information.</div>
            </div>
            <div className="flex items-center gap-2 text-[14px] font-semibold"><Mail className="size-4 text-gold-2" /> privacy@meridian-ortho.example · (516) 555-0148</div>
          </div>
        </section>
      </main>
      <SiteFooter />
    </>
  );
}
