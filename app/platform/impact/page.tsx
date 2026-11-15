"use client";

import { useMemo, useState } from "react";
import { Calculator, CheckCircle2, Info, RotateCcw, Target } from "lucide-react";
import { ToolHeader } from "@/components/platform/shell";
import { Card, CardHeader, cx, money } from "@/components/ui";

const DEFAULTS = {
  providers: 4,
  adminStaff: 12,
  visitsWeek: 620,
  authsWeek: 70,
  minPerAuth: 45,
  docsWeek: 520,
  callsWeek: 1200,
  claimsMonth: 2600,
  denialRate: 11,
  avgClaim: 240,
  noShowRate: 13,
  revenuePerVisit: 185,
  staffHourly: 24,
  docMinPerVisit: 10,
  authReduction: 70,
  intakeAutomation: 75,
  callDeflection: 30,
  denialReduction: 40,
  noShowReduction: 35,
  scribeReduction: 60,
};
type K = keyof typeof DEFAULTS;

const OPS: { k: K; label: string; min: number; max: number; step?: number; unit?: string }[] = [
  { k: "providers", label: "Providers", min: 1, max: 20 },
  { k: "adminStaff", label: "Admin / front-desk staff", min: 2, max: 60 },
  { k: "visitsWeek", label: "Patient visits per week", min: 100, max: 3000, step: 10 },
  { k: "authsWeek", label: "Prior auths per week", min: 5, max: 300 },
  { k: "minPerAuth", label: "Minutes per manual auth", min: 10, max: 90, unit: " min" },
  { k: "docsWeek", label: "Faxes / documents per week", min: 50, max: 2000, step: 10 },
  { k: "callsWeek", label: "Calls & messages per week", min: 100, max: 4000, step: 50 },
  { k: "claimsMonth", label: "Claims per month", min: 300, max: 12000, step: 100 },
  { k: "denialRate", label: "Initial denial rate", min: 2, max: 25, unit: "%" },
  { k: "avgClaim", label: "Average claim value", min: 80, max: 1500, step: 10, unit: "$" },
  { k: "noShowRate", label: "No-show rate", min: 2, max: 30, unit: "%" },
  { k: "revenuePerVisit", label: "Revenue per visit", min: 60, max: 600, step: 5, unit: "$" },
  { k: "staffHourly", label: "Loaded staff cost / hour", min: 15, max: 60, unit: "$" },
  { k: "docMinPerVisit", label: "Provider documentation / visit", min: 4, max: 30, unit: " min" },
];
const LEVERS: { k: K; label: string; tool: string }[] = [
  { k: "authReduction", label: "Auth staff time reduced", tool: "Prior Auth Copilot" },
  { k: "intakeAutomation", label: "Document intake automated", tool: "Document AI" },
  { k: "callDeflection", label: "Calls & messages resolved by AI", tool: "Agent + Messages" },
  { k: "denialReduction", label: "Denials prevented", tool: "Claim Scrubber" },
  { k: "noShowReduction", label: "No-shows prevented", tool: "Smart Scheduling" },
  { k: "scribeReduction", label: "Documentation time saved", tool: "Clinical Scribe" },
];

