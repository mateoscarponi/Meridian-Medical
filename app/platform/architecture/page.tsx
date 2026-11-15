"use client";

import { useState } from "react";
import { Cloud, Code2, Flag, Layers, Lock, Rocket, Search, Users } from "lucide-react";
import { ToolHeader } from "@/components/platform/shell";
import { Card, CardHeader, CodeBlock, Segmented, cx } from "@/components/ui";

const LAYERS = [
  { name: "Channels", tone: "bg-gold-soft text-gold", items: ["Practice website & smart intake", "Patient portal", "SMS / IVR / voicemail", "Staff console", "Fax & email"] },
  { name: "Applications", tone: "bg-sky-2 text-ocean", items: ["Prior Auth Copilot", "Document AI", "Clinical Scribe", "Claim Scrubber", "Scheduling", "Referrals", "AI Agent", "Command Center"] },
  { name: "AI & services", tone: "bg-sage-soft text-sage", items: ["LLM gateway (Claude / OpenAI, BAA, ZDR)", "Clinical NLP", "OCR / Document AI", "ML models (no-show, denial risk)", "Rules engine", "Workflow orchestrator"] },
  { name: "Integration", tone: "bg-[#ece3f3] text-[#6b4a93]", items: ["FHIR R4 gateway (SMART)", "HL7 v2 MLLP listener", "X12 via clearinghouse", "Payer PA APIs (Da Vinci)", "Fax / SMS providers", "WCB Medical Portal"] },
  { name: "Data", tone: "bg-sand text-ink", items: ["Operational Postgres", "FHIR store", "Warehouse (star schema)", "Object storage (raw)", "Feature store", "Immutable audit log"] },
];

const STACK = [
  { area: "Languages", items: ["TypeScript (Next.js, Node)", "Python (FastAPI, pandas, scikit-learn, PyTorch)", "Go (high-throughput HL7 listener)", "Java (Mirth Connect channels) · C# (.NET vendor SDKs)", "SQL"] },
  { area: "AI / ML", items: ["Claude & OpenAI APIs", "LangChain-style tool orchestration", "spaCy / transformer NER", "TensorFlow / PyTorch for custom models", "Evals & guardrails"] },
  { area: "Healthcare standards", items: ["HL7 v2.5.1", "FHIR R4 · US Core · SMART", "X12 5010 (270/271, 276/277, 278, 837P, 835)", "ICD-10-CM · CPT · LOINC"] },
  { area: "Cloud & infra", items: ["AWS (HIPAA-eligible services) or Azure / GCP", "Docker · Terraform · GitHub Actions", "Queues + dead-letter + retries", "OpenTelemetry monitoring"] },
];

const ROADMAP = [
  { phase: "Days 1–30", title: "Listen, measure, quick wins", icon: Search, items: ["Shadow front desk, auth, billing and providers at each site", "2-week time study → baseline for every KPI", "Read-only EMR / clearinghouse access and data map", "Ship: pre-visit checklist SMS and fax-intake classification"] },
  { phase: "Days 31–60", title: "Revenue cycle automation", icon: Rocket, items: ["Prior Auth Copilot for the top 5 CPTs by volume", "Claim scrubber rules from 12 months of 835 denials", "Central pipeline + Command Center dashboard", "Weekly KPI review with operations"] },
  { phase: "Days 61–90", title: "Scale & intelligence", icon: Flag, items: ["No-show model live with waitlist backfill", "Clinical scribe pilot with 1–2 providers", "AI agent for rescheduling & status questions (EN/ES)", "90-day results vs. baseline → next-quarter roadmap"] },
];

const SNIPPETS = {
  ts: `// HL7 v2 → FHIR (lib/hl7.ts, simplified)
export function parseHl7(input: string): Hl7Message {
  const lines = input.replace(/\\r\\n|\\n/g, "\\r").split("\\r").filter(Boolean);
  const fieldSep = lines[0][3];                 // MSH-1
  const [comp, rep, esc, sub] = lines[0].slice(4, 8); // MSH-2
  const segments = lines.map((line) => {
    const [id, ...fields] = line.split(fieldSep);
    return { id, fields: fields.map((raw) =>
      raw.split(rep).map((r) => r.split(comp).map(unescape))) };
  });
  return { segments, type: field(segments[0], 9), controlId: field(segments[0], 10) };
}`,
  py: `# No-show model (ml/no_show_model.py, excerpt): stdlib-only logistic regression
def train(X, y, lr=0.9, epochs=700, l2=1e-4):
    w, b = [0.0] * len(X[0]), 0.0
    for _ in range(epochs):
        gw, gb = [0.0] * len(w), 0.0
        for xi, yi in zip(X, y):
            err = sigmoid(b + dot(w, xi)) - yi
            gb += err
            gw = [g + err * v for g, v in zip(gw, xi)]
        b -= lr * gb / len(X)
        w = [wj - lr * (gj / len(X) + l2 * wj) for wj, gj in zip(w, gw)]
    return b, w   # exported to lib/models/no-show-weights.json`,
  llm: `// LLM call with on-device fallback (app/api/ai/route.ts, excerpt)
const response = await client.beta.messages.create({
  model: "claude-opus-5-5",
  max_tokens: 4000,
  betas: ["server-side-fallback-2026-07-01"],
  fallbacks: "default",
  output_config: { effort: "low" },
  system: SYSTEM.scribe,               // documentation-only, no diagnosis
  messages: [{ role: "user", content: transcript }],
});
if (response.stop_reason === "refusal") return localFallback();`,
};

