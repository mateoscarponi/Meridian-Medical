"use client";

import { useMemo, useState } from "react";
import { AlertTriangle, CheckCircle2, ShieldCheck, Wand2, XOctagon } from "lucide-react";
import { ToolHeader } from "@/components/platform/shell";
import { Badge, Button, Card, CardHeader, CodeBlock, Segmented, Stat, cx, money } from "@/components/ui";
import { Bars } from "@/components/platform/charts";
import { CLAIMS, denialRisk, total, type Claim } from "@/lib/data/claims";

function riskTone(r: number) {
  return r >= 50 ? "danger" : r >= 25 ? "warn" : "sage";
}

function to837(c: Claim) {
  const lines = c.lines.map((l, i) => `LX*${i + 1}~\nSV1*HC:${l.cpt}${l.modifiers.length ? ":" + l.modifiers.join(":") : ""}*${l.charge.toFixed(2)}*UN*${l.units}***${l.icd10.map((_, k) => k + 1).join(":")}~\nDTP*472*D8*${c.dos.slice(6)}${c.dos.slice(0, 2)}${c.dos.slice(3, 5)}~`);
  const dx = [...new Set(c.lines.flatMap((l) => l.icd10))].map((d, i) => `${i === 0 ? "ABK" : "ABF"}:${d.replace(".", "")}`).join("*");
  return [
    "ST*837*0001*005010X222A1~",
    `CLM*${c.id}*${total(c).toFixed(2)}***11:B:1*Y*A*Y*Y~`,
    c.authNumber ? `REF*G1*${c.authNumber}~` : "",
    c.wcbCaseNumber ? `REF*Y4*${c.wcbCaseNumber}~` : "",
    `HI*${dx}~`,
    ...lines,
    "SE*…*0001~",
  ].filter(Boolean).join("\n");
}

