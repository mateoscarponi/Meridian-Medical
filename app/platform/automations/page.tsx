"use client";

import { useState } from "react";
import { Bolt, CheckCircle2, CircleDashed, GitBranch, Loader2, Play, UserCheck, Workflow } from "lucide-react";
import { ToolHeader } from "@/components/platform/shell";
import { Badge, Button, Card, CardHeader, cx } from "@/components/ui";

interface Node { kind: "trigger" | "step" | "branch" | "human"; title: string; system: string; ms: number; log: string }
interface Recipe { id: string; name: string; desc: string; runsWeek: number; minSaved: number; success: number; enabled: boolean; nodes: Node[] }

const RECIPES: Recipe[] = [
  {
    id: "fax-intake", name: "Fax referral → booked appointment", desc: "From an inbound fax to a scheduled new-patient visit without retyping.", runsWeek: 86, minSaved: 14, success: 96, enabled: true,
    nodes: [
      { kind: "trigger", title: "Fax received", system: "eFax", ms: 300, log: "DOC-5521 received from (516) 555-0172, 2 pages" },
      { kind: "step", title: "Read & classify", system: "Document AI", ms: 900, log: "Referral (98%) · 7 fields extracted" },
      { kind: "step", title: "Match or create patient", system: "MPI · EMR FHIR", ms: 500, log: "Matched MOS-205518 (0.97)" },
      { kind: "step", title: "Verify eligibility", system: "Clearinghouse 270/271", ms: 800, log: "Empire BCBS active · specialist copay $40 · referral not required" },
      { kind: "branch", title: "Prior auth needed for first visit?", system: "Payer rules", ms: 300, log: "No → continue" },
      { kind: "step", title: "Offer 3 slots by SMS", system: "Scheduling · SMS", ms: 700, log: "Patient picked Tue 10:45 AM Westbury" },
      { kind: "step", title: "Send forms & imaging link", system: "Portal", ms: 400, log: "Intake (ES) + HIPAA + imaging upload sent" },
      { kind: "step", title: "Notify referring office", system: "Direct messaging", ms: 400, log: "Appointment confirmation sent to Patel Family Medicine" },
    ],
  },
  {
    id: "auth-chaser", name: "Authorization status chaser", desc: "Polls payers, updates orders, nudges humans only when needed.", runsWeek: 140, minSaved: 9, success: 99, enabled: true,
    nodes: [
      { kind: "trigger", title: "Every hour, 7am–7pm", system: "Scheduler", ms: 300, log: "22 pending authorizations" },
      { kind: "step", title: "Check status", system: "X12 276/278 · payer APIs", ms: 1000, log: "4 approved · 1 pended for info · 17 unchanged" },
      { kind: "step", title: "Write auth # to orders & claims", system: "EMR · Billing", ms: 500, log: "4 orders updated (REF*G1)" },
      { kind: "branch", title: "Pended for more information?", system: "Rules", ms: 300, log: "Yes (1) → Copilot assembles missing records" },
      { kind: "human", title: "Auth specialist approves resubmission", system: "Work queue", ms: 800, log: "Approved by J. Alvarez" },
      { kind: "step", title: "Offer procedure slots to patients", system: "Scheduling · SMS", ms: 500, log: "4 patients texted · 3 booked" },
    ],
  },
  {
    id: "denial-appeal", name: "Denial → appeal draft", desc: "Reads the 835, finds the root cause, drafts the appeal with chart evidence.", runsWeek: 31, minSaved: 25, success: 88, enabled: true,
    nodes: [
      { kind: "trigger", title: "835 remittance with denial", system: "Clearinghouse", ms: 300, log: "CLM-90122 · CARC 197 (precert absent)" },
      { kind: "step", title: "Root-cause analysis", system: "Rules + LLM", ms: 900, log: "Auth existed (HUM-AUTH-8841207) but was missing on the claim" },
      { kind: "branch", title: "Correctable without appeal?", system: "Rules", ms: 300, log: "Yes → corrected claim (frequency code 7)" },
      { kind: "step", title: "Submit corrected claim", system: "837P", ms: 600, log: "Resubmitted with REF*G1" },
      { kind: "step", title: "Add scrubber rule", system: "Claim Scrubber", ms: 400, log: "Payer-specific auth check strengthened" },
    ],
  },
  {
    id: "post-op", name: "Post-op check-in", desc: "Day 1 and day 7 after surgery or injection, with red-flag escalation.", runsWeek: 44, minSaved: 11, success: 97, enabled: false,
    nodes: [
      { kind: "trigger", title: "Procedure completed", system: "EMR (HL7 SIU/DFT)", ms: 300, log: "ACDF C5-C6 · Thomas Russo" },
      { kind: "step", title: "Day-1 SMS questionnaire", system: "SMS", ms: 700, log: "Pain 4/10 · no fever · eating & drinking" },
      { kind: "branch", title: "Any red flag?", system: "NLP triage", ms: 400, log: "No → schedule day-7 check" },
      { kind: "step", title: "Day-7 check & PT referral", system: "SMS · Go Rehab", ms: 600, log: "PT referral sent; first visit booked" },
    ],
  },
];

