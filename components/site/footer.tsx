import Link from "next/link";
import { Lock, ShieldCheck } from "lucide-react";
import { Logo, DemoPill } from "./brand";
import { PRACTICE } from "@/lib/practice";

export function SiteFooter() {
  return (
    <footer className="relative overflow-hidden bg-espresso text-white">
      <div className="pointer-events-none absolute -right-40 -top-40 size-[460px] rounded-full bg-gold/15 blur-3xl" />
      <div className="pointer-events-none absolute -bottom-48 -left-32 size-[420px] rounded-full bg-[#7a4a2a]/25 blur-3xl" />
      <div className="relative mx-auto grid max-w-7xl gap-10 px-4 py-14 sm:px-6 md:grid-cols-5">
        <div className="md:col-span-2">
          <Logo light />
          <p className="mt-4 max-w-sm text-[14px] text-white/60">{PRACTICE.tagline} Orthopedic and spine care for {PRACTICE.region} since {PRACTICE.since}.</p>
          <div className="mt-5 flex flex-wrap gap-2 text-[11.5px] font-semibold text-white/70">
            <span className="inline-flex items-center gap-1 rounded-full bg-white/5 px-2.5 py-1 ring-1 ring-white/10"><Lock className="size-3 text-gold-2" /> HTTPS only</span>
            <span className="inline-flex items-center gap-1 rounded-full bg-white/5 px-2.5 py-1 ring-1 ring-white/10"><ShieldCheck className="size-3 text-gold-2" /> No ad trackers or cookies</span>
            <span className="inline-flex items-center gap-1 rounded-full bg-white/5 px-2.5 py-1 ring-1 ring-white/10"><ShieldCheck className="size-3 text-gold-2" /> PHI only in the secure portal</span>
          </div>
        </div>
        <div>
          <div className="text-[13px] font-semibold uppercase tracking-[0.14em] text-gold-2">Practice</div>
          <ul className="mt-3 space-y-2 text-[14px] text-white/65">
            <li><Link href="/#specialties" className="transition hover:text-white">Specialties</Link></li>
            <li><Link href="/#team" className="transition hover:text-white">Our team</Link></li>
            <li><Link href="/#locations" className="transition hover:text-white">Locations</Link></li>
            <li><Link href="/#patients" className="transition hover:text-white">For patients</Link></li>
          </ul>
        </div>
        <div>
          <div className="text-[13px] font-semibold uppercase tracking-[0.14em] text-gold-2">Patients</div>
          <ul className="mt-3 space-y-2 text-[14px] text-white/65">
            <li><Link href="/portal" className="transition hover:text-white">Secure patient portal</Link></li>
            <li><Link href="/privacy" className="transition hover:text-white">Privacy &amp; HIPAA</Link></li>
            <li><Link href="/privacy#notice" className="transition hover:text-white">Notice of Privacy Practices</Link></li>
            <li><Link href="/privacy#rights" className="transition hover:text-white">Your health information rights</Link></li>
          </ul>
        </div>
        <div>
          <div className="text-[13px] font-semibold uppercase tracking-[0.14em] text-gold-2">Care platform</div>
          <ul className="mt-3 space-y-2 text-[14px] text-white/65">
            <li><Link href="/platform" className="transition hover:text-white">Command center</Link></li>
            <li><Link href="/platform/impact" className="transition hover:text-white">Impact calculator</Link></li>
            <li><Link href="/platform/compliance" className="transition hover:text-white">HIPAA &amp; security</Link></li>
            <li><Link href="/platform/architecture" className="transition hover:text-white">Architecture</Link></li>
          </ul>
        </div>
      </div>
      <div className="relative border-t border-white/10">
        <div className="mx-auto max-w-7xl px-4 py-6 sm:px-6">
          <div className="flex flex-col gap-3 rounded-2xl border border-white/10 bg-white/[0.04] p-4 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex items-start gap-3">
              <DemoPill className="mt-0.5 !text-gold" />
              <p className="text-[12.5px] leading-relaxed text-white/60">
                <b className="text-white">Concept demo: not a real medical practice.</b> Built to show how AI can streamline a multi-site medical practice and optimize its workflows.
                Provider names, phone numbers and all patient data are fictional or synthetic; no real patients.
              </p>
            </div>
            <div className="flex shrink-0 items-center gap-3">
              <span className="text-[12px] text-white/50">Designed &amp; engineered by Built by Oak · 2026</span>
              <a
                href="https://www.linkedin.com/in/mateogscarponi/"
                target="_blank"
                rel="noopener noreferrer"
                aria-label="LinkedIn profile (opens in a new tab)"
                title="LinkedIn"
                className="grid size-8 place-items-center rounded-lg bg-white/5 text-white/70 ring-1 ring-white/15 transition hover:bg-[#0a66c2] hover:text-white hover:ring-[#0a66c2]"
              >
                <svg viewBox="0 0 24 24" className="size-4" fill="currentColor" aria-hidden="true">
                  <path d="M20.45 20.45h-3.55v-5.57c0-1.33-.03-3.04-1.85-3.04-1.86 0-2.14 1.45-2.14 2.94v5.67H9.36V9h3.41v1.56h.05c.48-.9 1.64-1.85 3.37-1.85 3.6 0 4.27 2.37 4.27 5.46v6.28zM5.34 7.43a2.06 2.06 0 1 1 0-4.12 2.06 2.06 0 0 1 0 4.12zM7.12 20.45H3.56V9h3.56v11.45zM22.22 0H1.77C.79 0 0 .77 0 1.73v20.54C0 23.23.79 24 1.77 24h20.45c.98 0 1.78-.77 1.78-1.73V1.73C24 .77 23.2 0 22.22 0z" />
                </svg>
              </a>
            </div>
          </div>
        </div>
      </div>
    </footer>
  );
}
