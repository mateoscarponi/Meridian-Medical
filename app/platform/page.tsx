"use client";

import Link from "next/link";
import { useState } from "react";
import { ArrowUpRight, Check, CircleAlert, Compass, ClipboardCheck, Lightbulb, Plug, Receipt, Share2, CalendarClock, Timer, Network, Workflow, Brain } from "lucide-react";
import { ToolHeader } from "@/components/platform/shell";
import { Badge, Button, Card, CardHeader, Progress, Stat, cx } from "@/components/ui";
import { PairedBars, TrendLine } from "@/components/platform/charts";

const SCORECARD = [
  { goal: "Reduced administrative workload", kpi: "Staff hours on manual admin / week", base: "318 h", now: "188 h", target: "170 h", pct: 88, href: "/platform/impact", icon: Timer },
  { goal: "Faster authorizations & documentation", kpi: "Median auth turnaround · note close time", base: "10 d · 2.1 d", now: "2.6 d · same day", target: "2 d · same day", pct: 92, href: "/platform/prior-auth", icon: ClipboardCheck },
  { goal: "Improved interoperability", kpi: "Systems connected via HL7 / FHIR / X12", base: "2", now: "9", target: "10", pct: 90, href: "/platform/interop", icon: Network },
  { goal: "Operational efficiency through automation", kpi: "Share of repeatable tasks automated", base: "8%", now: "71%", target: "75%", pct: 95, href: "/platform/automations", icon: Workflow },
  { goal: "AI tools that measurably improve workflows", kpi: "AI tools in production with a tracked KPI", base: "0", now: "7 of 7", target: "7", pct: 100, href: "/platform/impact", icon: Brain },
];

const ACTIONS = [
  { id: 1, icon: ClipboardCheck, title: "3 Aetna authorizations pending > 5 business days", detail: "Copilot drafted peer-to-peer requests with the chart evidence attached.", cta: "Send P2P requests", href: "/platform/prior-auth", tone: "warn" as const },
  { id: 2, icon: CalendarClock, title: "Westbury, Thu 2–4 PM: 4 open slots, 6 waitlist matches", detail: "Matched by visit type, insurance and distance. Expected fill rate 83%.", cta: "Text offers", href: "/platform/scheduling", tone: "ocean" as const },
  { id: 3, icon: Receipt, title: "2 claims at high denial risk ($2,480)", detail: "Bundled 77003 with 64483 and an invalid M54.5 code. Fixes are pre-filled.", cta: "Apply fixes", href: "/platform/claims", tone: "danger" as const },
  { id: 4, icon: Share2, title: "Referral loop open 6 days: Dr. Patel's office", detail: "Consult note signed 11/12. Auto-send via Direct secure messaging.", cta: "Close the loop", href: "/platform/referrals", tone: "ocean" as const },
];

const QUEUES = [
  { team: "Front desk", items: 14, sla: 96, oldest: "18 min" },
  { team: "Authorizations", items: 22, sla: 91, oldest: "2.1 d" },
  { team: "Billing", items: 37, sla: 88, oldest: "3 d" },
  { team: "Clinical inbox", items: 9, sla: 99, oldest: "41 min" },
];

const INTEGRATIONS = [
  { name: "EMR · FHIR R4", status: "ok", latency: "118 ms" },
  { name: "HL7 ADT / SIU feed", status: "ok", latency: "1.2 s" },
  { name: "Clearinghouse X12", status: "ok", latency: "4.0 s" },
  { name: "Fax → Document AI", status: "ok", latency: "38 s" },
  { name: "SMS / IVR gateway", status: "ok", latency: "0.6 s" },
  { name: "NY WCB Medical Portal", status: "degraded", latency: "retrying" },
];

const TOUR = [
  { href: "/#request", label: "Smart intake", hint: "patient-side triage & routing" },
  { href: "/platform/prior-auth", label: "Prior Auth Copilot", hint: "run it on PA-24812" },
  { href: "/platform/documents", label: "Document AI", hint: "press Process all" },
  { href: "/platform/claims", label: "Claim Scrubber", hint: "fix all safe edits" },
  { href: "/platform/agent", label: "AI Agent", hint: "play the Spanish scenario" },
  { href: "/platform/interop", label: "HL7 / FHIR Lab", hint: "POST a real message" },
  { href: "/platform/impact", label: "Impact & ROI", hint: "plug in real volumes" },
];

