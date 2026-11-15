"use client";

import { useState } from "react";
import { AlarmClock, Briefcase, Car, CheckCircle2, FileSignature, Scale } from "lucide-react";
import { ToolHeader } from "@/components/platform/shell";
import { Badge, Button, Card, CardHeader, cx } from "@/components/ui";

interface Task { label: string; due: string; daysLeft: number | null; status: "done" | "open" | "late" | "draft"; note: string }
interface Case { id: string; kind: "wc" | "nf"; patient: string; carrier: string; claim: string; event: string; attorney?: string; tasks: Task[] }

const CASES: Case[] = [
  {
    id: "WC-118", kind: "wc", patient: "Jorge Castillo", carrier: "State Insurance Fund (NYSIF)", claim: "WCB # pending", event: "Injured 11/13/2026 · lifting at warehouse · first visit 11/15",
    tasks: [
      { label: "Doctor's initial report (C-4)", due: "11/17/2026", daysLeft: 1, status: "draft", note: "Must be filed within 48 hours of first treatment. Draft generated from the 11/15 note; needs provider signature." },
      { label: "Causal relationship statement", due: "with C-4", daysLeft: 1, status: "draft", note: "Copilot extracted mechanism of injury and the provider's causation opinion." },
      { label: "Treatment plan vs. Medical Treatment Guidelines", due: "before treatment", daysLeft: null, status: "done", note: "PT 2–3×/wk and NSAIDs fall within the Mid & Low Back MTG; no variance PAR needed." },
    ],
  },
  {
    id: "WC-097", kind: "wc", patient: "Robert Williams", carrier: "NYSIF", claim: "WCB G331 8042", event: "Injured 09/14/2026 · neck · lifting pallet", attorney: "Represented: Klein & Partners",
    tasks: [
      { label: "C-4 initial report", due: "09/16/2026", daysLeft: null, status: "done", note: "Filed within 48 hours." },
      { label: "Progress report", due: "11/20/2026", daysLeft: 4, status: "open", note: "Pre-filled from the last 3 visits; pain scores and work status trended automatically." },
      { label: "Cervical ESI (62321)", due: "11/17/2026", daysLeft: 1, status: "done", note: "Within MTG; scheduled. Bill will carry the WCB case number." },
      { label: "Carrier IME", due: "12/03/2026", daysLeft: 17, status: "open", note: "Records packet (minimum necessary) prepared for the IME physician." },
    ],
  },
  {
    id: "NF-221", kind: "nf", patient: "Aisha Rahman", carrier: "GEICO (PIP)", claim: "NF-77120934", event: "Motor vehicle accident 09/20/2026 · cervical strain",
    tasks: [
      { label: "NF-2 application (patient)", due: "10/20/2026", daysLeft: null, status: "done", note: "Received 10/02, within 30 days of the accident." },
      { label: "Bill for DOS 09/22 (NF-3)", due: "11/06/2026", daysLeft: -10, status: "late", note: "Outside the 45-day window: hospital records arrived late. Justification letter drafted and attached." },
      { label: "Carrier verification request", due: "11/24/2026", daysLeft: 8, status: "open", note: "Requested: X-ray report & office notes. Packet assembled from the chart; ready to send." },
    ],
  },
  {
    id: "NF-230", kind: "nf", patient: "Daniel Ortiz", carrier: "Progressive (PIP)", claim: "NF-88412077", event: "Motor vehicle accident 11/02/2026 · lumbar strain",
    tasks: [
      { label: "NF-2 application (patient)", due: "12/02/2026", daysLeft: 16, status: "open", note: "Bilingual SMS reminder sent with a pre-filled form link." },
      { label: "Bill for DOS 11/09 (NF-3)", due: "12/24/2026", daysLeft: 38, status: "open", note: "Will auto-generate once the visit note is signed." },
    ],
  },
];

function DueChip({ t }: { t: Task }) {
  if (t.status === "done") return <Badge tone="sage"><CheckCircle2 className="size-3" /> Done</Badge>;
  if (t.status === "late") return <Badge tone="danger">{Math.abs(t.daysLeft ?? 0)} d late</Badge>;
  if (t.daysLeft === null) return <Badge>Open</Badge>;
  return <Badge tone={t.daysLeft <= 2 ? "danger" : t.daysLeft <= 7 ? "warn" : "ocean"}><AlarmClock className="size-3" /> {t.daysLeft} d left</Badge>;
}

