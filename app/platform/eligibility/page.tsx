"use client";

import { useState } from "react";
import { AlertTriangle, BadgeCheck, CheckCircle2, Loader2, Play, ShieldQuestion, XCircle } from "lucide-react";
import { ToolHeader } from "@/components/platform/shell";
import { Badge, Button, Card, CardHeader, CodeBlock, Stat, cx } from "@/components/ui";

type Result = "active" | "action" | "inactive";
interface Row {
  time: string; patient: string; payer: string; memberId: string; visit: string;
  result: Result; copay: string; deductible: string; flags: string[]; fix?: string;
}

const ROWS: Row[] = [
  { time: "8:15", patient: "Maria Gonzalez", payer: "Fidelis Care (Medicaid)", memberId: "FC88310442", visit: "Follow-up", result: "active", copay: "$0", deductible: "n/a", flags: ["Established patient: Medicaid accepted"] },
  { time: "8:45", patient: "Harold Becker", payer: "Medicare Part B", memberId: "1EG4-TE5-MK72", visit: "Follow-up", result: "active", copay: "20% coinsurance", deductible: "Met", flags: [] },
  { time: "9:30", patient: "Priya Natarajan", payer: "Cigna Open Access", memberId: "U7730 18842", visit: "New consult", result: "action", copay: "$50 specialist", deductible: "$1,140 remaining", flags: ["Scoliosis X-ray (72082) billed with visit: patient responsibility est. $96"], fix: "Good-faith estimate texted; collect at check-in" },
  { time: "10:00", patient: "Aisha Rahman", payer: "GEICO No-Fault", memberId: "NF-77120934", visit: "Follow-up", result: "active", copay: "$0", deductible: "n/a", flags: ["No-fault claim open · NF-3 deadline tracked"] },
  { time: "10:45", patient: "Carmen Diaz", payer: "Empire BCBS", memberId: "YRP 451207733", visit: "New consult", result: "active", copay: "$40 specialist", deductible: "$310 remaining", flags: [] },
  { time: "11:30", patient: "Kevin Murphy", payer: "Oxford (UHC)", memberId: "OXF 33019276", visit: "Follow-up", result: "inactive", copay: "—", deductible: "—", flags: ["Coverage terminated 10/31/2026 (271 EB*6)", "Possible new employer plan"], fix: "Patient texted for new card photo; slot held 24 h" },
  { time: "1:00", patient: "Robert Williams", payer: "NYSIF (Workers' Comp)", memberId: "WCB G3318042", visit: "Procedure (ESI)", result: "active", copay: "$0", deductible: "n/a", flags: ["Within MTG → pre-authorized", "WCB case # on file"] },
  { time: "2:15", patient: "Thomas Russo", payer: "UnitedHealthcare", memberId: "UHC 90418827", visit: "Pre-op", result: "action", copay: "$40 specialist", deductible: "$0 remaining", flags: ["Surgery auth pending (PA-24813)", "Out-of-pocket max reached in 2026"], fix: "Surgery date held; auth chaser polling hourly" },
  { time: "3:00", patient: "James O'Connor", payer: "Aetna PPO", memberId: "W219-55810", visit: "Follow-up", result: "active", copay: "$35 specialist", deductible: "$0 remaining", flags: [] },
  { time: "3:45", patient: "Linda Chen", payer: "Humana Medicare Advantage", memberId: "H5619-0042", visit: "Follow-up", result: "active", copay: "$25 specialist", deductible: "n/a", flags: ["ESI auth HUM-AUTH-8841207 valid to 02/14/2027"] },
];

const SAMPLE_271 = `ISA*00*          *00*          *ZZ*CLEARINGHOUSE  *ZZ*MATEOORTHO     *261116*0612*^*00501*000004117*0*P*:~
ST*271*0001*005010X279A1~
BHT*0022*11*ELG-0011730*20261116*0612~
NM1*PR*2*OXFORD HEALTH PLANS*****PI*06111~
NM1*IL*1*MURPHY*KEVIN****MI*OXF33019276~
DTP*346*D8*20260101~
DTP*347*D8*20261031~
EB*6**30~                         <- 6 = Inactive · 30 = Health benefit plan coverage
MSG*COVERAGE TERMINATED~
SE*9*0001~`;

