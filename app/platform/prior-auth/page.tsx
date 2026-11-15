"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import { AlertTriangle, Check, CheckCircle2, CircleDashed, Clock, Copy, FileText, Loader2, Paperclip, Send, Sparkles, XCircle, MinusCircle } from "lucide-react";
import { ToolHeader } from "@/components/platform/shell";
import { Badge, Button, Card, CardHeader, Progress, cx } from "@/components/ui";
import { AUTH_CASES, evaluateCase, medicalNecessityLetter } from "@/lib/data/prior-auth";
import { extractEntities } from "@/lib/nlp";
import { HighlightedText } from "@/components/platform/highlight";

const STEPS = [
  { key: "elig", label: "Eligibility & benefits", detail: "X12 270/271 · active coverage, plan type, in-network" },
  { key: "crd", label: "Is authorization required?", detail: "Payer rules / Da Vinci CRD lookup for CPT + plan" },
  { key: "nlp", label: "Extract clinical evidence", detail: "NLP over notes, imaging and PT records" },
  { key: "crit", label: "Match payer criteria", detail: "Criterion-by-criterion with cited evidence" },
  { key: "pkt", label: "Assemble packet", detail: "Questionnaire auto-filled (DTR), attachments linked" },
  { key: "ltr", label: "Draft medical-necessity letter", detail: "Provider reviews and signs" },
  { key: "sub", label: "Submit & track", detail: "Portal / X12 278 / FHIR PAS · status polled hourly" },
];

