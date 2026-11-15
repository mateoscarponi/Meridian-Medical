"use client";

import { useState } from "react";
import { ArrowRight, Clock, Mail, Printer, Share2, Zap, Network } from "lucide-react";
import { ToolHeader } from "@/components/platform/shell";
import { Badge, Button, Card, CardHeader, Stat, cx } from "@/components/ui";

const STAGES = ["Received", "Insurance verified", "Scheduled", "Seen", "Loop closed"] as const;
type Stage = (typeof STAGES)[number];

interface Ref { id: string; patient: string; from: string; via: "Fax" | "Direct" | "FHIR" | "Portal"; reason: string; stage: Stage; days: number; auto?: string }

const INITIAL: Ref[] = [
  { id: "R-3301", patient: "Carmen Diaz", from: "Patel Family Medicine", via: "Fax", reason: "Right sciatica, 5 wk", stage: "Received", days: 0 },
  { id: "R-3302", patient: "Grace Kim", from: "Northwell PCP – Garden City", via: "FHIR", reason: "Neck pain w/ arm numbness", stage: "Received", days: 1 },
  { id: "R-3303", patient: "Luis Moreno", from: "Queens Medical Assoc.", via: "Direct", reason: "Lumbar stenosis", stage: "Insurance verified", days: 1 },
  { id: "R-3304", patient: "Ana Pereira", from: "Go Rehab PT", via: "Portal", reason: "Failed PT, consider ESI", stage: "Insurance verified", days: 3 },
  { id: "R-3305", patient: "Priya Natarajan", from: "Garden City Pediatrics & Adult", via: "Fax", reason: "Adult scoliosis", stage: "Scheduled", days: 2 },
  { id: "R-3306", patient: "Thomas Russo", from: "Dr. Feld – Internal Medicine", via: "Direct", reason: "C5-C6 radiculopathy", stage: "Seen", days: 6 },
  { id: "R-3307", patient: "Linda Chen", from: "Flushing Family Care", via: "FHIR", reason: "Claudication", stage: "Seen", days: 2 },
  { id: "R-3308", patient: "Harold Becker", from: "Babylon Medical Group", via: "Fax", reason: "Lumbar spondylosis", stage: "Loop closed", days: 0 },
];

const VIA_ICON = { Fax: Printer, Direct: Mail, FHIR: Network, Portal: Share2 };
const NEXT_ACTION: Record<Stage, string> = {
  Received: "Auto: match patient → check eligibility (270/271)",
  "Insurance verified": "Auto: text patient 3 slot options",
  Scheduled: "Auto: reminders + pre-visit checklist",
  Seen: "Auto: send signed consult note to referrer",
  "Loop closed": "Done",
};