export default function ImpactPage() {
  const [v, setV] = useState(DEFAULTS);
  const set = (k: K, n: number) => setV((s) => ({ ...s, [k]: n }));

  const r = useMemo(() => {
    const pct = (k: K) => v[k] / 100;
    const authH = (v.authsWeek * v.minPerAuth * pct("authReduction")) / 60;
    const intakeH = (v.docsWeek * 6 * pct("intakeAutomation")) / 60;
    const callsH = (v.callsWeek * 4 * pct("callDeflection")) / 60;
    const claimsWeek = v.claimsMonth / 4.33;
    const deniedWeek = claimsWeek * pct("denialRate");
    const claimsH = (deniedWeek * pct("denialReduction") * 30 + claimsWeek * 0.5) / 60;
    const referralH = (v.visitsWeek * 0.12 * 8 * 0.55) / 60;
    const staffH = authH + intakeH + callsH + claimsH + referralH;
    const providerH = (v.visitsWeek * v.docMinPerVisit * pct("scribeReduction")) / 60;
    const fte = staffH / 38;
    const laborYear = staffH * v.staffHourly * 50;
    const recoveredVisitsWeek = v.visitsWeek * pct("noShowRate") * pct("noShowReduction");
    const noShowRevenue = recoveredVisitsWeek * v.revenuePerVisit * 50;
    const denialRevenue = v.claimsMonth * 12 * pct("denialRate") * pct("denialReduction") * v.avgClaim * 0.35;
    const authDays = Math.max(1.5, 10 * (1 - pct("authReduction") * 1.06));
    const totalStaffWeek = v.adminStaff * 38;
    const adminReduction = Math.min(0.95, staffH / Math.max(1, totalStaffWeek * 0.7));
    const automationShare = Math.min(0.95, 0.08 + (pct("intakeAutomation") + pct("callDeflection") + pct("authReduction") + pct("denialReduction")) / 4 * 0.85);
    return {
      breakdown: [
        { label: "Prior authorizations", h: authH },
        { label: "Document intake", h: intakeH },
        { label: "Calls & messages", h: callsH },
        { label: "Claims & denials", h: claimsH },
        { label: "Referral coordination", h: referralH },
      ],
      staffH, providerH, fte, laborYear, noShowRevenue, denialRevenue, recoveredVisitsWeek, authDays, adminReduction, automationShare,
      total: laborYear + noShowRevenue + denialRevenue,
    };
  }, [v]);

  const maxH = Math.max(...r.breakdown.map((b) => b.h));

  return (
    <div className="mx-auto max-w-7xl">
      <ToolHeader
        eyebrow="Overview · What success looks like"
        title="Impact & ROI Calculator"
        description="Enter the practice's own volumes and see the time and money each tool returns. Every lever links to the outcome it moves, so success is measured from day one instead of assumed."
        tags={["transparent formulas", "editable assumptions", "baseline → target", "per-workflow attribution"]}
      />

      <div className="grid gap-6 xl:grid-cols-[380px_1fr]">
        <div className="space-y-6">
          <Card>
            <CardHeader title="Practice volumes" icon={<Calculator className="size-4" />} right={<button onClick={() => setV(DEFAULTS)} className="inline-flex items-center gap-1 text-[12px] font-semibold text-muted hover:text-ink"><RotateCcw className="size-3" /> Reset</button>} />
            <div className="space-y-4 p-5">
              {OPS.map((o) => <Slider key={o.k} label={o.label} value={v[o.k]} min={o.min} max={o.max} step={o.step} unit={o.unit} onChange={(n) => set(o.k, n)} />)}
            </div>
          </Card>
        </div>

        <div className="space-y-6">
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <Big label="Staff hours returned / week" value={`${Math.round(r.staffH)} h`} sub={`≈ ${r.fte.toFixed(1)} FTE of capacity`} />
            <Big label="Provider hours returned / week" value={`${Math.round(r.providerH)} h`} sub="less after-hours charting" />
            <Big label="Visits recovered / week" value={`+${Math.round(r.recoveredVisitsWeek)}`} sub="from prevented no-shows" />
            <Big label="Estimated annual impact" value={money(r.total)} sub="labor + recovered revenue" accent />
          </div>

          <Card>
            <CardHeader title="Automation levers" subtitle="Conservative defaults; adjust to match your pilot results" />
            <div className="grid gap-x-8 gap-y-4 p-5 sm:grid-cols-2">
              {LEVERS.map((l) => <Slider key={l.k} label={l.label} hint={l.tool} value={v[l.k]} min={0} max={90} unit="%" onChange={(n) => set(l.k, n)} />)}
            </div>
          </Card>

          <div className="grid gap-6 lg:grid-cols-[1fr_1fr]">
            <Card>
              <CardHeader title="Where the staff hours come from" subtitle="Hours returned per week, by workflow" />
              <div className="space-y-3 p-5">
                {r.breakdown.map((b) => (
                  <div key={b.label} className="grid grid-cols-[140px_1fr_48px] items-center gap-3 text-[12.5px]">
                    <span className="text-ink">{b.label}</span>
                    <div className="h-2.5 rounded-r-[4px] bg-sand"><div className="h-2.5 rounded-r-[4px] bg-[#1F5C8F] transition-all duration-500" style={{ width: `${(b.h / maxH) * 100}%` }} /></div>
                    <span className="text-right font-semibold tabular-nums text-ink">{b.h.toFixed(0)} h</span>
                  </div>
                ))}
              </div>
              <div className="grid grid-cols-3 gap-px border-t border-line bg-line text-center">
                <Mini label="Labor value / yr" value={money(r.laborYear)} />
                <Mini label="No-show revenue / yr" value={money(r.noShowRevenue)} />
                <Mini label="Denial revenue / yr" value={money(r.denialRevenue)} />
              </div>
            </Card>

            <Card>
              <CardHeader title="Mapped to “What success looks like”" icon={<Target className="size-4" />} />
              <ul className="divide-y divide-line">
                {[
                  ["Reduced administrative workload for staff and providers", `−${Math.round(r.adminReduction * 100)}% admin hours · ${Math.round(r.providerH)} provider h/wk returned`],
                  ["Faster processing of authorizations and documentation", `Auth turnaround 10 → ${r.authDays.toFixed(1)} business days · notes closed same day`],
                  ["Improved interoperability between systems", "EMR, payers, imaging, fax, SMS, PT connected via HL7 / FHIR / X12"],
                  ["Increased operational efficiency through automation", `${Math.round(r.automationShare * 100)}% of repeatable tasks automated`],
                  ["AI-driven tools that measurably improve workflows", "Every tool logs time saved & outcome KPIs, reviewed monthly"],
                ].map(([g, k]) => (
                  <li key={g} className="flex gap-3 px-5 py-3.5">
                    <CheckCircle2 className="mt-0.5 size-4 shrink-0 text-sage" />
                    <div><div className="text-[13px] font-semibold text-ink">{g}</div><div className="text-[12.5px] text-muted">{k}</div></div>
                  </li>
                ))}
              </ul>
            </Card>
          </div>

          <div className="flex items-start gap-2 rounded-xl bg-sand p-4 text-[12px] text-muted">
            <Info className="mt-0.5 size-4 shrink-0" />
            <span>Assumptions: 6 min of manual handling per document, 4 min per call/message, 30 min of rework per denial, 35% of prevented denials representing revenue that would otherwise be lost, 12% of visits starting from referrals (8 min each), 38-hour FTE, 50 working weeks. All inputs are editable; replace them with baseline measurements from a 2-week time study.</span>
          </div>
        </div>
      </div>
    </div>
  );
}

