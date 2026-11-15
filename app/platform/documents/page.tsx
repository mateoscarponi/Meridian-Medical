"use client";

import { useMemo, useRef, useState } from "react";
import { ArrowRight, CheckCircle2, FileScan, Inbox, Loader2, Printer, ScanLine, Smartphone, Upload, Wand2 } from "lucide-react";
import { ToolHeader } from "@/components/platform/shell";
import { Badge, Button, Card, CardHeader, CodeBlock, Segmented, cx } from "@/components/ui";

interface Field { label: string; value: string; conf: number }
interface Doc {
  id: string;
  source: "Fax" | "Portal upload" | "Email" | "Mobile photo";
  from: string;
  received: string;
  type: string;
  typeConf: number;
  pages: number;
  lines: string[];
  fields: Field[];
  route: { queue: string; actions: string[] };
  fhir: Record<string, unknown>;
}

const DOCS: Doc[] = [
  {
    id: "DOC-5521", source: "Fax", from: "(516) 555-0172 · Patel Family Medicine", received: "8:02 AM", type: "Referral", typeConf: 0.98, pages: 2,
    lines: ["PATEL FAMILY MEDICINE — REFERRAL FORM", "Date: 11/13/2026        Urgency: Routine", "Patient: Carmen Diaz      DOB: 08/15/1972", "Insurance: Empire BCBS   Member ID: YRP451207733", "Referred to: Orthopedic Spine — Mateo Orthopedic & Spine", "Reason: Right sciatica x 5 weeks, failed NSAIDs", "Dx: M54.31 Sciatica, right side", "Referring: Rina Patel, MD   NPI 1888888801", "Please evaluate and treat. Notes attached."],
    fields: [
      { label: "Patient", value: "Carmen Diaz", conf: 0.99 }, { label: "DOB", value: "08/15/1972", conf: 0.99 }, { label: "Insurance", value: "Empire BCBS", conf: 0.97 },
      { label: "Member ID", value: "YRP451207733", conf: 0.95 }, { label: "Reason", value: "Right sciatica x 5 weeks, failed NSAIDs", conf: 0.93 }, { label: "Dx", value: "M54.31", conf: 0.98 }, { label: "Referring", value: "Rina Patel, MD", conf: 0.97 },
    ],
    route: { queue: "Referrals → Scheduling", actions: ["Matched existing patient MOS-205518 (score 0.97)", "Eligibility verified (271: active, specialist copay $40)", "Referral created; offered 3 slots by SMS", "Loop-closure task set for referring office"] },
    fhir: { resourceType: "ServiceRequest", status: "active", intent: "order", category: [{ text: "Referral" }], code: { text: "Orthopedic spine evaluation" }, reasonCode: [{ coding: [{ system: "http://hl7.org/fhir/sid/icd-10-cm", code: "M54.31" }] }], subject: { reference: "Patient/pt-1007" }, requester: { display: "Rina Patel, MD", identifier: { system: "http://hl7.org/fhir/sid/us-npi", value: "1888888801" } } },
  },
  {
    id: "DOC-5522", source: "Fax", from: "Nassau Imaging Center", received: "8:17 AM", type: "Imaging report (MRI)", typeConf: 0.99, pages: 3,
    lines: ["NASSAU IMAGING CENTER — MRI REPORT", "Patient: James O'Connor   DOB: 09/30/1975   MRN: MOS-198344", "Exam: MRI LUMBAR SPINE WITHOUT CONTRAST (CPT 72148)", "Date of exam: 11/15/2026", "IMPRESSION:", "1. Right paracentral disc extrusion at L4-L5 contacting", "   the descending right L5 nerve root.", "2. Moderate right lateral recess stenosis at L4-L5.", "Radiologist: K. Iyer, MD"],
    fields: [
      { label: "Patient", value: "James O'Connor", conf: 0.99 }, { label: "MRN", value: "MOS-198344", conf: 0.99 }, { label: "Exam", value: "MRI LUMBAR SPINE WITHOUT CONTRAST", conf: 0.98 },
      { label: "Date of exam", value: "11/15/2026", conf: 0.99 }, { label: "Impression", value: "Right paracentral disc extrusion at L4-L5", conf: 0.94 }, { label: "Radiologist", value: "K. Iyer, MD", conf: 0.96 },
    ],
    route: { queue: "Clinical inbox → Dr. Okafor", actions: ["Filed to chart as DiagnosticReport", "Attached to pending prior auth PA-24812 (criterion: imaging concordance ✓)", "Provider notified: result ready for review"] },
    fhir: { resourceType: "DiagnosticReport", status: "final", code: { coding: [{ system: "http://www.ama-assn.org/go/cpt", code: "72148" }] }, subject: { reference: "Patient/pt-1002" }, effectiveDateTime: "2026-11-15", conclusion: "Right paracentral disc extrusion at L4-L5 contacting the descending right L5 nerve root." },
  },
  {
    id: "DOC-5523", source: "Mobile photo", from: "Patient upload · Aisha Rahman", received: "8:40 AM", type: "Insurance card", typeConf: 0.97, pages: 1,
    lines: ["GEICO — PERSONAL INJURY PROTECTION", "Claim #: NF-77120934", "Insured: Aisha Rahman", "Date of loss: 09/20/2026", "Adjuster: M. Torres  (800) 555-0110", "Policy: 4410-22-889"],
    fields: [{ label: "Payer", value: "GEICO", conf: 0.98 }, { label: "Claim #", value: "NF-77120934", conf: 0.97 }, { label: "Insured", value: "Aisha Rahman", conf: 0.99 }, { label: "Date of loss", value: "09/20/2026", conf: 0.96 }, { label: "Adjuster", value: "M. Torres", conf: 0.92 }],
    route: { queue: "Billing → No-fault", actions: ["Coverage updated (No-Fault, claim # added)", "NF-3 billing deadline set: 45 days from each DOS", "Adjuster contact saved to the case"] },
    fhir: { resourceType: "Coverage", status: "active", type: { text: "No-Fault (PIP)" }, subscriberId: "NF-77120934", beneficiary: { reference: "Patient/pt-1005" }, payor: [{ display: "GEICO" }] },
  },
  {
    id: "DOC-5524", source: "Portal upload", from: "Payer portal · Humana", received: "9:05 AM", type: "Authorization decision", typeConf: 0.96, pages: 1,
    lines: ["HUMANA — PRIOR AUTHORIZATION DETERMINATION", "Member: Linda Chen   ID: H5619-0042", "Service: 64483 Transforaminal ESI, lumbar", "Decision: APPROVED", "Auth #: HUM-AUTH-8841207", "Valid: 11/16/2026 – 02/14/2027   Units: 1"],
    fields: [{ label: "Member", value: "Linda Chen", conf: 0.99 }, { label: "Service", value: "64483", conf: 0.99 }, { label: "Decision", value: "APPROVED", conf: 0.99 }, { label: "Auth #", value: "HUM-AUTH-8841207", conf: 0.98 }, { label: "Valid", value: "11/16/2026 – 02/14/2027", conf: 0.97 }],
    route: { queue: "Authorizations → Scheduling", actions: ["Auth # written to the order and future claim (REF*G1)", "Procedure slot offered to patient", "Prior-auth case closed: 2.4 days end to end"] },
    fhir: { resourceType: "ClaimResponse", status: "active", use: "preauthorization", outcome: "complete", preAuthRef: "HUM-AUTH-8841207", preAuthPeriod: { start: "2026-11-16", end: "2027-02-14" }, patient: { reference: "Patient/pt-1003" } },
  },
  {
    id: "DOC-5525", source: "Email", from: "Law office · records request", received: "9:31 AM", type: "Records request (legal)", typeConf: 0.91, pages: 2,
    lines: ["RE: Request for medical records — HIPAA authorization enclosed", "Patient: Robert Williams   DOB: 06/02/1981", "Records requested: 08/01/2026 to present", "Purpose: Workers' compensation hearing", "Signed authorization: YES (page 2)"],
    fields: [{ label: "Patient", value: "Robert Williams", conf: 0.99 }, { label: "Date range", value: "08/01/2026 to present", conf: 0.95 }, { label: "Purpose", value: "Workers' compensation hearing", conf: 0.93 }, { label: "Authorization", value: "YES", conf: 0.9 }],
    route: { queue: "Release of information → Human review", actions: ["Authorization validity check: signature & expiration found", "Minimum-necessary packet assembled (WC encounters only)", "Held for staff approval before release (policy)"] },
    fhir: { resourceType: "Task", status: "requested", intent: "order", code: { text: "Release of information" }, for: { reference: "Patient/pt-1004" }, businessStatus: { text: "Awaiting staff approval" } },
  },
];

