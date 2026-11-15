"use client";

import { useMemo, useState } from "react";
import { CalendarClock, CheckCircle2, FlaskConical, MessageSquareText, Phone, Send, Users } from "lucide-react";
import { ToolHeader } from "@/components/platform/shell";
import { Badge, Button, Card, CardHeader, cx } from "@/components/ui";
import { MODEL, intervention, riskBand, scoreNoShow, type NoShowFeatures } from "@/lib/noshow";

interface Appt { time: string; patient: string; provider: string; location: string; visit: string; payer: string; f: NoShowFeatures }

const APPTS: Appt[] = [
  { time: "8:15", patient: "Maria Gonzalez", provider: "Dr. Okafor", location: "Westbury", visit: "Follow-up", payer: "Medicaid", f: { lead_time_days: 34, prior_no_shows: 2, new_patient: false, reminder_confirmed: false, medicaid: true, no_fault_or_wc: false, distance_miles: 9, monday_or_friday: false, early_morning: true } },
  { time: "8:45", patient: "Harold Becker", provider: "D. Park, PA-C", location: "Lindenhurst", visit: "Follow-up", payer: "Medicare", f: { lead_time_days: 14, prior_no_shows: 0, new_patient: false, reminder_confirmed: true, medicaid: false, no_fault_or_wc: false, distance_miles: 4, monday_or_friday: false, early_morning: true } },
  { time: "9:30", patient: "Priya Natarajan", provider: "Dr. Mateo", location: "Lindenhurst", visit: "New consult", payer: "Cigna", f: { lead_time_days: 21, prior_no_shows: 0, new_patient: true, reminder_confirmed: false, medicaid: false, no_fault_or_wc: false, distance_miles: 18, monday_or_friday: false, early_morning: false } },
  { time: "10:00", patient: "Aisha Rahman", provider: "Dr. Ruiz", location: "Elmhurst", visit: "Follow-up", payer: "No-Fault", f: { lead_time_days: 7, prior_no_shows: 1, new_patient: false, reminder_confirmed: true, medicaid: false, no_fault_or_wc: true, distance_miles: 2, monday_or_friday: false, early_morning: false } },
  { time: "10:45", patient: "Carmen Diaz", provider: "D. Park, PA-C", location: "Westbury", visit: "New consult", payer: "Empire BCBS", f: { lead_time_days: 4, prior_no_shows: 0, new_patient: true, reminder_confirmed: false, medicaid: false, no_fault_or_wc: false, distance_miles: 3, monday_or_friday: false, early_morning: false } },
  { time: "11:30", patient: "Kevin Murphy", provider: "D. Park, PA-C", location: "Westbury", visit: "Follow-up", payer: "Oxford", f: { lead_time_days: 48, prior_no_shows: 3, new_patient: false, reminder_confirmed: false, medicaid: false, no_fault_or_wc: false, distance_miles: 22, monday_or_friday: false, early_morning: false } },
  { time: "1:00", patient: "Robert Williams", provider: "Dr. Okafor", location: "Westbury", visit: "Procedure (ESI)", payer: "Workers' Comp", f: { lead_time_days: 10, prior_no_shows: 0, new_patient: false, reminder_confirmed: true, medicaid: false, no_fault_or_wc: true, distance_miles: 16, monday_or_friday: false, early_morning: false } },
  { time: "2:15", patient: "Thomas Russo", provider: "Dr. Mateo", location: "East Meadow", visit: "Pre-op", payer: "UnitedHealthcare", f: { lead_time_days: 12, prior_no_shows: 0, new_patient: false, reminder_confirmed: true, medicaid: false, no_fault_or_wc: false, distance_miles: 3, monday_or_friday: false, early_morning: false } },
  { time: "3:00", patient: "James O'Connor", provider: "Dr. Okafor", location: "Westbury", visit: "Follow-up", payer: "Aetna", f: { lead_time_days: 26, prior_no_shows: 1, new_patient: false, reminder_confirmed: false, medicaid: false, no_fault_or_wc: false, distance_miles: 8, monday_or_friday: false, early_morning: false } },
  { time: "3:45", patient: "Linda Chen", provider: "Dr. Mateo", location: "Elmhurst", visit: "Follow-up", payer: "Humana MA", f: { lead_time_days: 30, prior_no_shows: 0, new_patient: false, reminder_confirmed: false, medicaid: false, no_fault_or_wc: false, distance_miles: 6, monday_or_friday: false, early_morning: false } },
];

const OPEN_SLOTS = [
  { slot: "Thu 2:00 PM · Westbury · Dr. Okafor", match: "Ana Pereira", why: "Waitlisted 9 d · same visit type · 3.1 mi · auth approved" },
  { slot: "Thu 2:30 PM · Westbury · Dr. Okafor", match: "Luis Moreno", why: "Asked for earlier date · ES speaker · Medicare" },
  { slot: "Thu 3:15 PM · Westbury · D. Park, PA-C", match: "Grace Kim", why: "New patient referral 2 d old · 1.8 mi" },
  { slot: "Thu 3:45 PM · Westbury · D. Park, PA-C", match: "Daniel Shah", why: "Post-op check pulled forward · confirmed by SMS" },
];