export default function CommandCenter() {
  const [done, setDone] = useState<number[]>([]);
  const [tour, setTour] = useState(true);
  return (
    <div className="mx-auto max-w-7xl">
      <ToolHeader
        eyebrow="Monday, November 16 · All locations"
        title="Good morning. Here's where the practice stands."
        description="One view of the five outcomes this platform is built to move, the work waiting on each team, and AI-recommended next actions."
        tags={["decision-support", "real-time KPIs", "work queues", "SLA tracking"]}
      />

      {tour && (
        <Card className="mb-6 overflow-hidden border-ocean/20 bg-gradient-to-r from-sky-2 via-white to-sage-soft/50">
          <div className="flex flex-wrap items-center justify-between gap-3 px-5 pt-4">
            <div className="flex items-center gap-2 text-[13px] font-semibold text-ink"><Compass className="size-4 text-ocean" /> 5-minute guided tour</div>
            <button onClick={() => setTour(false)} className="text-[12px] font-semibold text-muted hover:text-ink">Hide</button>
          </div>
          <ol className="scroll-thin flex gap-2 overflow-x-auto px-5 pb-4 pt-3">
            {TOUR.map((t, i) => (
              <li key={t.href} className="shrink-0">
                <Link href={t.href} className="group flex items-center gap-2.5 rounded-xl bg-white px-3 py-2 ring-1 ring-line transition hover:ring-ocean">
                  <span className="grid size-6 place-items-center rounded-full bg-navy text-[11px] font-bold text-white">{i + 1}</span>
                  <span>
                    <span className="block text-[12.5px] font-semibold text-ink group-hover:text-ocean">{t.label}</span>
                    <span className="block text-[11px] text-muted">{t.hint}</span>
                  </span>
                </Link>
              </li>
            ))}
          </ol>
        </Card>
      )}

      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Stat label="Admin hours saved this week" value="130 h" delta="≈ 3.3 FTE" hint="vs. pre-automation baseline" />
        <Stat label="Median auth turnaround" value="2.6 d" delta="−74%" hint="baseline: up to 10 business days" />
        <Stat label="Clean claim rate" value="97.1%" delta="+9.4 pts" hint="first-pass acceptance" />
        <Stat label="No-show rate" value="8.6%" delta="−4.8 pts" hint="model-driven reminders & backfill" />
      </div>

      <div className="mt-6 grid gap-6 xl:grid-cols-[1.35fr_1fr]">
        <Card>
          <CardHeader title="What success looks like" subtitle="Baseline at go-live → today → 90-day target" icon={<Check className="size-4" />} />
          <ul className="divide-y divide-line">
            {SCORECARD.map((s) => (
              <li key={s.goal}>
                <Link href={s.href} className="group grid gap-3 px-5 py-4 transition hover:bg-cream sm:grid-cols-[1.4fr_1fr] sm:items-center">
                  <div className="flex items-start gap-3">
                    <s.icon className="mt-0.5 size-4 shrink-0 text-ocean" />
                    <div>
                      <div className="flex items-center gap-1 text-[14px] font-semibold text-ink">{s.goal}<ArrowUpRight className="size-3.5 text-faint opacity-0 transition group-hover:opacity-100" /></div>
                      <div className="text-[12px] text-muted">{s.kpi}</div>
                    </div>
                  </div>
                  <div>
                    <div className="mb-1.5 flex items-center justify-between text-[12px] tabular-nums">
                      <span className="text-faint line-through decoration-faint/50">{s.base}</span>
                      <span className="font-semibold text-ink">{s.now}</span>
                      <span className="text-muted">→ {s.target}</span>
                    </div>
                    <Progress value={s.pct} tone="sage" />
                  </div>
                </Link>
              </li>
            ))}
          </ul>
        </Card>

        <Card>
          <CardHeader title="Recommended actions" subtitle="Generated from live queues and payer data" icon={<Lightbulb className="size-4" />} />
          <ul className="divide-y divide-line">
            {ACTIONS.map((a) => {
              const isDone = done.includes(a.id);
              return (
                <li key={a.id} className={cx("px-5 py-4 transition", isDone && "bg-sage-soft/40")}>
                  <div className="flex items-start gap-3">
                    <div className={cx("mt-0.5 grid size-8 shrink-0 place-items-center rounded-lg", isDone ? "bg-sage text-white" : "bg-sand text-ink")}>
                      {isDone ? <Check className="size-4" /> : <a.icon className="size-4" />}
                    </div>
                    <div className="min-w-0 flex-1">
                      <div className={cx("text-[14px] font-semibold", isDone ? "text-sage" : "text-ink")}>{a.title}</div>
                      <div className="mt-0.5 text-[12px] text-muted">{isDone ? "Done: logged to the audit trail and the owning queue." : a.detail}</div>
                      {!isDone && (
                        <div className="mt-2.5 flex gap-2">
                          <Button size="sm" variant={a.tone === "danger" ? "dark" : "primary"} onClick={() => setDone((d) => [...d, a.id])}>{a.cta}</Button>
                          <Link href={a.href} className="inline-flex items-center rounded-xl px-3 py-1.5 text-[13px] font-semibold text-ocean hover:bg-sky-2">Review</Link>
                        </div>
                      )}
                    </div>
                  </div>
                </li>
              );
            })}
          </ul>
        </Card>
      </div>

      <div className="mt-6 grid gap-6 lg:grid-cols-2">
        <Card>
          <CardHeader title="Staff hours per week, by workflow" subtitle="Before the platform vs. the last 4 weeks (average)" />
          <div className="p-5">
            <PairedBars
              aLabel="Before"
              bLabel="Now"
              unit=" h"
              data={[
                { label: "Prior authorizations", a: 96, b: 34 },
                { label: "Fax & document intake", a: 58, b: 14 },
                { label: "Phone & messages", a: 72, b: 49 },
                { label: "Claim edits & denials", a: 52, b: 27 },
                { label: "Referral coordination", a: 40, b: 18 },
              ]}
            />
          </div>
        </Card>
        <Card>
          <CardHeader title="Median authorization turnaround" subtitle="Business days from order to payer decision, weekly" />
          <div className="p-5">
            <TrendLine
              label="Median authorization turnaround by week"
              unit=" d"
              target={2}
              targetLabel="Target"
              points={[9.6, 9.1, 8.8, 7.4, 6.1, 5.2, 4.4, 3.9, 3.4, 3.1, 2.8, 2.6].map((y, i) => ({ x: `W${i + 1}`, y }))}
            />
          </div>
        </Card>
      </div>

      <div className="mt-6 grid gap-6 lg:grid-cols-[1fr_1fr]">
        <Card>
          <CardHeader title="Work queues" subtitle="Items waiting · SLA attainment · oldest item" />
          <div className="grid gap-px bg-line sm:grid-cols-2">
            {QUEUES.map((q) => (
              <div key={q.team} className="bg-white p-5">
                <div className="flex items-center justify-between">
                  <div className="text-[13px] font-semibold text-ink">{q.team}</div>
                  <Badge tone={q.sla >= 95 ? "sage" : q.sla >= 90 ? "ocean" : "warn"}>{q.sla}% SLA</Badge>
                </div>
                <div className="mt-2 font-display text-3xl font-semibold text-ink tabular-nums">{q.items}</div>
                <div className="text-[12px] text-muted">oldest: {q.oldest}</div>
              </div>
            ))}
          </div>
        </Card>
        <Card>
          <CardHeader title="Integration health" subtitle="Interfaces monitored every 60 seconds" icon={<Plug className="size-4" />} />
          <ul className="divide-y divide-line">
            {INTEGRATIONS.map((i) => (
              <li key={i.name} className="flex items-center justify-between px-5 py-3 text-[13px]">
                <span className="flex items-center gap-2.5 font-medium text-ink">
                  <span className={cx("size-2 rounded-full", i.status === "ok" ? "bg-sage" : "animate-pulse-soft bg-warn")} />
                  {i.name}
                </span>
                <span className="flex items-center gap-3">
                  <span className="font-mono text-[12px] text-muted">{i.latency}</span>
                  {i.status === "ok" ? <Badge tone="sage">Healthy</Badge> : <Badge tone="warn"><CircleAlert className="size-3" /> Degraded · queued</Badge>}
                </span>
              </li>
            ))}
          </ul>
        </Card>
      </div>
    </div>
  );
}