export default function ArchitecturePage() {
  const [snip, setSnip] = useState<keyof typeof SNIPPETS>("ts");
  return (
    <div className="mx-auto max-w-7xl">
      <ToolHeader
        eyebrow="Trust & build · How it fits together"
        title="Platform Architecture & Rollout"
        description="A modular platform that sits next to the EMR instead of replacing it: standards-based integration, a central data layer, AI services behind guardrails, and a 90-day rollout plan built around measurable outcomes."
        tags={["modular services", "standards-first", "cloud-agnostic", "secure by default", "measured rollout"]}
      />

      <Card>
        <CardHeader title="Reference architecture" icon={<Layers className="size-4" />} subtitle="Every layer is replaceable; standards keep it vendor-neutral" />
        <div className="space-y-2.5 p-5">
          {LAYERS.map((l) => (
            <div key={l.name} className="grid items-stretch gap-2.5 md:grid-cols-[150px_1fr]">
              <div className={cx("flex items-center rounded-xl px-4 py-3 text-[13px] font-bold", l.tone)}>{l.name}</div>
              <div className="flex flex-wrap gap-2 rounded-xl border border-line bg-cream/50 p-2.5">
                {l.items.map((i) => <span key={i} className="rounded-lg bg-white px-3 py-1.5 text-[12.5px] font-medium text-ink shadow-sm ring-1 ring-line">{i}</span>)}
              </div>
            </div>
          ))}
          <div className="mt-1 flex items-center gap-2 rounded-xl bg-ink px-4 py-3 text-[12.5px] text-white/80"><Lock className="size-4 text-gold-2" /> Cross-cutting: SSO/MFA · RBAC · encryption · audit logging · PHI redaction · monitoring & alerting · BAAs</div>
        </div>
      </Card>

      <div className="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {[
          ["99.95%", "uptime target (SLO)", "multi-AZ, health checks, auto-restart"],
          ["< 200 ms", "p95 API latency", "caching + read replicas for dashboards"],
          ["0 lost messages", "HL7 & X12 delivery", "durable queues, retries, dead-letter + replay"],
          ["< 1 h / 15 min", "RTO / RPO", "PITR backups, infra as code, DR runbook"],
        ].map(([v, l, d]) => (
          <Card key={l} className="p-5">
            <div className="font-display text-2xl font-semibold text-ink">{v}</div>
            <div className="text-[12.5px] font-semibold text-ocean">{l}</div>
            <div className="mt-1 text-[12px] text-muted">{d}</div>
          </Card>
        ))}
      </div>

      <div className="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {STACK.map((s) => (
          <Card key={s.area} className="p-5">
            <div className="flex items-center gap-2 text-[13px] font-semibold text-ink">{s.area === "Cloud & infra" ? <Cloud className="size-4 text-ocean" /> : <Code2 className="size-4 text-ocean" />}{s.area}</div>
            <ul className="mt-3 space-y-1.5 text-[12.5px] text-muted">{s.items.map((i) => <li key={i}>• {i}</li>)}</ul>
          </Card>
        ))}
      </div>

      <Card className="mt-6">
        <CardHeader title="First 90 days" subtitle="Start with the people doing the work, then automate what measurably helps" icon={<Users className="size-4" />} />
        <div className="grid gap-px bg-line md:grid-cols-3">
          {ROADMAP.map((r) => (
            <div key={r.phase} className="bg-white p-5">
              <div className="flex items-center gap-2"><div className="grid size-8 place-items-center rounded-lg bg-navy text-white"><r.icon className="size-4" /></div><div><div className="text-[11px] font-bold uppercase tracking-wide text-gold">{r.phase}</div><div className="text-[14px] font-semibold text-ink">{r.title}</div></div></div>
              <ul className="mt-4 space-y-2 text-[12.5px] text-muted">{r.items.map((i) => <li key={i} className="flex gap-2"><span className="mt-[7px] size-1.5 shrink-0 rounded-full bg-sage" />{i}</li>)}</ul>
            </div>
          ))}
        </div>
      </Card>

      <Card className="mt-6">
        <CardHeader title="From the codebase" subtitle="Real excerpts from this demo" icon={<Code2 className="size-4" />} right={<Segmented value={snip} onChange={setSnip} options={[{ value: "ts", label: "TypeScript · HL7" }, { value: "py", label: "Python · ML" }, { value: "llm", label: "LLM API" }]} />} />
        <div className="p-4"><CodeBlock value={SNIPPETS[snip]} maxH="max-h-96" /></div>
      </Card>
    </div>
  );
}