const BOOL_FEATURES: { key: keyof NoShowFeatures; label: string }[] = [
  { key: "reminder_confirmed", label: "Confirmed reminder" },
  { key: "new_patient", label: "New patient" },
  { key: "medicaid", label: "Medicaid" },
  { key: "no_fault_or_wc", label: "No-fault / WC" },
  { key: "monday_or_friday", label: "Mon / Fri" },
  { key: "early_morning", label: "Before 9 AM" },
];

export default function SchedulingPage() {
  const scored = useMemo(() => APPTS.map((a) => ({ a, ...scoreNoShow(a.f) })), []);
  const [sel, setSel] = useState(0);
  const [sim, setSim] = useState<NoShowFeatures>(APPTS[5].f);
  const [offers, setOffers] = useState<Record<number, "idle" | "sent" | "accepted">>({});
  const [acted, setActed] = useState<number[]>([]);
  const cur = scored[sel];
  const simScore = scoreNoShow(sim);
  const expected = scored.reduce((s, x) => s + x.probability, 0);

  const sendOffers = () => {
    OPEN_SLOTS.forEach((_, i) => {
      setOffers((o) => ({ ...o, [i]: "sent" }));
      setTimeout(() => setOffers((o) => ({ ...o, [i]: i === 2 ? "sent" : "accepted" })), 900 + i * 500);
    });
  };

  return (
    <div className="mx-auto max-w-7xl">
      <ToolHeader
        eyebrow="Front office · Scheduling & predictive analytics"
        title="Smart Scheduling"
        description="A logistic-regression no-show model (trained in Python, scored live here) ranks every appointment, explains why, and recommends the cheapest intervention that works. Open slots are backfilled from the waitlist automatically."
        tags={[`model: ${MODEL.model}`, `AUC ${MODEL.test_auc}`, "explainable features", "waitlist matching", "two-way SMS"]}
        impact={[{ value: "13.4→8.6%", label: "no-show rate" }, { value: "+38", label: "visits recovered / wk" }]}
      />

      <div className="grid gap-6 xl:grid-cols-[1.25fr_1fr]">
        <Card className="overflow-hidden">
          <CardHeader title="Tuesday, Nov 17 · all locations" subtitle={`${APPTS.length} appointments · expected no-shows: ${expected.toFixed(1)}`} icon={<CalendarClock className="size-4" />} />
          <ul className="divide-y divide-line">
            {scored.map(({ a, probability }, i) => {
              const band = riskBand(probability);
              return (
                <li key={a.time + a.patient}>
                  <button onClick={() => setSel(i)} className={cx("grid w-full grid-cols-[52px_1fr_auto] items-center gap-3 px-5 py-3 text-left transition sm:grid-cols-[52px_1fr_140px_auto]", sel === i ? "bg-sky-2" : "hover:bg-cream")}>
                    <span className="font-mono text-[12px] text-muted">{a.time}</span>
                    <span className="min-w-0">
                      <span className="block truncate text-[13.5px] font-semibold text-ink">{a.patient} {acted.includes(i) && <CheckCircle2 className="ml-1 inline size-3.5 text-sage" />}</span>
                      <span className="block truncate text-[12px] text-muted">{a.visit} · {a.provider} · {a.location}</span>
                    </span>
                    <span className="hidden items-center gap-2 sm:flex">
                      <span className="h-1.5 flex-1 rounded-full bg-sand"><span className={cx("block h-1.5 rounded-full", band.tone === "danger" ? "bg-danger" : band.tone === "warn" ? "bg-warn" : "bg-sage")} style={{ width: `${Math.min(100, probability * 200)}%` }} /></span>
                      <span className="w-9 text-right text-[12px] font-semibold tabular-nums text-ink">{Math.round(probability * 100)}%</span>
                    </span>
                    <Badge tone={band.tone === "danger" ? "danger" : band.tone === "warn" ? "warn" : "sage"}>{band.label}</Badge>
                  </button>
                </li>
              );
            })}
          </ul>
        </Card>

        <div className="space-y-6">
          <Card>
            <CardHeader title={`${cur.a.patient} · ${Math.round(cur.probability * 100)}% no-show risk`} subtitle={`${cur.a.time} · ${cur.a.visit} · ${cur.a.payer}`} />
            <div className="p-5">
              <div className="text-[12px] font-semibold text-muted">Why: contribution to log-odds</div>
              <div className="mt-3 space-y-2">
                {cur.contributions.filter((c) => Math.abs(c.value) > 0.01).slice(0, 6).map((c) => {
                  const w = Math.min(100, Math.abs(c.value) * 60);
                  return (
                    <div key={c.feature} className="grid grid-cols-[150px_1fr] items-center gap-3 text-[12px]">
                      <span className="truncate text-ink">{c.label}</span>
                      <div className="relative h-4">
                        <div className="absolute inset-y-0 left-1/2 w-px bg-line-2" />
                        <div className={cx("absolute inset-y-[3px] rounded-[3px]", c.value > 0 ? "left-1/2 bg-danger/80" : "right-1/2 bg-sage")} style={{ width: `${w / 2}%` }} />
                      </div>
                    </div>
                  );
                })}
                <div className="flex justify-between text-[10px] text-faint"><span>← lowers risk</span><span>raises risk →</span></div>
              </div>
              <div className="mt-4 rounded-xl bg-sand p-3 text-[13px] text-ink"><b>Recommended:</b> {intervention(cur.probability, cur.a.f)}</div>
              <div className="mt-3 flex gap-2">
                <Button size="sm" onClick={() => setActed((x) => [...x, sel])} disabled={acted.includes(sel)}><MessageSquareText className="size-3.5" /> {acted.includes(sel) ? "Sent" : "Send two-way SMS"}</Button>
                <Button size="sm" variant="secondary" onClick={() => setActed((x) => [...x, sel])} disabled={acted.includes(sel)}><Phone className="size-3.5" /> Add to call list</Button>
              </div>
            </div>
          </Card>

          <Card>
            <CardHeader title="What-if simulator" subtitle="Change the inputs and the model re-scores instantly" icon={<FlaskConical className="size-4" />} right={<div className="text-right"><div className={cx("font-display text-2xl font-semibold tabular-nums", simScore.probability >= 0.3 ? "text-danger" : simScore.probability >= 0.15 ? "text-warn" : "text-sage")}>{Math.round(simScore.probability * 100)}%</div><div className="text-[11px] text-muted">risk</div></div>} />
            <div className="space-y-4 p-5">
              {([["lead_time_days", "Lead time", 0, 90, "d"], ["prior_no_shows", "Prior no-shows", 0, 6, ""], ["distance_miles", "Distance", 0, 45, " mi"]] as const).map(([k, label, min, max, unit]) => (
                <label key={k} className="block">
                  <div className="flex justify-between text-[12px]"><span className="font-semibold text-muted">{label}</span><span className="font-mono text-ink">{sim[k] as number}{unit}</span></div>
                  <input type="range" min={min} max={max} value={sim[k] as number} onChange={(e) => setSim((s) => ({ ...s, [k]: Number(e.target.value) }))} className="mt-1.5 w-full accent-[#1f5c8f]" />
                </label>
              ))}
              <div className="flex flex-wrap gap-2">
                {BOOL_FEATURES.map((b) => (
                  <button key={b.key} onClick={() => setSim((s) => ({ ...s, [b.key]: !s[b.key] }))} className={cx("rounded-lg px-2.5 py-1 text-[12px] font-semibold ring-1 ring-inset transition", sim[b.key] ? "bg-navy text-white ring-navy" : "bg-white text-muted ring-line-2")}>{b.label}</button>
                ))}
              </div>
            </div>
          </Card>
        </div>
      </div>

      <div className="mt-6 grid gap-6 lg:grid-cols-[1.25fr_1fr]">
        <Card>
          <CardHeader title="Waitlist backfill · Thursday" subtitle="Open slots matched to waitlisted patients by visit type, provider, insurance, language and distance" icon={<Users className="size-4" />} right={<Button size="sm" variant="sage" onClick={sendOffers} disabled={Object.keys(offers).length > 0}><Send className="size-3.5" /> Text offers</Button>} />
          <ul className="divide-y divide-line">
            {OPEN_SLOTS.map((o, i) => (
              <li key={o.slot} className="flex flex-wrap items-center justify-between gap-3 px-5 py-3">
                <div>
                  <div className="text-[13px] font-semibold text-ink">{o.slot}</div>
                  <div className="text-[12px] text-muted"><b className="text-ink">{o.match}</b> · {o.why}</div>
                </div>
                {offers[i] === "accepted" ? <Badge tone="sage"><CheckCircle2 className="size-3" /> Booked</Badge> : offers[i] === "sent" ? <Badge tone="ocean">Offer sent · waiting</Badge> : <Badge>Open</Badge>}
              </li>
            ))}
          </ul>
        </Card>
        <Card>
          <CardHeader title="Model card" subtitle="Transparent by design" />
          <dl className="grid grid-cols-2 gap-x-4 gap-y-3 p-5 text-[12.5px]">
            <dt className="text-muted">Algorithm</dt><dd className="text-ink">Logistic regression (L2)</dd>
            <dt className="text-muted">Training data</dt><dd className="text-ink">{MODEL.trained_on}</dd>
            <dt className="text-muted">Test AUC</dt><dd className="font-semibold text-ink">{MODEL.test_auc}</dd>
            <dt className="text-muted">Base rate</dt><dd className="text-ink">{(MODEL.base_no_show_rate * 100).toFixed(1)}%</dd>
            <dt className="text-muted">Source</dt><dd className="font-mono text-[11.5px] text-ink">ml/no_show_model.py</dd>
            <dt className="text-muted">Production plan</dt><dd className="text-ink">Retrain monthly on de-identified EMR history; monitor drift and fairness by payer and language</dd>
          </dl>
        </Card>
      </div>
    </div>
  );
}