export default function ClaimsPage() {
  const [claims, setClaims] = useState<Claim[]>(CLAIMS);
  const [sel, setSel] = useState(CLAIMS[0].id);
  const [view, setView] = useState<"edits" | "x12">("edits");
  const [fixedLog, setFixedLog] = useState<string[]>([]);

  const scored = useMemo(() => claims.map((c) => ({ c, ...denialRisk(c) })), [claims]);
  const cur = scored.find((s) => s.c.id === sel)!;
  const atRisk = scored.filter((s) => s.risk >= 25);
  const revenueAtRisk = atRisk.reduce((s, x) => s + total(x.c), 0);
  const clean = scored.filter((s) => s.findings.length === 0).length;

  const applyFix = (id: string, idx: number) => {
    const c = claims.find((x) => x.id === id);
    const f = c && denialRisk(c).findings[idx];
    if (!c || !f?.apply) return;
    setClaims((cs) => cs.map((x) => (x.id === id ? f.apply!(x) : x)));
    setFixedLog((l) => [`${id}: ${f.rule}`, ...l]);
  };
  const fixClaim = (c: Claim, log: string[]) => {
    let x = c;
    for (let guard = 0; guard < 8; guard++) {
      const f = denialRisk(x).findings.find((y) => y.apply);
      if (!f) break;
      log.push(`${c.id}: ${f.rule}`);
      x = f.apply!(x);
    }
    return x;
  };
  const fixAll = (id: string) => {
    const log: string[] = [];
    const next = claims.map((c) => (c.id === id ? fixClaim(c, log) : c));
    setClaims(next);
    setFixedLog((l) => [...log, ...l]);
  };
  const fixEverything = () => {
    const log: string[] = [];
    const next = claims.map((c) => fixClaim(c, log));
    setClaims(next);
    setFixedLog((l) => [...log, ...l]);
  };

  return (
    <div className="mx-auto max-w-7xl">
      <ToolHeader
        eyebrow="Revenue cycle · Billing & claims workflows"
        title="Claim Scrubber & Denial Predictor"
        description="Every claim is checked before it leaves: NCCI-style bundling, modifiers, retired ICD-10 codes, missing authorizations, and New York workers' comp and no-fault rules. Fixes come pre-filled and the risk score updates live."
        tags={["X12 837P / 835", "NCCI edits", "ICD-10-CM / CPT", "denial risk scoring", "NY no-fault · WCB"]}
        impact={[{ value: "97.1%", label: "clean claim rate" }, { value: "−42%", label: "denials" }]}
      />

      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Stat label="Claims in today's batch" value={claims.length} hint="Ready to transmit to the clearinghouse" tone="neutral" />
        <Stat label="Passing all edits" value={`${clean}/${claims.length}`} delta={clean === claims.length ? "ready" : undefined} />
        <Stat label="At denial risk" value={atRisk.length} tone="danger" delta={atRisk.length ? money(revenueAtRisk) : "none"} hint="risk ≥ 25%" />
        <Stat label="Edits auto-fixed (session)" value={fixedLog.length} hint="each one logged with who approved it" />
      </div>

      <div className="mt-6 grid gap-6 xl:grid-cols-[1.1fr_1fr]">
        <Card className="overflow-hidden">
          <CardHeader title="Pre-submission batch" subtitle="Click a claim to see its edits" right={<Button size="sm" variant="dark" onClick={fixEverything} disabled={atRisk.length === 0 && scored.every((s) => !s.findings.length)}><Wand2 className="size-3.5" /> Fix all safe edits</Button>} />
          <div className="overflow-x-auto">
            <table className="w-full min-w-[620px] text-[13px]">
              <thead className="bg-cream text-left text-[11px] uppercase tracking-wide text-faint">
                <tr><th className="px-5 py-2.5">Claim</th><th className="px-2 py-2.5">Payer</th><th className="px-2 py-2.5">Codes</th><th className="px-2 py-2.5 text-right">Charge</th><th className="px-5 py-2.5 text-right">Denial risk</th></tr>
              </thead>
              <tbody className="divide-y divide-line">
                {scored.map(({ c, risk, findings }) => (
                  <tr key={c.id} onClick={() => setSel(c.id)} className={cx("cursor-pointer transition", sel === c.id ? "bg-sky-2" : "hover:bg-cream")}>
                    <td className="px-5 py-3">
                      <div className="font-semibold text-ink">{c.patient}</div>
                      <div className="font-mono text-[11px] text-faint">{c.id} · DOS {c.dos}</div>
                    </td>
                    <td className="px-2 py-3"><div className="text-ink">{c.payer}</div><div className="text-[11px] text-muted">{c.payerType}</div></td>
                    <td className="px-2 py-3 font-mono text-[11.5px] text-muted">{c.lines.map((l) => l.cpt + (l.modifiers.length ? `-${l.modifiers.join("-")}` : "")).join(", ")}</td>
                    <td className="px-2 py-3 text-right tabular-nums text-ink">{money(total(c))}</td>
                    <td className="px-5 py-3 text-right">
                      <div className="flex items-center justify-end gap-2">
                        {findings.length > 0 && <span className="text-[11px] text-muted">{findings.length} edit{findings.length > 1 ? "s" : ""}</span>}
                        <Badge tone={riskTone(risk)}>{risk}%</Badge>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>

        <Card className="flex flex-col">
          <CardHeader
            title={`${cur.c.patient} · ${cur.c.id}`}
            subtitle={`${cur.c.payer} · ${cur.c.provider} · ${money(total(cur.c))}`}
            right={<Segmented value={view} onChange={setView} options={[{ value: "edits", label: "Edits" }, { value: "x12", label: "837P" }]} />}
          />
          {view === "x12" ? (
            <div className="p-4"><CodeBlock value={to837(cur.c)} maxH="max-h-[420px]" /><p className="mt-2 text-[11px] text-muted">Simplified 837P claim loop. Production adds envelope, billing/rendering provider loops and payer IDs.</p></div>
          ) : (
            <div className="flex-1">
              <div className="flex items-center justify-between border-b border-line px-5 py-4">
                <div>
                  <div className="text-[12px] text-muted">Predicted denial risk</div>
                  <div className={cx("font-display text-4xl font-semibold tabular-nums", cur.risk >= 50 ? "text-danger" : cur.risk >= 25 ? "text-warn" : "text-sage")}>{cur.risk}%</div>
                </div>
                {cur.findings.some((f) => f.apply) && <Button variant="sage" onClick={() => fixAll(cur.c.id)}><Wand2 className="size-4" /> Apply all fixes</Button>}
              </div>
              {cur.findings.length === 0 ? (
                <div className="animate-fade-up flex items-center gap-3 p-6 text-[14px] text-sage"><ShieldCheck className="size-6" /> All edits pass. Claim is ready to transmit.</div>
              ) : (
                <ul className="divide-y divide-line">
                  {cur.findings.map((f, i) => (
                    <li key={f.rule + i} className="px-5 py-4">
                      <div className="flex items-start gap-3">
                        {f.severity === "critical" ? <XOctagon className="mt-0.5 size-4 shrink-0 text-danger" /> : <AlertTriangle className="mt-0.5 size-4 shrink-0 text-warn" />}
                        <div className="min-w-0 flex-1">
                          <div className="flex flex-wrap items-center gap-2"><span className="text-[13px] font-semibold text-ink">{f.rule}</span><Badge tone={f.severity === "critical" ? "danger" : "warn"}>{f.severity}</Badge><span className="text-[11px] text-faint">+{f.weight} risk</span></div>
                          <p className="mt-1 text-[12.5px] text-muted">{f.message}</p>
                          <div className="mt-2 flex flex-wrap items-center justify-between gap-2 rounded-lg bg-sand px-3 py-2">
                            <span className="text-[12px] text-ink"><b>Fix:</b> {f.fix}</span>
                            {f.apply && <Button size="sm" variant="secondary" onClick={() => applyFix(cur.c.id, i)}>Apply</Button>}
                          </div>
                        </div>
                      </div>
                    </li>
                  ))}
                </ul>
              )}
              <div className="border-t border-line px-5 py-4">
                <div className="text-[12px] font-semibold text-muted">Claim lines</div>
                <ul className="mt-2 space-y-1 font-mono text-[12px] text-ink">
                  {cur.c.lines.map((l, i) => (
                    <li key={i} className="flex justify-between gap-3"><span>{l.cpt}{l.modifiers.length ? `-${l.modifiers.join("-")}` : ""} × {l.units} · {l.icd10.join(", ")}</span><span className="text-muted">{money(l.charge)}</span></li>
                  ))}
                </ul>
                {cur.c.authNumber && <div className="mt-2 text-[12px] text-sage"><CheckCircle2 className="mr-1 inline size-3.5" />Auth {cur.c.authNumber}</div>}
              </div>
            </div>
          )}
        </Card>
      </div>

      <div className="mt-6 grid gap-6 lg:grid-cols-[1fr_1fr]">
        <Card>
          <CardHeader title="Top denial reasons, last 90 days" subtitle="Share of denied claims, from 835 remittance CARC codes" />
          <div className="p-5">
            <Bars
              label="Top denial reasons last 90 days"
              unit="%"
              data={[
                { label: "Auth missing", value: 27 },
                { label: "Bundling", value: 19 },
                { label: "Modifier", value: 15 },
                { label: "Eligibility", value: 13 },
                { label: "Dx/medical nec.", value: 11 },
                { label: "Timely filing", value: 8 },
                { label: "Other", value: 7 },
              ]}
            />
          </div>
        </Card>
        <Card>
          <CardHeader title="How the predictor works" />
          <ul className="space-y-3 p-5 text-[13px] text-muted">
            <li><b className="text-ink">1 · Deterministic edits</b> catch known errors with certainty: bundling pairs, modifier rules, retired codes, payer filing limits.</li>
            <li><b className="text-ink">2 · Learned risk</b> weighs each edit using historical 835 remittances by payer, CPT and provider (gradient-boosted model in production; weighted rules here).</li>
            <li><b className="text-ink">3 · Feedback loop:</b> every denial that still happens is labeled and fed back weekly, so the scrubber learns each payer&apos;s habits.</li>
            <li><b className="text-ink">4 · Human control:</b> coding changes that affect medical necessity always need coder approval; formatting fixes apply automatically.</li>
          </ul>
        </Card>
      </div>
    </div>
  );
}