export default function ReferralsPage() {
  const [refs, setRefs] = useState(INITIAL);
  const [autoOn, setAutoOn] = useState(true);
  const advance = (id: string) =>
    setRefs((rs) => rs.map((r) => (r.id === id && r.stage !== "Loop closed" ? { ...r, stage: STAGES[STAGES.indexOf(r.stage) + 1], days: 0, auto: autoOn ? NEXT_ACTION[r.stage].replace("Auto: ", "") : undefined } : r)));
  const runAll = () => setRefs((rs) => rs.map((r) => (r.stage === "Loop closed" ? r : { ...r, stage: STAGES[STAGES.indexOf(r.stage) + 1], days: 0, auto: NEXT_ACTION[r.stage].replace("Auto: ", "") })));
  const stalled = refs.filter((r) => r.days >= 3 && r.stage !== "Loop closed").length;

  return (
    <div className="mx-auto max-w-7xl">
      <ToolHeader
        eyebrow="Front office · Referral management"
        title="Referral Tracker"
        description="Every referral, whether fax, Direct message, FHIR or portal, moves through one pipeline with SLAs. Automations do the repetitive steps and close the loop with the referring physician, which is what keeps referrals coming."
        tags={["Direct secure messaging", "FHIR ServiceRequest", "eligibility 270/271", "loop closure", "leakage analytics"]}
        impact={[{ value: "5.8→1.9 d", label: "referral to appointment" }, { value: "6%", label: "leakage (was 22%)" }]}
      />

      <div className="mb-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Stat label="Open referrals" value={refs.filter((r) => r.stage !== "Loop closed").length} tone="neutral" />
        <Stat label="Stalled ≥ 3 days" value={stalled} tone={stalled ? "danger" : "sage"} delta={stalled ? "needs attention" : "none"} />
        <Stat label="Loops closed this month" value="142" delta="98%" hint="consult notes returned to referrers" />
        <Stat label="Top referrer" value="Northwell PCPs" hint="31 referrals · 30 days" tone="neutral" />
      </div>

      <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
        <label className="inline-flex cursor-pointer items-center gap-2 text-[13px] font-semibold text-ink">
          <button role="switch" aria-checked={autoOn} onClick={() => setAutoOn((v) => !v)} className={cx("relative h-6 w-11 rounded-full transition", autoOn ? "bg-sage" : "bg-line-2")}>
            <span className={cx("absolute top-0.5 size-5 rounded-full bg-white shadow transition-all", autoOn ? "left-[22px]" : "left-0.5")} />
          </button>
          Automations {autoOn ? "on" : "off"}
        </label>
        <Button size="sm" onClick={runAll}><Zap className="size-3.5" /> Run automations on all</Button>
      </div>

      <div className="scroll-thin -mx-4 overflow-x-auto px-4 pb-2 sm:-mx-8 sm:px-8">
        <div className="grid min-w-[1100px] grid-cols-5 gap-4">
          {STAGES.map((s) => {
            const items = refs.filter((r) => r.stage === s);
            return (
              <div key={s} className="rounded-2xl bg-sand/70 p-3">
                <div className="mb-3 flex items-center justify-between px-1">
                  <span className="text-[12px] font-bold uppercase tracking-wide text-muted">{s}</span>
                  <span className="rounded-full bg-white px-2 text-[11px] font-semibold text-ink">{items.length}</span>
                </div>
                <div className="space-y-2.5">
                  {items.map((r) => {
                    const Icon = VIA_ICON[r.via];
                    return (
                      <Card key={r.id} className="animate-fade-up p-3.5">
                        <div className="flex items-center justify-between">
                          <span className="font-mono text-[10.5px] text-faint">{r.id}</span>
                          <span className="inline-flex items-center gap-1 text-[11px] text-muted"><Icon className="size-3" />{r.via}</span>
                        </div>
                        <div className="mt-1 text-[13.5px] font-semibold text-ink">{r.patient}</div>
                        <div className="text-[12px] text-muted">{r.reason}</div>
                        <div className="mt-1 truncate text-[11px] text-faint">from {r.from}</div>
                        {r.auto && <div className="mt-2 rounded-md bg-sage-soft px-2 py-1 text-[11px] text-sage">✓ {r.auto}</div>}
                        <div className="mt-2.5 flex items-center justify-between">
                          {r.days >= 3 && s !== "Loop closed" ? <Badge tone="danger"><Clock className="size-3" /> {r.days} d</Badge> : <span className="text-[11px] text-faint">{r.days === 0 ? "today" : `${r.days} d in stage`}</span>}
                          {s !== "Loop closed" && (
                            <button onClick={() => advance(r.id)} className="inline-flex items-center gap-1 rounded-md px-2 py-1 text-[11px] font-semibold text-ocean hover:bg-sky-2">Advance <ArrowRight className="size-3" /></button>
                          )}
                        </div>
                      </Card>
                    );
                  })}
                </div>
              </div>
            );
          })}
        </div>
      </div>

      <Card className="mt-6">
        <CardHeader title="Automation rules" subtitle="What runs at each stage when automations are on" />
        <div className="grid gap-px bg-line sm:grid-cols-2 lg:grid-cols-4">
          {STAGES.slice(0, 4).map((s) => (
            <div key={s} className="bg-white p-4">
              <div className="text-[12px] font-bold uppercase tracking-wide text-faint">{s}</div>
              <div className="mt-1.5 text-[13px] text-ink">{NEXT_ACTION[s].replace("Auto: ", "")}</div>
            </div>
          ))}
        </div>
      </Card>
    </div>
  );
}