function Slider({ label, value, min, max, step = 1, unit = "", hint, onChange }: { label: string; value: number; min: number; max: number; step?: number; unit?: string; hint?: string; onChange: (n: number) => void }) {
  const display = unit === "$" ? `$${value.toLocaleString()}` : `${value.toLocaleString()}${unit}`;
  return (
    <label className="block">
      <div className="flex items-baseline justify-between gap-2 text-[12px]">
        <span className="font-semibold text-muted">{label}{hint && <span className="ml-1.5 font-normal text-faint">· {hint}</span>}</span>
        <span className="font-mono font-semibold text-ink">{display}</span>
      </div>
      <input type="range" min={min} max={max} step={step} value={value} onChange={(e) => onChange(Number(e.target.value))} className="mt-1.5 w-full accent-[#1f5c8f]" />
    </label>
  );
}

function Big({ label, value, sub, accent }: { label: string; value: string; sub: string; accent?: boolean }) {
  return (
    <Card className={cx("p-4", accent && "border-sage/30 bg-gradient-to-br from-sage-soft to-white")}>
      <div className="text-[11.5px] font-medium uppercase tracking-wide text-faint">{label}</div>
      <div className={cx("mt-1.5 font-display text-[26px] leading-none font-semibold tabular-nums", accent ? "text-sage" : "text-ink")}>{value}</div>
      <div className="mt-1.5 text-[12px] text-muted">{sub}</div>
    </Card>
  );
}

function Mini({ label, value }: { label: string; value: string }) {
  return (
    <div className="bg-white px-3 py-3">
      <div className="text-[13px] font-semibold tabular-nums text-ink">{value}</div>
      <div className="text-[10.5px] text-muted">{label}</div>
    </div>
  );
}
