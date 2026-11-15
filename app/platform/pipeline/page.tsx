"use client";

import { useMemo, useState } from "react";
import { CheckCircle2, Database, GitMerge, Loader2, Play, Table2, TriangleAlert } from "lucide-react";
import { ToolHeader } from "@/components/platform/shell";
import { Badge, Button, Card, CardHeader, CodeBlock, Segmented, cx, inputCls } from "@/components/ui";

type Cloud = "aws" | "azure" | "gcp";
const STAGES: { key: string; title: string; detail: string; svc: Record<Cloud, string>; count: number }[] = [
  { key: "src", title: "Sources", detail: "EMR (FHIR + HL7), PM/billing, clearinghouse 835/837, fax/Document AI, phone/SMS, PT system", svc: { aws: "9 connectors", azure: "9 connectors", gcp: "9 connectors" }, count: 48210 },
  { key: "ing", title: "Ingest", detail: "MLLP listener, FHIR $export (bulk), SFTP/X12 drops, webhooks → durable queue", svc: { aws: "ECS listener · SQS · S3 (raw, KMS)", azure: "Container Apps · Service Bus · ADLS", gcp: "Cloud Run · Pub/Sub · GCS" }, count: 48210 },
  { key: "norm", title: "Normalize", detail: "Map to FHIR R4, code systems (ICD-10, CPT, LOINC), patient matching, dedupe, data-quality checks", svc: { aws: "Python workers · HealthLake", azure: "Functions · Health Data Services", gcp: "Dataflow · Healthcare API" }, count: 47962 },
  { key: "store", title: "Store", detail: "Operational FHIR store + analytics warehouse (star schema), immutable audit log", svc: { aws: "RDS Postgres · Redshift", azure: "Azure SQL · Synapse", gcp: "Cloud SQL · BigQuery" }, count: 47962 },
  { key: "serve", title: "Serve", detail: "APIs for apps & agent, dashboards, ML features, scheduled reports", svc: { aws: "API Gateway · QuickSight · SageMaker", azure: "APIM · Power BI · Azure ML", gcp: "Apigee · Looker · Vertex AI" }, count: 47962 },
];

const DQ = [
  { check: "Required fields present (PID-3, PID-5, PID-7)", pass: 99.6, issues: 182 },
  { check: "Valid ICD-10-CM codes (no retired M54.5)", pass: 99.1, issues: 41 },
  { check: "Appointment ↔ encounter reconciliation", pass: 98.4, issues: 77 },
  { check: "Duplicate patients resolved by MPI", pass: 99.8, issues: 12 },
  { check: "835 payments matched to 837 claims", pass: 97.9, issues: 64 },
];

const TERMS = [
  ["eCW local", "LBP w/ sciatica R", "ICD-10-CM", "M54.41 · Lumbago with sciatica, right side"],
  ["athena order", "MRI L-SPINE WO", "CPT", "72148 · MRI lumbar spine w/o contrast"],
  ["PT system", "Pain NRS", "LOINC", "72514-3 · Pain severity 0-10 (reported)"],
  ["Intake form", "Smoker? No", "LOINC", "72166-2 · Tobacco smoking status = Never"],
  ["Fax OCR", "TFESI L4-5 RT", "CPT", "64483-RT · Transforaminal ESI, lumbar, single level"],
];

const SQL = `-- Median authorization turnaround by payer, last 90 days
SELECT p.payer_name,
       COUNT(*)                                             AS requests,
       PERCENTILE_CONT(0.5) WITHIN GROUP
         (ORDER BY a.decided_at - a.submitted_at)           AS median_turnaround,
       AVG(CASE WHEN a.decision = 'denied' THEN 1 ELSE 0 END) AS denial_rate
FROM   fact_authorization a
JOIN   dim_payer p ON p.payer_key = a.payer_key
WHERE  a.submitted_at >= CURRENT_DATE - INTERVAL '90 days'
GROUP  BY p.payer_name
ORDER  BY median_turnaround DESC;`;