export default function PriorAuthPage() {
  const [selected, setSelected] = useState(AUTH_CASES[0].id);
  const [phase, setPhase] = useState<"idle" | "running" | "done">("idle");
  const [stepIdx, setStepIdx] = useState(-1);
  const [submitted, setSubmitted] = useState<string[]>([]);
  const [copied, setCopied] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const c = AUTH_CASES.find((x) => x.id === selected)!;
  const ev = useMemo(() => evaluateCase(c), [c]);
  const entities = useMemo(() => extractEntities(c.note), [c]);
  const notRequired = c.pathway === "NY WCB Medical Portal";
  const blocked = ev.blocking.length > 0;
  const letter = useMemo(() => medicalNecessityLetter(c, ev.results), [c, ev]);

  useEffect(() => {
    setPhase("idle");
    setStepIdx(-1);
    if (timer.current) clearTimeout(timer.current);
  }, [selected]);

  const run = () => {
    setPhase("running");
    setStepIdx(0);
    const tick = (i: number) => {
      timer.current = setTimeout(() => {
        const lastStep = blocked ? 3 : STEPS.length - 2;
        if (i >= lastStep) {
          setStepIdx(i + 1);
          setPhase("done");
          return;
        }
        setStepIdx(i + 1);
        tick(i + 1);
      }, 520);
    };
    tick(0);
  };

  const stepState = (i: number) => {
    if (phase === "idle") return "pending";
    if (blocked && i > 3 && phase === "done") return "skipped";
    if (i < stepIdx) return "done";
    if (i === stepIdx && phase === "running") return "active";
    if (phase === "done" && i === STEPS.length - 1) return submitted.includes(c.id) ? "done" : "pending";
    return i <= stepIdx ? "done" : "pending";
  };

  return (
    <div className="mx-auto max-w-7xl">
      <ToolHeader
        eyebrow="Revenue cycle · Insurance authorization support"
        title="Prior Auth Copilot"
        description="Reads the chart, checks payer criteria with cited evidence, fills the questionnaire, drafts the letter of medical necessity and submits. It also tells staff when a request will be denied, before anyone sends it."
        tags={["NLP evidence extraction", "X12 270/271 · 278", "Da Vinci CRD · DTR · PAS", "NY WCB MTG", "CMS-0057-F"]}
        impact={[{ value: "4 min", label: "per request (was 45)" }, { value: "2.6 d", label: "median decision" }]}
      />

      <div className="grid gap-6 xl:grid-cols-[300px_1fr]">
        <Card className="h-fit overflow-hidden">
          <CardHeader title="Queue" subtitle={`${AUTH_CASES.length} requests · 2 urgent today`} />
          <ul className="divide-y divide-line">
            {AUTH_CASES.map((a) => {
              const e = evaluateCase(a);
              const isSel = a.id === selected;
              return (
                <li key={a.id}>
                  <button onClick={() => setSelected(a.id)} className={cx("w-full px-4 py-3.5 text-left transition", isSel ? "bg-sky-2 shadow-[inset_3px_0_0_var(--color-ocean)]" : "hover:bg-cream")}>
                    <div className="flex items-center justify-between gap-2">
                      <span className="font-mono text-[11px] text-faint">{a.id}</span>
                      {submitted.includes(a.id) ? <Badge tone="sage">Submitted</Badge> : a.pathway === "NY WCB Medical Portal" ? <Badge tone="ocean">Within MTG</Badge> : e.blocking.length ? <Badge tone="warn">Not ready</Badge> : <Badge tone="sage">Ready</Badge>}
                    </div>
                    <div className="mt-1 text-[14px] font-semibold text-ink">{a.patient}</div>
                    <div className="text-[12px] text-muted">CPT {a.cpt} · {a.payer}</div>
                    <div className="mt-2 flex items-center gap-2">
                      <Progress value={e.readinessAfterFix} tone={e.blocking.length ? "warn" : "sage"} />
                      <span className="text-[11px] font-semibold tabular-nums text-muted">{e.readinessAfterFix}%</span>
                    </div>
                  </button>
                </li>
              );
            })}
          </ul>
        </Card>

        <div className="space-y-6">
          <Card>
            <div className="flex flex-wrap items-start justify-between gap-4 p-5">
              <div>
                <div className="flex flex-wrap items-center gap-2">
                  <h2 className="font-display text-2xl font-semibold text-ink">{c.patient}</h2>
                  <Badge tone="neutral">{c.payerType}</Badge>
                  <Badge tone="ocean">{c.pathway}</Badge>
                </div>
                <p className="mt-1 text-[14px] text-ink"><span className="font-mono font-semibold">CPT {c.cpt}</span> · {c.service}</p>
                <p className="mt-0.5 text-[13px] text-muted">Dx {c.icd10.join(", ")} · Ordering: {c.provider} · Received {c.receivedDaysAgo === 0 ? "today" : `${c.receivedDaysAgo} d ago`}</p>
              </div>
              <div className="flex items-center gap-2">
                {phase !== "running" && (
                  <Button onClick={run} variant={phase === "done" ? "secondary" : "primary"}>
                    <Sparkles className="size-4" /> {phase === "done" ? "Run again" : "Run Copilot"}
                  </Button>
                )}
                {phase === "running" && <Button disabled><Loader2 className="size-4 animate-spin" /> Working…</Button>}
              </div>
            </div>
            <div className="grid gap-px border-t border-line bg-line md:grid-cols-7">
              {STEPS.map((s, i) => {
                const st = stepState(i);
                return (
                  <div key={s.key} className={cx("bg-white p-3 transition-colors", st === "active" && "bg-sky-2")}>
                    <div className="flex items-center gap-1.5">
                      {st === "done" && <CheckCircle2 className="size-4 text-sage" />}
                      {st === "active" && <Loader2 className="size-4 animate-spin text-ocean" />}
                      {st === "pending" && <CircleDashed className="size-4 text-line-2" />}
                      {st === "skipped" && <MinusCircle className="size-4 text-warn" />}
                      <span className="text-[11px] font-bold text-faint">{i + 1}</span>
                    </div>
                    <div className="mt-1.5 text-[12px] font-semibold leading-snug text-ink">{s.label}</div>
                    <div className="mt-0.5 hidden text-[11px] leading-snug text-muted xl:block">{s.detail}</div>
                  </div>
                );
              })}
            </div>
          </Card>

          <div className="grid gap-6 lg:grid-cols-2">
            <Card>
              <CardHeader title="Chart evidence" subtitle="Entities found by the NLP layer are highlighted" icon={<FileText className="size-4" />} />
              <div className="p-5 text-[14px] leading-7 text-ink">
                <HighlightedText text={c.note} entities={entities} />
              </div>
              <div className="flex flex-wrap gap-2 border-t border-line px-5 py-3">
                {c.attachments.map((a) => (
                  <span key={a} className="inline-flex items-center gap-1 rounded-lg bg-sand px-2 py-1 text-[11px] font-medium text-muted"><Paperclip className="size-3" />{a}</span>
                ))}
              </div>
            </Card>

            <Card>
              <CardHeader
                title="Payer criteria"
                subtitle={notRequired ? "NY Workers' Compensation Medical Treatment Guidelines" : `${c.payer} policy for CPT ${c.cpt}`}
                right={
                  <div className="text-right">
                    <div className="font-display text-2xl font-semibold tabular-nums text-ink">{phase === "done" ? ev.readinessAfterFix : ev.readiness}%</div>
                    <div className="text-[11px] text-muted">readiness</div>
                  </div>
                }
              />
              <ul className="divide-y divide-line">
                {ev.results.map((r) => {
                  const autoFixed = phase === "done" && r.status === "missing" && r.fix?.startsWith("Auto-pulled");
                  const status = autoFixed ? "met" : r.status;
                  return (
                    <li key={r.label} className="px-5 py-3.5">
                      <div className="flex items-start gap-2.5">
                        {status === "met" && <Check className="mt-0.5 size-4 shrink-0 text-sage" />}
                        {status === "missing" && <XCircle className="mt-0.5 size-4 shrink-0 text-danger" />}
                        {status === "na" && <MinusCircle className="mt-0.5 size-4 shrink-0 text-faint" />}
                        <div className="min-w-0">
                          <div className="text-[13px] font-semibold text-ink">{r.label}</div>
                          <div className="mt-0.5 text-[12px] leading-relaxed text-muted">{r.evidence || "—"}</div>
                          {r.status === "missing" && r.fix && (
                            <div className={cx("mt-2 rounded-lg px-2.5 py-1.5 text-[12px]", autoFixed ? "bg-sage-soft text-sage" : "bg-warn-soft text-warn")}>
                              {autoFixed ? "✓ " : ""}{r.fix}
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

          {phase === "done" && (
            <div className="animate-fade-up">
              {notRequired ? (
                <Card className="border-ocean/30 bg-sky-2 p-5">
                  <div className="flex items-start gap-3">
                    <CheckCircle2 className="mt-0.5 size-5 text-ocean" />
                    <div>
                      <div className="text-[15px] font-semibold text-ink">No authorization request needed: treatment is within the NY Medical Treatment Guidelines.</div>
                      <p className="mt-1 text-[13px] text-muted">Copilot documented MTG compliance in the chart, scheduled the procedure and set the bill to include the WCB case number. A variance PAR in the WCB Medical Portal is only drafted when a request falls outside the guidelines.</p>
                      <Button className="mt-3" size="sm" onClick={() => setSubmitted((s) => [...s, c.id])} disabled={submitted.includes(c.id)}>
                        {submitted.includes(c.id) ? "Scheduled ✓" : "Schedule procedure"}
                      </Button>
                    </div>
                  </div>
                </Card>
              ) : blocked ? (
                <Card className="border-warn/30 bg-warn-soft p-5">
                  <div className="flex items-start gap-3">
                    <AlertTriangle className="mt-0.5 size-5 text-warn" />
                    <div>
                      <div className="text-[15px] font-semibold text-ink">Hold. This request would likely be denied as written.</div>
                      <ul className="mt-2 space-y-1 text-[13px] text-muted">
                        {ev.blocking.map((b) => <li key={b.label}>• <b className="text-ink">{b.label}:</b> {b.fix ?? b.evidence}</li>)}
                      </ul>
                      <p className="mt-2 text-[13px] text-muted">Copilot added a task to re-evaluate on <b className="text-ink">Mon, Nov 30</b> (week 6 of therapy) and will re-run automatically when the PT note arrives. One avoided denial saves about 2 hours of rework and a two-week delay for the patient.</p>
                    </div>
                  </div>
                </Card>
              ) : (
                <div className="grid gap-6 lg:grid-cols-[1.3fr_1fr]">
                  <Card>
                    <CardHeader
                      title="Letter of medical necessity (draft)"
                      subtitle="Generated from criteria evidence. Requires provider review and signature."
                      right={
                        <Button size="sm" variant="secondary" onClick={() => { navigator.clipboard?.writeText(letter); setCopied(true); setTimeout(() => setCopied(false), 1500); }}>
                          <Copy className="size-3.5" /> {copied ? "Copied" : "Copy"}
                        </Button>
                      }
                    />
                    <textarea defaultValue={letter} key={c.id} className="scroll-thin h-80 w-full resize-none bg-cream/50 p-5 font-mono text-[12px] leading-relaxed text-ink outline-none" />
                  </Card>
                  <Card className="flex flex-col">
                    <CardHeader title="Submission" subtitle={c.pathway} />
                    <div className="flex-1 space-y-3 p-5 text-[13px]">
                      <div className="flex justify-between"><span className="text-muted">Packet</span><span className="font-semibold text-ink">{c.attachments.length + 2} documents</span></div>
                      <div className="flex justify-between"><span className="text-muted">Questionnaire</span><span className="font-semibold text-ink">auto-filled 14 / 14</span></div>
                      <div className="flex justify-between"><span className="text-muted">Staff time</span><span className="font-semibold text-sage">4 min (vs. ~45 min manual)</span></div>
                      <div className="flex justify-between"><span className="text-muted">Expected decision</span><span className="font-semibold text-ink">{c.payerType === "Medicare Advantage" ? "≤ 7 calendar days (CMS-0057-F)" : "3–5 business days"}</span></div>
                      {c.payerType === "Medicare Advantage" && (
                        <p className="rounded-lg bg-sky-2 p-2.5 text-[12px] text-ocean">Medicare Advantage plans must decide standard requests within 7 calendar days (72 h expedited) and expose a FHIR Prior Authorization API by Jan 1, 2027. This request goes through that API.</p>
                      )}
                    </div>
                    <div className="border-t border-line p-5">
                      {submitted.includes(c.id) ? (
                        <div className="flex items-center gap-2 text-[13px] font-semibold text-sage"><Clock className="size-4" /> Submitted · tracking # PA-{c.id.slice(3)}-NY · status checked hourly</div>
                      ) : (
                        <Button className="w-full" variant="sage" onClick={() => setSubmitted((s) => [...s, c.id])}><Send className="size-4" /> Approve &amp; submit</Button>
                      )}
                    </div>
                  </Card>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