export default function WcNoFaultPage() {
  const [filter, setFilter] = useState<"all" | "wc" | "nf">("all");
  const [signed, setSigned] = useState<string[]>([]);
  const list = CASES.filter((c) => filter === "all" || c.kind === filter);
  const urgent = CASES.flatMap((c) => c.tasks.filter((t) => t.status !== "done" && (t.status === "late" || (t.daysLeft ?? 99) <= 2))).length;

  return (
    <div className="mx-auto max-w-7xl">
      <ToolHeader
        eyebrow="Revenue cycle · Workers' comp & no-fault"
        title="WC & No-Fault Desk"
        description="Workers' compensation and no-fault cases carry strict New York filing deadlines. This desk tracks every clock, drafts the forms from the chart, and escalates before a deadline is missed, which is often the difference between getting paid and writing it off."
        tags={["NY WCB forms", "Medical Treatment Guidelines", "No-fault NF-2 / NF-3", "deadline engine", "minimum-necessary records"]}
        impact={[{ value: `${urgent}`, label: "deadlines ≤ 48 h or late" }, { value: "0", label: "missed C-4s since go-live" }]}
      />

      <div className="mb-5 flex flex-wrap items-center gap-2">
        {([["all", "All cases"], ["wc", "Workers' comp"], ["nf", "No-fault"]] as const).map(([k, l]) => (
          <button key={k} onClick={() => setFilter(k)} className={cx("rounded-xl px-3 py-1.5 text-[13px] font-semibold ring-1 ring-inset transition", filter === k ? "bg-navy text-white ring-navy" : "bg-white text-muted ring-line-2 hover:text-ink")}>{l}</button>
        ))}
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        {list.map((c) => (
          <Card key={c.id} className="animate-fade-up">
            <CardHeader
              title={<span className="flex items-center gap-2">{c.patient} <span className="font-mono text-[11px] font-normal text-faint">{c.id}</span></span>}
              subtitle={`${c.carrier} · ${c.claim}`}
              icon={c.kind === "wc" ? <Briefcase className="size-4" /> : <Car className="size-4" />}
              right={<Badge tone={c.kind === "wc" ? "gold" : "ocean"}>{c.kind === "wc" ? "Workers' comp" : "No-fault"}</Badge>}
            />
            <div className="border-b border-line px-5 py-2.5 text-[12.5px] text-muted">{c.event}{c.attorney && <> · <Scale className="mb-0.5 inline size-3" /> {c.attorney}</>}</div>
            <ul className="divide-y divide-line">
              {c.tasks.map((t) => {
                const isSigned = signed.includes(c.id + t.label);
                return (
                  <li key={t.label} className="px-5 py-3.5">
                    <div className="flex items-start justify-between gap-3">
                      <div className="min-w-0">
                        <div className="text-[13px] font-semibold text-ink">{t.label}</div>
                        <div className="text-[11.5px] text-faint">Due {t.due}</div>
                      </div>
                      {isSigned ? <Badge tone="sage"><CheckCircle2 className="size-3" /> Filed</Badge> : <DueChip t={t} />}
                    </div>
                    <p className="mt-1.5 text-[12.5px] text-muted">{t.note}</p>
                    {t.status === "draft" && !isSigned && (
                      <Button size="sm" className="mt-2" onClick={() => setSigned((s) => [...s, c.id + t.label])}><FileSignature className="size-3.5" /> Review, sign &amp; file</Button>
                    )}
                  </li>
                );
              })}
            </ul>
          </Card>
        ))}
      </div>

      <Card className="mt-6">
        <CardHeader title="Deadline rules encoded" subtitle="Configurable per carrier; always confirm current Board and DFS requirements" />
        <div className="grid gap-px bg-line sm:grid-cols-2 lg:grid-cols-4">
          {[
            ["C-4 initial report", "Filed within 48 hours of first treatment of a work injury."],
            ["MTG compliance", "Care within the Medical Treatment Guidelines is pre-authorized; a variance goes through the WCB Medical Portal (PAR)."],
            ["NF-2 application", "Patient files with the no-fault carrier within 30 days of the accident."],
            ["NF-3 billing", "Provider bills within 45 days of service, or attaches a reasonable justification for the delay."],
          ].map(([t, d]) => (
            <div key={t} className="bg-white p-4"><div className="text-[13px] font-semibold text-ink">{t}</div><div className="mt-1 text-[12px] text-muted">{d}</div></div>
          ))}
        </div>
      </Card>
    </div>
  );
}