const SQL_ROWS = [
  ["UnitedHealthcare", 214, "3.4 d", "7.9%"],
  ["Aetna", 163, "3.1 d", "6.1%"],
  ["Empire BCBS", 188, "2.7 d", "5.3%"],
  ["Humana MA", 97, "1.9 d", "4.1%"],
  ["Cigna", 72, "2.2 d", "3.8%"],
];

function norm(s: string) {
  return s.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase().replace(/[^a-z0-9]/g, "");
}
function jaroWinkler(a: string, b: string) {
  a = norm(a); b = norm(b);
  if (!a || !b) return 0;
  if (a === b) return 1;
  const range = Math.max(0, Math.floor(Math.max(a.length, b.length) / 2) - 1);
  const am = new Array(a.length).fill(false), bm = new Array(b.length).fill(false);
  let m = 0;
  for (let i = 0; i < a.length; i++) {
    for (let j = Math.max(0, i - range); j < Math.min(b.length, i + range + 1); j++) {
      if (bm[j] || a[i] !== b[j]) continue;
      am[i] = bm[j] = true; m++; break;
    }
  }
  if (!m) return 0;
  let t = 0, k = 0;
  for (let i = 0; i < a.length; i++) {
    if (!am[i]) continue;
    while (!bm[k]) k++;
    if (a[i] !== b[k]) t++;
    k++;
  }
  const jaro = (m / a.length + m / b.length + (m - t / 2) / m) / 3;
  let p = 0;
  while (p < 4 && a[p] === b[p]) p++;
  return jaro + p * 0.1 * (1 - jaro);
}

const FIELDS = [
  { key: "first", label: "First name", w: 0.2 },
  { key: "last", label: "Last name", w: 0.25 },
  { key: "dob", label: "Date of birth", w: 0.3 },
  { key: "phone", label: "Phone", w: 0.15 },
  { key: "zip", label: "ZIP", w: 0.1 },
] as const;
type Rec = Record<(typeof FIELDS)[number]["key"], string>;