export default function EligibilityPage() {
  const [state, setState] = useState<"idle" | "running" | "done">("idle");
  const [count, setCount] = useState(0);
  const [resolved, setResolved] = useState<string[]>([]);

  const run = () => {
    setState("running");
    setCount(0);
    ROWS.forEach((_, i) => setTimeout(() => { setCount(i + 1); if (i === ROWS.length - 1) setState("done"); }, 180 * (i + 1)));
  };

  const done = (i: number) => state === "done" || count > i;
  const actions = ROWS.filter((r) => r.result !== "active").length;

  return (
    <div className="mx-auto max-w-7xl">
      <ToolHeader
        eyebrow="Revenue cycle · Insurance verification"
        title="Eligibility & Benefits"
        description="Every appointment for tomorrow is verified overnight via X12 270/271, so front-desk staff start the day with exceptions only: terminated plans, patient balances to estimate, and authorizations still pending, each with the next step already taken."
        tags={["X12 270 / 271", "batch verification", "good-faith estimates", "auth cross-check", "exceptions-only workflow"]}
        impact={[{ value: "−18 h", label: "phone verification / wk" }, { value: "−61%", label: "eligibility denials" }]}
      />

      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Stat label="Appointments tomorrow" value={ROWS.length} tone="neutral" hint="Tue, Nov 17 · all locations" />
        <Stat label="Verified active" value={state === "idle" ? "—" : ROWS.filter((r, i) => r.result === "active" && done(i)).length} />
        <Stat label="Need attention" value={state === "idle" ? "—" : actions} tone="danger" hint="already actioned automatically" />
        <Stat label="Run time" value={state === "done" ? "1.8 s" : "—"} tone="neutral" hint="vs. ~6 min per patient by phone/portal" />
      </div>

      <Card className="mt-6 overflow-hidden">
        <CardHeader title="Tomorrow's schedule" subtitle={state === "done" ? "Verified at 6:12 AM · next run in 24 h" : "Run the overnight batch on demand"} icon={<BadgeCheck className="size-4" />} right={<Button onClick={run} disabled={state === "running"}>{state === "running" ? <Loader2 className="size-4 animate-spin" /> : <Play className="size-4" />} Verify all</Button>} />
        <div className="overflow-x-auto">
          <table className="w-full min-w-[860px] text-[13px]">
            <thead className="bg-cream text-left text-[11px] uppercase tracking-wide text-faint">
              <tr><th className="px-5 py-2.5">Time</th><th className="px-2 py-2.5">Patient</th><th className="px-2 py-2.5">Payer</th><th className="px-2 py-2.5">Status</th><th className="px-2 py-2.5">Copay</th><th className="px-2 py-2.5">Deductible</th><th className="px-5 py-2.5">Notes & next step</th></tr>
            </thead>
            <tbody className="divide-y divide-line">
              {ROWS.map((r, i) => (
                <tr key={r.patient} className={cx("transition-colors", done(i) && r.result === "inactive" && "bg-danger-soft/40", done(i) && r.result === "action" && "bg-warn-soft/40")}>
                  <td className="px-5 py-3 font-mono text-[12px] text-muted">{r.time}</td>
                  <td className="px-2 py-3"><div className="font-semibold text-ink">{r.patient}</div><div className="text-[11.5px] text-muted">{r.visit}</div></td>
                  <td className="px-2 py-3"><div className="text-ink">{r.payer}</div><div className="font-mono text-[11px] text-faint">{r.memberId}</div></td>
                  <td className="px-2 py-3">
                    {!done(i) ? (state === "running" ? <Loader2 className="size-4 animate-spin text-ocean" /> : <span className="text-faint">—</span>) :
                      r.result === "active" ? <Badge tone="sage"><CheckCircle2 className="size-3" /> Active</Badge> :
                      r.result === "action" ? <Badge tone="warn"><AlertTriangle className="size-3" /> Action</Badge> :
                      <Badge tone="danger"><XCircle className="size-3" /> Inactive</Badge>}
                  </td>
                  <td className="px-2 py-3 text-ink">{done(i) ? r.copay : ""}</td>
                  <td className="px-2 py-3 text-muted">{done(i) ? r.deductible : ""}</td>
                  <td className="px-5 py-3">
                    {done(i) && (
                      <div className="space-y-1">
                        {r.flags.map((f) => <div key={f} className="text-[12px] text-muted">• {f}</div>)}
                        {r.fix && (
                          <div className="flex items-center gap-2">
                            <span className="text-[12px] font-semibold text-ocean">→ {r.fix}</span>
                            {!resolved.includes(r.patient) ? <button onClick={() => setResolved((x) => [...x, r.patient])} className="rounded-md px-1.5 py-0.5 text-[11px] font-semibold text-sage hover:bg-sage-soft">Mark resolved</button> : <Badge tone="sage">Resolved</Badge>}
                          </div>
                        )}
                      </div>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Card>

      <div className="mt-6 grid gap-6 lg:grid-cols-2">
        <Card>
          <CardHeader title="Raw 271 response (inactive coverage)" subtitle="What the clearinghouse returned for Kevin Murphy" icon={<ShieldQuestion className="size-4" />} />
          <div className="p-4"><CodeBlock value={SAMPLE_271} maxH="max-h-72" /></div>
        </Card>
        <Card>
          <CardHeader title="Rules applied after each 271" />
          <ul className="space-y-2.5 p-5 text-[13px] text-muted">
            <li>• <b className="text-ink">Inactive or terminated plan:</b> text the patient for a new card photo, hold the slot 24 h, then release it to the waitlist.</li>
            <li>• <b className="text-ink">Patient balance expected:</b> send a good-faith estimate from the deductible and coinsurance left.</li>
            <li>• <b className="text-ink">Medicaid new patient:</b> flag against practice policy (established patients and hospital follow-ups only).</li>
            <li>• <b className="text-ink">Service needs auth:</b> cross-check Prior Auth Copilot; no valid auth means no procedure slot.</li>
            <li>• <b className="text-ink">Workers&apos; comp / no-fault:</b> confirm case or claim number and carrier instead of a commercial 270.</li>
          </ul>
        </Card>
      </div>
    </div>
  );
}