const KIND_ICON = { trigger: Bolt, step: Workflow, branch: GitBranch, human: UserCheck };

export default function AutomationsPage() {
  const [sel, setSel] = useState(RECIPES[0].id);
  const [enabled, setEnabled] = useState<Record<string, boolean>>(() => Object.fromEntries(RECIPES.map((r) => [r.id, r.enabled])));
  const [progress, setProgress] = useState<number>(-1);
  const [running, setRunning] = useState(false);
  const r = RECIPES.find((x) => x.id === sel)!;
  const totalMs = r.nodes.reduce((s, n) => s + n.ms, 0);

  const simulate = () => {
    setRunning(true);
    setProgress(0);
    let t = 0;
    r.nodes.forEach((n, i) => {
      t += n.ms;
      setTimeout(() => {
        setProgress(i + 1);
        if (i === r.nodes.length - 1) setRunning(false);
      }, t);
    });
  };

  const weeklyHours = RECIPES.filter((x) => enabled[x.id]).reduce((s, x) => s + (x.runsWeek * x.minSaved) / 60, 0);

  return (
    <div className="mx-auto max-w-7xl">
      <ToolHeader
        eyebrow="Data & integration · Workflow automation"
        title="Automations"
        description="Repeatable workflows built as event-driven recipes that span the EMR, payers, fax, SMS and staff queues. Humans approve the steps that need judgment, and every run is timed, so the savings are measured rather than estimated."
        tags={["event-driven", "idempotent steps", "retries + dead-letter", "human-in-the-loop", "run analytics"]}
        impact={[{ value: `${weeklyHours.toFixed(0)} h`, label: "saved / week (enabled)" }, { value: `${RECIPES.filter((x) => enabled[x.id]).length}/${RECIPES.length}`, label: "recipes on" }]}
      />

      <div className="grid gap-6 xl:grid-cols-[360px_1fr]">
        <div className="space-y-3">
          {RECIPES.map((x) => (
            <Card key={x.id} className={cx("cursor-pointer p-4 transition", sel === x.id ? "ring-2 ring-ocean" : "hover:shadow-[var(--shadow-lift)]")} onClick={() => { setSel(x.id); setProgress(-1); }}>
              <div className="flex items-start justify-between gap-3">
                <div>
                  <div className="text-[14px] font-semibold text-ink">{x.name}</div>
                  <div className="mt-0.5 text-[12px] text-muted">{x.desc}</div>
                </div>
                <button
                  role="switch"
                  aria-checked={enabled[x.id]}
                  aria-label={`Enable ${x.name}`}
                  onClick={(e) => { e.stopPropagation(); setEnabled((s) => ({ ...s, [x.id]: !s[x.id] })); }}
                  className={cx("relative h-5 w-9 shrink-0 rounded-full transition", enabled[x.id] ? "bg-sage" : "bg-line-2")}
                >
                  <span className={cx("absolute top-0.5 size-4 rounded-full bg-white shadow transition-all", enabled[x.id] ? "left-[18px]" : "left-0.5")} />
                </button>
              </div>
              <div className="mt-3 grid grid-cols-3 gap-2 text-center">
                <div className="rounded-lg bg-cream py-1.5"><div className="text-[13px] font-semibold text-ink">{x.runsWeek}</div><div className="text-[10px] text-muted">runs/wk</div></div>
                <div className="rounded-lg bg-cream py-1.5"><div className="text-[13px] font-semibold text-ink">{x.minSaved}m</div><div className="text-[10px] text-muted">saved/run</div></div>
                <div className="rounded-lg bg-cream py-1.5"><div className="text-[13px] font-semibold text-ink">{x.success}%</div><div className="text-[10px] text-muted">no-touch</div></div>
              </div>
            </Card>
          ))}
        </div>

        <Card>
          <CardHeader
            title={r.name}
            subtitle={`${r.nodes.length} steps · simulated run ≈ ${(totalMs / 1000).toFixed(1)} s · manual equivalent ≈ ${r.minSaved} min`}
            right={<Button onClick={simulate} disabled={running}>{running ? <Loader2 className="size-4 animate-spin" /> : <Play className="size-4" />} Simulate run</Button>}
          />
          <div className="p-6">
            <ol className="relative space-y-3">
              {r.nodes.map((n, i) => {
                const Icon = KIND_ICON[n.kind];
                const done = progress > i;
                const active = running && progress === i;
                return (
                  <li key={i} className="relative flex gap-4">
                    {i < r.nodes.length - 1 && <div className={cx("absolute left-[19px] top-10 h-[calc(100%-16px)] w-0.5 transition-colors duration-500", done ? "bg-sage" : "bg-line")} />}
                    <div className={cx("z-10 grid size-10 shrink-0 place-items-center rounded-xl transition-all duration-300", done ? "bg-sage text-white" : active ? "bg-ocean text-white shadow-[var(--shadow-lift)]" : n.kind === "trigger" ? "bg-gold-soft text-gold" : n.kind === "human" ? "bg-sky-2 text-ocean" : "bg-sand text-muted")}>
                      {active ? <Loader2 className="size-4 animate-spin" /> : done ? <CheckCircle2 className="size-4" /> : <Icon className="size-4" />}
                    </div>
                    <div className={cx("flex-1 rounded-xl border px-4 py-3 transition-all duration-300", active ? "border-ocean bg-sky-2" : done ? "border-sage/30 bg-white" : "border-line bg-cream/40")}>
                      <div className="flex flex-wrap items-center justify-between gap-2">
                        <div className="flex items-center gap-2">
                          <span className="text-[13.5px] font-semibold text-ink">{n.title}</span>
                          {n.kind === "human" && <Badge tone="ocean">Human approval</Badge>}
                          {n.kind === "branch" && <Badge tone="gold">Condition</Badge>}
                        </div>
                        <span className="font-mono text-[11px] text-faint">{n.system}</span>
                      </div>
                      {done ? <div className="animate-fade-up mt-1 font-mono text-[11.5px] text-sage">✓ {n.log}</div> : <div className="mt-1 flex items-center gap-1 text-[11.5px] text-faint"><CircleDashed className="size-3" /> waiting</div>}
                    </div>
                  </li>
                );
              })}
            </ol>
            {progress === r.nodes.length && !running && (
              <div className="animate-fade-up mt-5 rounded-xl bg-sage-soft p-4 text-[13px] text-sage">
                <b>Run completed</b> in {(totalMs / 1000).toFixed(1)} s with {r.nodes.filter((n) => n.kind === "human").length} human touch{r.nodes.filter((n) => n.kind === "human").length === 1 ? "" : "es"}. Logged with inputs, outputs and timings for audit and ROI reporting.
              </div>
            )}
          </div>
        </Card>
      </div>
    </div>
  );
}