const SOURCE_ICON = { Fax: Printer, "Portal upload": Upload, Email: Inbox, "Mobile photo": Smartphone };

export default function DocumentsPage() {
  const [sel, setSel] = useState(DOCS[0].id);
  const [status, setStatus] = useState<Record<string, "queued" | "reading" | "done">>(() => Object.fromEntries(DOCS.map((d) => [d.id, "queued"])));
  const [view, setView] = useState<"fields" | "fhir">("fields");
  const busy = useRef(false);
  const d = DOCS.find((x) => x.id === sel)!;
  const st = status[d.id];
  const processedCount = Object.values(status).filter((s) => s === "done").length;

  const processOne = (id: string) =>
    new Promise<void>((res) => {
      setStatus((s) => ({ ...s, [id]: "reading" }));
      setTimeout(() => {
        setStatus((s) => ({ ...s, [id]: "done" }));
        res();
      }, 1500);
    });

  const processAll = async () => {
    if (busy.current) return;
    busy.current = true;
    for (const doc of DOCS) {
      if (status[doc.id] === "done") continue;
      setSel(doc.id);
      await processOne(doc.id);
    }
    busy.current = false;
  };

  const highlightLine = (line: string) => {
    if (st !== "done") return line;
    const hit = d.fields.find((f) => line.includes(f.value));
    if (!hit) return line;
    const i = line.indexOf(hit.value);
    return (
      <>
        {line.slice(0, i)}
        <span className="rounded-[3px] bg-ocean/10 outline outline-[1.5px] outline-ocean/70" title={`${hit.label} · ${Math.round(hit.conf * 100)}%`}>{hit.value}</span>
        {line.slice(i + hit.value.length)}
      </>
    );
  };

  const typesSummary = useMemo(() => DOCS.reduce<Record<string, number>>((m, x) => ((m[x.source] = (m[x.source] ?? 0) + 1), m), {}), []);

  return (
    <div className="mx-auto max-w-7xl">
      <ToolHeader
        eyebrow="Clinical AI · OCR / Document AI"
        title="Document AI Inbox"
        description="Every fax, upload and email gets read, classified, matched to the right patient, turned into structured data and routed to the right queue. Staff review exceptions instead of retyping documents."
        tags={["OCR", "document classification", "key-value extraction", "patient matching", "FHIR write-back"]}
        impact={[{ value: "−44 h", label: "intake hours / week" }, { value: "96%", label: "straight-through rate" }]}
      />

      <div className="grid gap-6 xl:grid-cols-[320px_1fr]">
        <Card className="h-fit overflow-hidden">
          <CardHeader
            title="Incoming today"
            subtitle={`${processedCount} of ${DOCS.length} processed · ${Object.entries(typesSummary).map(([k, v]) => `${v} ${k.toLowerCase()}`).join(", ")}`}
            right={<Button size="sm" onClick={processAll} disabled={processedCount === DOCS.length}><Wand2 className="size-3.5" /> Process all</Button>}
          />
          <ul className="divide-y divide-line">
            {DOCS.map((x) => {
              const Icon = SOURCE_ICON[x.source];
              const s = status[x.id];
              return (
                <li key={x.id}>
                  <button onClick={() => setSel(x.id)} className={cx("flex w-full items-start gap-3 px-4 py-3.5 text-left transition", sel === x.id ? "bg-sky-2 shadow-[inset_3px_0_0_var(--color-ocean)]" : "hover:bg-cream")}>
                    <div className="mt-0.5 grid size-8 shrink-0 place-items-center rounded-lg bg-sand text-muted"><Icon className="size-4" /></div>
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center justify-between gap-2">
                        <span className="truncate text-[13px] font-semibold text-ink">{s === "done" ? x.type : "Unclassified document"}</span>
                        {s === "queued" && <Badge>Queued</Badge>}
                        {s === "reading" && <Badge tone="ocean"><Loader2 className="size-3 animate-spin" /> Reading</Badge>}
                        {s === "done" && <Badge tone="sage">Routed</Badge>}
                      </div>
                      <div className="truncate text-[12px] text-muted">{x.from}</div>
                      <div className="text-[11px] text-faint">{x.source} · {x.pages} pg · {x.received}</div>
                    </div>
                  </button>
                </li>
              );
            })}
          </ul>
        </Card>

        <div className="grid gap-6 lg:grid-cols-[1fr_1fr]">
          <Card className="overflow-hidden">
            <CardHeader title={d.id} subtitle={`${d.source} from ${d.from}`} icon={<FileScan className="size-4" />} right={st === "queued" && <Button size="sm" onClick={() => processOne(d.id)}><ScanLine className="size-3.5" /> Read</Button>} />
            <div className="bg-sand/70 p-5">
              <div className="grain relative mx-auto max-w-md -rotate-[0.6deg] overflow-hidden rounded-sm bg-[#fdfcf8] p-6 font-mono text-[11.5px] leading-6 text-[#2a2f36] shadow-[0_10px_30px_-12px_rgb(15_30_46/0.35)]">
                {st === "reading" && <div className="pointer-events-none absolute inset-x-0 h-10 animate-scan bg-gradient-to-b from-transparent via-ocean/25 to-transparent" />}
                <div className="mb-3 flex justify-between text-[9px] uppercase tracking-widest text-[#8a8f96]"><span>{d.source === "Fax" ? `FAX · ${d.received}` : d.source}</span><span>Page 1/{d.pages}</span></div>
                {d.lines.map((l, i) => (
                  <div key={i} className={cx("whitespace-pre-wrap", i === 0 && "mb-2 font-bold")}>{highlightLine(l)}</div>
                ))}
              </div>
            </div>
          </Card>

          <Card className="flex flex-col">
            <CardHeader
              title={st === "done" ? d.type : "Awaiting processing"}
              subtitle={st === "done" ? `Classified with ${Math.round(d.typeConf * 100)}% confidence` : "Run Read or Process all"}
              right={st === "done" && <Segmented value={view} onChange={setView} options={[{ value: "fields", label: "Fields" }, { value: "fhir", label: "FHIR" }]} />}
            />
            {st !== "done" ? (
              <div className="grid flex-1 place-items-center p-10 text-center text-[13px] text-muted">
                {st === "reading" ? <div><Loader2 className="mx-auto size-7 animate-spin text-ocean" /><p className="mt-3">OCR → layout analysis → classification → extraction…</p></div> : <div><ScanLine className="mx-auto size-8 text-line-2" /><p className="mt-3">Not processed yet.</p></div>}
              </div>
            ) : view === "fhir" ? (
              <div className="animate-fade-up p-4"><CodeBlock value={d.fhir} maxH="max-h-[420px]" /></div>
            ) : (
              <div className="animate-fade-up flex-1">
                <table className="w-full text-[13px]">
                  <tbody className="divide-y divide-line">
                    {d.fields.map((f) => (
                      <tr key={f.label}>
                        <td className="w-32 px-5 py-2.5 font-semibold text-muted">{f.label}</td>
                        <td className="px-2 py-2.5 text-ink">{f.value}</td>
                        <td className="px-5 py-2.5 text-right"><span className={cx("font-mono text-[11px]", f.conf >= 0.95 ? "text-sage" : "text-warn")}>{Math.round(f.conf * 100)}%</span></td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                <div className="border-t border-line p-5">
                  <div className="flex items-center gap-2 text-[13px] font-semibold text-ink"><ArrowRight className="size-4 text-ocean" /> Routed to {d.route.queue}</div>
                  <ul className="mt-3 space-y-2">
                    {d.route.actions.map((a) => <li key={a} className="flex gap-2 text-[12.5px] text-muted"><CheckCircle2 className="mt-0.5 size-3.5 shrink-0 text-sage" />{a}</li>)}
                  </ul>
                </div>
              </div>
            )}
          </Card>
        </div>
      </div>
    </div>
  );
}
