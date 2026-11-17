"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState, type ReactNode } from "react";
import {
  Bot, CalendarClock, ClipboardCheck, Database, FileScan, FileText, Gauge, LayoutDashboard, Menu, MessagesSquare,
  Network, Receipt, ScanText, Share2, ShieldCheck, Workflow, X, Layers, ArrowLeft, BadgeCheck, Briefcase, Lock,
} from "lucide-react";
import { useIdle } from "../use-idle";
import { BrandMark, DemoPill } from "../site/brand";
import { cx } from "../ui";

export const NAV_GROUPS = [
  { label: "Overview", items: [{ href: "/platform", label: "Command Center", icon: LayoutDashboard }, { href: "/platform/impact", label: "Impact & ROI", icon: Gauge }] },
  {
    label: "Revenue cycle",
    items: [
      { href: "/platform/prior-auth", label: "Prior Auth Copilot", icon: ClipboardCheck },
      { href: "/platform/eligibility", label: "Eligibility", icon: BadgeCheck },
      { href: "/platform/claims", label: "Claim Scrubber", icon: Receipt },
      { href: "/platform/wc-nofault", label: "WC & No-Fault Desk", icon: Briefcase },
    ],
  },
  {
    label: "Front office",
    items: [
      { href: "/platform/scheduling", label: "Smart Scheduling", icon: CalendarClock },
      { href: "/platform/referrals", label: "Referrals", icon: Share2 },
      { href: "/platform/messages", label: "Patient Messages", icon: MessagesSquare },
      { href: "/platform/agent", label: "AI Front-Desk Agent", icon: Bot },
    ],
  },
  {
    label: "Clinical AI",
    items: [
      { href: "/platform/scribe", label: "Clinical Scribe", icon: FileText },
      { href: "/platform/chart-nlp", label: "Chart NLP", icon: ScanText },
      { href: "/platform/documents", label: "Document AI", icon: FileScan },
    ],
  },
  {
    label: "Data & integration",
    items: [
      { href: "/platform/interop", label: "HL7 / FHIR Lab", icon: Network },
      { href: "/platform/pipeline", label: "Data Pipeline", icon: Database },
      { href: "/platform/automations", label: "Automations", icon: Workflow },
    ],
  },
  {
    label: "Trust & build",
    items: [
      { href: "/platform/compliance", label: "HIPAA & Security", icon: ShieldCheck },
      { href: "/platform/architecture", label: "Architecture", icon: Layers },
    ],
  },
];

function NavList({ onNavigate }: { onNavigate?: () => void }) {
  const path = usePathname();
  return (
    <nav className="space-y-6" aria-label="Platform">
      {NAV_GROUPS.map((g) => (
        <div key={g.label}>
          <div className="px-3 text-[10px] font-bold uppercase tracking-[0.16em] text-white/35">{g.label}</div>
          <ul className="mt-2 space-y-0.5">
            {g.items.map((it) => {
              const active = it.href === "/platform" ? path === "/platform" : path.startsWith(it.href);
              return (
                <li key={it.href}>
                  <Link
                    href={it.href}
                    onClick={onNavigate}
                    className={cx(
                      "flex items-center gap-2.5 rounded-lg px-3 py-2 text-[13.5px] font-medium transition-all",
                      active ? "bg-white/10 text-white shadow-[inset_2px_0_0_var(--color-gold-2)]" : "text-white/60 hover:bg-white/5 hover:text-white",
                    )}
                  >
                    <it.icon className={cx("size-4", active ? "text-gold-2" : "")} />
                    {it.label}
                  </Link>
                </li>
              );
            })}
          </ul>
        </div>
      ))}
    </nav>
  );
}

