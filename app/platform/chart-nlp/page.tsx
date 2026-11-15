"use client";

import { useMemo, useState } from "react";
import { AlertOctagon, Braces, CheckCircle2, RotateCcw, ScanText, ShieldAlert } from "lucide-react";
import { ToolHeader } from "@/components/platform/shell";
import { Badge, Card, CardHeader, CodeBlock, Segmented, cx } from "@/components/ui";
import { HighlightedText, ENTITY_SWATCH } from "@/components/platform/highlight";
import { ENTITY_LABELS, SAMPLE_NOTE, conservativeCareWeeks, extractEntities, redFlags, suggestIcd10, type EntityType } from "@/lib/nlp";

const SAMPLES = {
  lumbar: SAMPLE_NOTE,
  urgent: `62-year-old female 3 weeks after L4-L5 microdiscectomy calls with new saddle anesthesia and urinary retention since this morning, with progressive weakness in both legs. Reports low back pain 8/10. No fever. Taking gabapentin and acetaminophen.`,
  cervical: `Patient with neck pain and pain radiating into the left arm for 4 months, numbness in the thumb. Completed 12 weeks of physical therapy and 8 weeks of gabapentin. MRI of the cervical spine shows a left foraminal disc herniation at C5-C6 with nerve root compression. No myelopathy. Considering ACDF.`,
};