export default function PipelinePage() {
  const [cloud, setCloud] = useState<Cloud>("aws");
  const [running, setRunning] = useState(-1);
  const [ran, setRan] = useState(false);
  const [a, setA] = useState<Rec>({ first: "María", last: "González-Ruiz", dob: "1968-04-12", phone: "516-555-0191", zip: "11550" });
  const [b, setB] = useState<Rec>({ first: "Maria", last: "Gonzalez", dob: "1968-04-12", phone: "(516) 555-0191", zip: "11550" });

  const match = useMemo(() => {
    const per = FIELDS.map((f) => {
      let s = f.key === "dob" ? (a.dob === b.dob ? 1 : a.dob.slice(0, 7) === b.dob.slice(0, 7) ? 0.4 : 0) : f.key === "phone" ? (norm(a.phone).slice(-10) === norm(b.phone).slice(-10) ? 1 : 0) : jaroWinkler(a[f.key], b[f.key]);
      if (f.key === "last" && s < 1) {
        const parts = a.last.split(/[-\s]/).concat(b.last.split(/[-\s]/));
        if (parts.some((p) => norm(p) && (norm(a.last).startsWith(norm(p)) && norm(b.last).startsWith(norm(p))))) s = Math.max(s, 0.95);
      }
      return { ...f, s };
    });
    const score = per.reduce((t, x) => t + x.s * x.w, 0);
    return { per, score, decision: score >= 0.92 ? "auto" : score >= 0.78 ? "review" : "distinct" };
  }, [a, b]);

  const run = () => {
    setRan(false);
    STAGES.forEach((_, i) => setTimeout(() => setRunning(i), i * 650));
    setTimeout(() => { setRunning(-1); setRan(true); }, STAGES.length * 650);
  };

  return (
    <div className="mx-auto max-w-7xl">
      <ToolHeader
        eyebrow="Data & integration · Centralized data pipelines"
        title="Data Pipeline & Patient Matching"
        description="Data from every system lands in one normalized, de-duplicated model, mapped to FHIR, coded with standard terminologies, quality-checked and loaded into a warehouse that powers dashboards, models and the agent. It runs on any major cloud."
        tags={["ETL / ELT", "FHIR R4 normalization", "MPI (Jaro-Winkler)", "data quality", "star schema", "AWS · Azure · GCP"]}
        impact={[{ value: "99.3%", label: "records passing DQ" }, { value: "< 5 min", label: "source → dashboard" }]}
      />

      <Card>
        <CardHeader
          title="Nightly + streaming pipeline"
          subtitle={ran ? "Last run: just now · 48,210 records in · 248 rejected to quarantine with reasons" : "HL7 streams in real time; bulk sources run nightly"}
          icon={<Database className="size-4" />}
          right={
            <div className="flex items-center gap-2">
              <Segmented value={cloud} onChange={setCloud} options={[{ value: "aws", label: "AWS" }, { value: "azure", label: "Azure" }, { value: "gcp", label: "GCP" }]} />
              <Button size="sm" onClick={run} disabled={running >= 0}>{running >= 0 ? <Loader2 className="size-3.5 animate-spin" /> : <Play className="size-3.5" />} Run</Button>
            </div>
          }
        />
        <div className="grid gap-3 p-5 md:grid-cols-5">
          {STAGES.map((s, i) => {
            const active = running === i;
            const done = ran || (running > i);
            return (
              <div key={s.key} className="relative">
                <div className={cx("h-full rounded-xl border p-4 transition-all duration-500", active ? "border-ocean bg-sky-2 shadow-[var(--shadow-lift)]" : done ? "border-sage/40 bg-sage-soft/40" : "border-line bg-cream/50")}>
                  <div className="flex items-center justify-between">
                    <span className="text-[11px] font-bold uppercase tracking-wide text-faint">{i + 1} · {s.title}</span>
                    {active ? <Loader2 className="size-3.5 animate-spin text-ocean" /> : done ? <CheckCircle2 className="size-3.5 text-sage" /> : null}
                  </div>
                  <div className="mt-2 font-mono text-[11.5px] font-semibold text-ocean">{s.svc[cloud]}</div>
                  <p className="mt-2 text-[12px] leading-relaxed text-muted">{s.detail}</p>
                  <div className="mt-3 font-display text-lg font-semibold tabular-nums text-ink">{done || active ? s.count.toLocaleString() : "—"}</div>
                </div>
                {i < STAGES.length - 1 && <div className={cx("absolute -right-2.5 top-1/2 z-10 hidden h-0.5 w-2 md:block", done ? "bg-sage" : "bg-line-2")} />}
              </div>
            );
          })}
        </div>
      </Card>

      <div className="mt-6 grid gap-6 xl:grid-cols-[1.2fr_1fr]">
        <Card>
          <CardHeader title="Patient matching (MPI)" subtitle="Same person from two systems? Edit either record to see the score change" icon={<GitMerge className="size-4" />} right={<Badge tone={match.decision === "auto" ? "sage" : match.decision === "review" ? "warn" : "neutral"}>{match.decision === "auto" ? "Auto-link" : match.decision === "review" ? "Human review" : "Different people"}</Badge>} />
          <div className="p-5">
            <div className="grid grid-cols-[110px_1fr_1fr_64px] items-center gap-2 text-[12px]">
              <div />
              <div className="font-semibold text-muted">EMR (eClinicalWorks)</div>
              <div className="font-semibold text-muted">Referral fax (OCR)</div>
              <div className="text-right font-semibold text-muted">Sim.</div>
              {match.per.map((f) => (
                <FieldRow key={f.key} label={f.label} a={a[f.key]} b={b[f.key]} s={f.s} onA={(v) => setA((x) => ({ ...x, [f.key]: v }))} onB={(v) => setB((x) => ({ ...x, [f.key]: v }))} />
              ))}
            </div>
            <div className="mt-5 flex items-center justify-between rounded-xl bg-sand p-4">
              <div className="text-[13px] text-muted">Weighted match score <span className="text-[11px]">(auto-link ≥ 0.92 · review ≥ 0.78)</span></div>
              <div className={cx("font-display text-3xl font-semibold tabular-nums", match.decision === "auto" ? "text-sage" : match.decision === "review" ? "text-warn" : "text-muted")}>{match.score.toFixed(3)}</div>
            </div>
            <p className="mt-3 text-[12px] text-muted">Accent-insensitive Jaro-Winkler on names, compound-surname handling, exact DOB and phone matching. Borderline pairs go to a review queue, never an automatic merge.</p>
          </div>
        </Card>

        <Card>
          <CardHeader title="Data-quality checks" subtitle="Failing records are quarantined with a reason, not dropped" icon={<TriangleAlert className="size-4" />} />
          <ul className="divide-y divide-line">
            {DQ.map((d) => (
              <li key={d.check} className="px-5 py-3">
                <div className="flex items-center justify-between gap-3 text-[13px]"><span className="text-ink">{d.check}</span><span className="font-semibold tabular-nums text-ink">{d.pass}%</span></div>
                <div className="mt-1.5 flex items-center gap-3">
                  <div className="h-1.5 flex-1 rounded-full bg-sand"><div className="h-1.5 rounded-full bg-sage" style={{ width: `${d.pass}%` }} /></div>
                  <span className="w-24 text-right text-[11px] text-muted">{d.issues} quarantined</span>
                </div>
              </li>
            ))}
          </ul>
        </Card>
      </div>

      <div className="mt-6 grid gap-6 xl:grid-cols-2">
        <Card className="overflow-hidden">
          <CardHeader title="Terminology normalization" subtitle="Local codes and free text → standard code systems" />
          <div className="overflow-x-auto">
            <table className="w-full min-w-[520px] text-[12.5px]">
              <thead className="bg-cream text-left text-[11px] uppercase tracking-wide text-faint"><tr><th className="px-5 py-2.5">Source</th><th className="px-2 py-2.5">Raw value</th><th className="px-2 py-2.5">System</th><th className="px-5 py-2.5">Standard code</th></tr></thead>
              <tbody className="divide-y divide-line">
                {TERMS.map((t) => (
                  <tr key={t[1]}><td className="px-5 py-2.5 text-muted">{t[0]}</td><td className="px-2 py-2.5 font-mono text-ink">{t[1]}</td><td className="px-2 py-2.5"><Badge tone="ocean">{t[2]}</Badge></td><td className="px-5 py-2.5 text-ink">{t[3]}</td></tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
        <Card className="overflow-hidden">
          <CardHeader title="Warehouse query" subtitle="Star schema: fact_authorization, fact_claim, fact_appointment + dimensions" icon={<Table2 className="size-4" />} />
          <div className="p-4"><CodeBlock value={SQL} maxH="max-h-64" /></div>
          <table className="w-full text-[12.5px]">
            <thead className="bg-cream text-left text-[11px] uppercase tracking-wide text-faint"><tr><th className="px-5 py-2">payer_name</th><th className="px-2 py-2 text-right">requests</th><th className="px-2 py-2 text-right">median</th><th className="px-5 py-2 text-right">denial_rate</th></tr></thead>
            <tbody className="divide-y divide-line">
              {SQL_ROWS.map((r) => <tr key={r[0] as string}><td className="px-5 py-2 text-ink">{r[0]}</td><td className="px-2 py-2 text-right tabular-nums text-muted">{r[1]}</td><td className="px-2 py-2 text-right tabular-nums text-ink">{r[2]}</td><td className="px-5 py-2 text-right tabular-nums text-muted">{r[3]}</td></tr>)}
            </tbody>
          </table>
        </Card>
      </div>
    </div>
  );
}

function FieldRow({ label, a, b, s, onA, onB }: { label: string; a: string; b: string; s: number; onA: (v: string) => void; onB: (v: string) => void }) {
  return (
    <>
      <div className="font-semibold text-ink">{label}</div>
      <input value={a} onChange={(e) => onA(e.target.value)} className={cx(inputCls, "!py-1.5 !text-[12.5px]")} />
      <input value={b} onChange={(e) => onB(e.target.value)} className={cx(inputCls, "!py-1.5 !text-[12.5px]")} />
      <div className={cx("text-right font-mono text-[12px] font-semibold", s >= 0.95 ? "text-sage" : s >= 0.8 ? "text-warn" : "text-danger")}>{s.toFixed(2)}</div>
    </>
  );
}