export function PlatformShell({ children }: { children: ReactNode }) {
  const [open, setOpen] = useState(false);
  const [manualLock, setManualLock] = useState(false);
  const { idle, reset } = useIdle(15 * 60 * 1000);
  const locked = idle || manualLock;
  return (
    <div className="min-h-screen bg-cream">
      {locked && (
        <div className="fixed inset-0 z-[100] grid place-items-center bg-ink/75 p-4 backdrop-blur-xl" role="dialog" aria-modal="true" aria-label="Session locked">
          <div className="animate-fade-up w-full max-w-sm rounded-3xl bg-white p-7 text-center shadow-[var(--shadow-lift)]">
            <div className="mx-auto grid size-12 place-items-center rounded-full bg-navy text-white"><Lock className="size-5" /></div>
            <h2 className="mt-4 font-display text-xl font-semibold text-ink">Session locked</h2>
            <p className="mt-1.5 text-[13px] text-muted">{idle ? "Locked automatically after 15 minutes of inactivity" : "Locked by user"} to protect patient information on unattended screens.</p>
            <button onClick={() => { reset(); setManualLock(false); }} className="mt-5 w-full rounded-xl bg-navy px-4 py-2.5 text-[14px] font-semibold text-white transition hover:bg-ocean">Unlock</button>
            <p className="mt-2 text-[11px] text-faint">Demo unlock. Production re-authenticates with SSO + MFA.</p>
          </div>
        </div>
      )}
      <aside className="scroll-thin fixed inset-y-0 left-0 z-40 hidden w-64 flex-col overflow-y-auto bg-ink px-3 py-5 lg:flex">
        <Link href="/" className="flex items-center gap-2.5 px-3">
          <BrandMark light className="size-9" />
          <span className="leading-tight">
            <span className="block font-display text-[15px] font-semibold text-white">Meridian Orthopedics</span>
            <span className="block text-[10px] font-semibold uppercase tracking-[0.16em] text-white/45">Care Operations</span>
          </span>
        </Link>
        <div className="mt-8 flex-1">
          <NavList />
        </div>
        <div className="mt-8 rounded-xl border border-white/10 bg-white/[0.04] p-3 text-[11px] leading-relaxed text-white/55">
          <div className="mb-1 flex items-center gap-2"><DemoPill /><span className="font-semibold text-white/80">Synthetic data</span></div>
          No real patients. Built to show how AI can streamline a multi-site medical practice.
          <div className="mt-2 flex items-center gap-1.5 text-white/70"><ShieldCheck className="size-3 text-sage-2" /> HIPAA safeguards demonstrated</div>
        </div>
      </aside>

      {open && (
        <div className="fixed inset-0 z-50 lg:hidden">
          <div className="absolute inset-0 bg-ink/50 backdrop-blur-sm" onClick={() => setOpen(false)} />
          <div className="scroll-thin absolute inset-y-0 left-0 w-72 overflow-y-auto bg-ink px-3 py-5">
            <div className="mb-6 flex items-center justify-between px-3">
              <span className="font-display text-[15px] font-semibold text-white">Meridian Orthopedics</span>
              <button onClick={() => setOpen(false)} className="text-white/70" aria-label="Close navigation"><X className="size-5" /></button>
            </div>
            <NavList onNavigate={() => setOpen(false)} />
          </div>
        </div>
      )}

      <div className="lg:pl-64">
        <header className="sticky top-0 z-30 border-b border-line bg-cream/85 backdrop-blur-xl">
          <div className="flex items-center justify-between gap-4 px-4 py-3 sm:px-8">
            <div className="flex items-center gap-3">
              <button onClick={() => setOpen(true)} className="rounded-lg p-1.5 text-ink lg:hidden" aria-label="Open navigation"><Menu className="size-5" /></button>
              <Link href="/" className="hidden items-center gap-1.5 text-[13px] font-medium text-muted hover:text-ink sm:inline-flex">
                <ArrowLeft className="size-3.5" /> Practice website
              </Link>
            </div>
            <div className="flex items-center gap-3">
              <span className="hidden items-center gap-1.5 rounded-full bg-sage-soft px-2.5 py-1 text-[11px] font-semibold text-sage xl:inline-flex">
                <span className="size-1.5 animate-pulse-soft rounded-full bg-sage" /> All integrations healthy
              </span>
              <Link href="/platform/compliance" className="hidden items-center gap-1.5 rounded-full bg-sky-2 px-2.5 py-1 text-[11px] font-semibold text-ocean hover:bg-sky md:inline-flex" title="HIPAA safeguards active: audit logging, RBAC, auto-lock after 15 minutes idle">
                <ShieldCheck className="size-3" /> HIPAA safeguards on · auto-lock 15 min
              </Link>
              <button onClick={() => setManualLock(true)} className="grid size-8 place-items-center rounded-full text-muted ring-1 ring-line-2 transition hover:bg-sand hover:text-ink" aria-label="Lock screen" title="Lock screen">
                <Lock className="size-3.5" />
              </button>
              <DemoPill className="!text-gold" />
              <div className="flex items-center gap-2">
                <div className="grid size-8 place-items-center rounded-full bg-navy text-[12px] font-semibold text-white">OM</div>
                <div className="hidden leading-tight sm:block">
                  <div className="text-[13px] font-semibold text-ink">Operations Manager</div>
                  <div className="text-[11px] text-muted">All locations</div>
                </div>
              </div>
            </div>
          </div>
        </header>
        <main className="px-4 py-8 sm:px-8">{children}</main>
      </div>
    </div>
  );
}

export function ToolHeader({
  eyebrow, title, description, tags = [], impact,
}: { eyebrow: string; title: string; description: string; tags?: string[]; impact?: { value: string; label: string }[] }) {
  return (
    <div className="mb-8 flex flex-wrap items-end justify-between gap-6">
      <div className="max-w-3xl">
        <div className="text-[12px] font-semibold uppercase tracking-[0.16em] text-gold">{eyebrow}</div>
        <h1 className="mt-2 font-display text-[34px] leading-tight font-semibold tracking-tight text-ink">{title}</h1>
        <p className="mt-2 text-[15px] leading-relaxed text-muted">{description}</p>
        {tags.length > 0 && (
          <div className="mt-4 flex flex-wrap gap-1.5">
            {tags.map((t) => (
              <span key={t} className="rounded-md bg-white px-2 py-1 font-mono text-[11px] text-muted ring-1 ring-inset ring-line">{t}</span>
            ))}
          </div>
        )}
      </div>
      {impact && (
        <div className="flex gap-3">
          {impact.map((i) => (
            <div key={i.label} className="rounded-2xl border border-line bg-white px-4 py-3 shadow-[var(--shadow-soft)]">
              <div className="font-display text-2xl font-semibold text-sage tabular-nums">{i.value}</div>
              <div className="text-[11px] font-medium text-muted">{i.label}</div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