export default function ChartNlpPage() {
  const [sample, setSample] = useState<keyof typeof SAMPLES>("lumbar");
  const [text, setText] = useState(SAMPLES.lumbar);
  const [hidden, setHidden] = useState<EntityType[]>([]);
  const [view, setView] = useState<"summary" | "json">("summary");

  const entities = useMemo(() => extractEntities(text), [text]);
  const rf = useMemo(() => redFlags(entities), [entities]);
  const icd = useMemo(() => suggestIcd10(text, entities), [text, entities]);
  const cc = useMemo(() => conservativeCareWeeks(text), [text]);

  const byType = (t: EntityType) => entities.filter((e) => e.type === t && !e.negated).map((e) => e.text);
  const structured = {
    symptoms: byType("symptom"),
    anatomy: [...new Set(byType("anatomy"))],
    imaging_findings: byType("imaging"),
    medications: byType("medication"),
    conservative_care: { modalities: byType("therapy"), documented_weeks: cc.weeks },
    procedures_planned_or_prior: byType("procedure"),
    red_flags: { present: rf.active.map((e) => e.text), explicitly_denied: rf.negated.map((e) => e.text) },
    icd10_suggestions: icd.map((i) => ({ code: i.code, display: i.display, confidence: Number(i.confidence.toFixed(2)) })),
  };

  return (
    <div className="mx-auto max-w-7xl">
      <ToolHeader
        eyebrow="Clinical AI · NLP-based chart analysis"
        title="Chart NLP"
        description="Turns free-text notes into structured data: symptoms, levels, imaging findings, conservative-care duration, red flags (with negation), and ICD-10 suggestions. It runs entirely in the browser, so no PHI leaves the device. Edit the note and watch it update."
        tags={["named-entity recognition", "NegEx negation", "duration normalization", "ICD-10-CM", "red-flag triage"]}
        impact={[{ value: `${entities.length}`, label: "entities found" }, { value: `${cc.weeks} wk`, label: "conservative care" }]}
      />

      <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
        <Segmented
          value={sample}
          onChange={(v) => { setSample(v); setText(SAMPLES[v]); }}
          options={[{ value: "lumbar", label: "Lumbar radiculopathy" }, { value: "urgent", label: "Post-op red flags" }, { value: "cervical", label: "Cervical / ACDF" }]}
        />
        <button onClick={() => setText(SAMPLES[sample])} className="inline-flex items-center gap-1.5 text-[13px] font-semibold text-muted hover:text-ink"><RotateCcw className="size-3.5" /> Reset text</button>
      </div>

      {rf.urgent && (
        <div className="animate-fade-up mb-5 flex items-start gap-3 rounded-2xl border border-danger/30 bg-danger-soft p-4 text-danger" role="alert">
          <AlertOctagon className="mt-0.5 size-5 shrink-0" />
          <div>
            <div className="text-[15px] font-semibold">Red flags present: possible cauda equina / progressive neurologic deficit</div>
            <div className="mt-0.5 text-[13px]">Found: {rf.active.map((r) => r.text).join(", ")}. Routed to the on-call surgeon with a callback within 15 minutes; patient instructed to go to the ER.</div>
          </div>
        </div>
      )}

      <div className="grid gap-6 xl:grid-cols-2">
        <Card>
          <CardHeader title="Clinical note" subtitle="Type or paste any spine / orthopedic note" icon={<ScanText className="size-4" />} />
          <textarea value={text} onChange={(e) => setText(e.target.value)} className="scroll-thin h-44 w-full resize-y border-b border-line bg-cream/40 p-5 text-[14px] leading-relaxed text-ink outline-none" />
          <div className="p-5 text-[14px] leading-7 text-ink">
            <HighlightedText text={text} entities={entities} hidden={hidden} />
          </div>
          <div className="flex flex-wrap gap-1.5 border-t border-line px-5 py-3">
            {(Object.keys(ENTITY_LABELS) as EntityType[]).map((t) => {
              const n = entities.filter((e) => e.type === t).length;
              const off = hidden.includes(t);
              return (
                <button key={t} onClick={() => setHidden((h) => (off ? h.filter((x) => x !== t) : [...h, t]))} className={cx("inline-flex items-center gap-1.5 rounded-lg px-2 py-1 text-[11px] font-semibold ring-1 ring-inset ring-line transition", off ? "opacity-40" : "bg-white")}>
                  <span className={cx("ent size-2.5 rounded-sm !p-0", ENTITY_SWATCH[t])} />
                  {ENTITY_LABELS[t]} <span className="text-faint">{n}</span>
                </button>
              );
            })}
            <span className="ml-auto self-center text-[11px] text-faint"><span className="line-through">struck</span> = negated</span>
          </div>
        </Card>

        <div className="space-y-6">
          <Card>
            <CardHeader title="Structured output" subtitle="What gets written back to the chart / data pipeline" icon={<Braces className="size-4" />} right={<Segmented value={view} onChange={setView} options={[{ value: "summary", label: "Summary" }, { value: "json", label: "JSON" }]} />} />
            {view === "json" ? (
              <div className="p-4"><CodeBlock value={structured} maxH="max-h-[460px]" /></div>
            ) : (
              <div className="divide-y divide-line">
                <Row label="Symptoms" items={structured.symptoms} />
                <Row label="Levels / anatomy" items={structured.anatomy} />
                <Row label="Imaging" items={structured.imaging_findings} />
                <Row label="Medications" items={structured.medications} />
                <div className="grid grid-cols-[140px_1fr] gap-3 px-5 py-3 text-[13px]">
                  <div className="font-semibold text-muted">Conservative care</div>
                  <div className="text-ink">
                    <b>{cc.weeks} weeks</b> documented{structured.conservative_care.modalities.length ? ` · ${structured.conservative_care.modalities.join(", ")}` : ""}
                    <div className="mt-1"><Badge tone={cc.weeks >= 6 ? "sage" : "warn"}>{cc.weeks >= 6 ? "Meets typical 6-week payer threshold" : "Below typical 6-week threshold"}</Badge></div>
                  </div>
                </div>
                <div className="grid grid-cols-[140px_1fr] gap-3 px-5 py-3 text-[13px]">
                  <div className="font-semibold text-muted">Red flags</div>
                  <div>
                    {rf.active.length ? <span className="font-semibold text-danger"><ShieldAlert className="mr-1 inline size-3.5" />{rf.active.map((r) => r.text).join(", ")}</span> : <span className="text-sage"><CheckCircle2 className="mr-1 inline size-3.5" />None active</span>}
                    {rf.negated.length > 0 && <div className="mt-1 text-[12px] text-muted">Explicitly denied: {rf.negated.map((r) => r.text).join(", ")}</div>}
                  </div>
                </div>
              </div>
            )}
          </Card>
          <Card>
            <CardHeader title="ICD-10-CM suggestions" subtitle="Specific codes supersede generic ones; the coder confirms" />
            <ul className="divide-y divide-line">
              {icd.length === 0 && <li className="px-5 py-4 text-[13px] text-muted">No codable findings yet.</li>}
              {icd.map((i) => (
                <li key={i.code} className="flex items-center justify-between gap-4 px-5 py-3">
                  <div>
                    <div className="text-[13px] font-semibold text-ink"><span className="mr-2 font-mono text-ocean">{i.code}</span>{i.display}</div>
                    <div className="text-[12px] text-muted">{i.why}</div>
                  </div>
                  <div className="w-20 text-right">
                    <div className="text-[12px] font-semibold tabular-nums text-ink">{Math.round(i.confidence * 100)}%</div>
                    <div className="mt-1 h-1 rounded-full bg-sand"><div className="h-1 rounded-full bg-sage" style={{ width: `${i.confidence * 100}%` }} /></div>
                  </div>
                </li>
              ))}
            </ul>
          </Card>
        </div>
      </div>
    </div>
  );
}

function Row({ label, items }: { label: string; items: string[] }) {
  return (
    <div className="grid grid-cols-[140px_1fr] gap-3 px-5 py-3 text-[13px]">
      <div className="font-semibold text-muted">{label}</div>
      <div className="flex flex-wrap gap-1.5">{items.length ? items.map((t, i) => <span key={i} className="rounded-md bg-sand px-2 py-0.5 text-ink">{t}</span>) : <span className="text-faint">—</span>}</div>
    </div>
  );
}
