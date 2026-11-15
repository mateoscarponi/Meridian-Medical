import Link from "next/link";
import { Lock, ShieldCheck } from "lucide-react";
import { Logo, DemoPill } from "./brand";
import { PRACTICE } from "@/lib/practice";

export function SiteFooter() {
  return (
    <footer className="border-t border-line bg-cream">
      <div className="mx-auto grid max-w-7xl gap-10 px-4 py-14 sm:px-6 md:grid-cols-5">
        <div className="md:col-span-2">
          <Logo />
          <p className="mt-4 max-w-sm text-[14px] text-muted">{PRACTICE.tagline} Orthopedic and spine care for {PRACTICE.region} since {PRACTICE.since}.</p>
          <div className="mt-5 flex flex-wrap gap-2 text-[11.5px] font-semibold text-muted">
            <span className="inline-flex items-center gap-1 rounded-full bg-white px-2.5 py-1 ring-1 ring-line"><Lock className="size-3 text-sage" /> HTTPS only</span>
            <span className="inline-flex items-center gap-1 rounded-full bg-white px-2.5 py-1 ring-1 ring-line"><ShieldCheck className="size-3 text-sage" /> No ad trackers or cookies</span>
            <span className="inline-flex items-center gap-1 rounded-full bg-white px-2.5 py-1 ring-1 ring-line"><ShieldCheck className="size-3 text-sage" /> PHI only in the secure portal</span>
          </div>
        </div>
        <div>
          <div className="text-[13px] font-semibold text-ink">Practice</div>
          <ul className="mt-3 space-y-2 text-[14px] text-muted">
            <li><Link href="/#specialties" className="hover:text-ink">Specialties</Link></li>
            <li><Link href="/#team" className="hover:text-ink">Our team</Link></li>
            <li><Link href="/#locations" className="hover:text-ink">Locations</Link></li>
            <li><Link href="/#patients" className="hover:text-ink">For patients</Link></li>
          </ul>
        </div>
        <div>
          <div className="text-[13px] font-semibold text-ink">Patients</div>
          <ul className="mt-3 space-y-2 text-[14px] text-muted">
            <li><Link href="/portal" className="hover:text-ink">Secure patient portal</Link></li>
            <li><Link href="/privacy" className="hover:text-ink">Privacy &amp; HIPAA</Link></li>
            <li><Link href="/privacy#notice" className="hover:text-ink">Notice of Privacy Practices</Link></li>
            <li><Link href="/privacy#rights" className="hover:text-ink">Your health information rights</Link></li>
          </ul>
        </div>
        <div>
          <div className="text-[13px] font-semibold text-ink">Care platform</div>
          <ul className="mt-3 space-y-2 text-[14px] text-muted">
            <li><Link href="/platform" className="hover:text-ink">Command center</Link></li>
            <li><Link href="/platform/impact" className="hover:text-ink">Impact calculator</Link></li>
            <li><Link href="/platform/compliance" className="hover:text-ink">HIPAA &amp; security</Link></li>
            <li><Link href="/platform/architecture" className="hover:text-ink">Architecture</Link></li>
          </ul>
        </div>
      </div>
      <div className="border-t border-line">
        <div className="mx-auto max-w-7xl px-4 py-6 sm:px-6">
          <div className="flex flex-col gap-3 rounded-2xl border border-line bg-white/70 p-4 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex items-start gap-3">
              <DemoPill className="mt-0.5 !text-gold" />
              <p className="text-[12.5px] leading-relaxed text-muted">
                <b className="text-ink">Concept demo: not a real medical practice.</b> Built to show how AI can streamline a multi-site medical practice and optimize its workflows.
                Provider names, phone numbers and all patient data are fictional or synthetic; no real patients.
              </p>
            </div>
            <span className="shrink-0 text-[12px] text-faint">Designed &amp; engineered by Mateo Scarponi · 2026</span>
          </div>
        </div>
      </div>
    </footer>
  );
}
