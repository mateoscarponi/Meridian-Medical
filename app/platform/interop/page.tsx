"use client";

import { useMemo, useState } from "react";
import { ArrowRight, CheckCircle2, ChevronRight, Loader2, Network, Play, Server, Terminal } from "lucide-react";
import { ToolHeader } from "@/components/platform/shell";
import { Badge, Button, Card, CardHeader, CodeBlock, Segmented, cx } from "@/components/ui";
import { SAMPLE_MESSAGES, hl7ToFhir, parseHl7 } from "@/lib/hl7";

const ENDPOINTS = [
  { label: "CapabilityStatement", url: "/api/fhir/metadata" },
  { label: "Search patients by name", url: "/api/fhir/Patient?name=chen" },
  { label: "Read patient by id", url: "/api/fhir/Patient/pt-1004" },
  { label: "Coverage for a patient", url: "/api/fhir/Coverage?patient=pt-1002" },
  { label: "Tomorrow's appointments", url: "/api/fhir/Appointment?date=2026-11-17" },
  { label: "Orders (ServiceRequest)", url: "/api/fhir/ServiceRequest" },
  { label: "Unsupported type → OperationOutcome", url: "/api/fhir/Medication" },
];

const CONNECTORS = [
  { emr: "eClinicalWorks", method: "FHIR R4 (US Core) + HL7 v2 interfaces", feeds: "ADT, SIU, ORU, DFT", status: "Live" },
  { emr: "athenahealth", method: "athenaOne REST API + FHIR R4", feeds: "Scheduling, clinical docs, claims", status: "Live" },
  { emr: "Epic (hospital partners)", method: "FHIR R4 · SMART Backend Services", feeds: "Discharge summaries, imaging", status: "Live" },
  { emr: "Oracle Health (Cerner)", method: "FHIR R4 · Care Everywhere-style HIE query", feeds: "Records retrieval", status: "Pilot" },
  { emr: "NextGen", method: "FHIR R4 + HL7 v2", feeds: "Referrals in/out", status: "Pilot" },
  { emr: "AdvancedMD / DrChrono", method: "Vendor REST APIs (OAuth 2.0)", feeds: "PT & partner practice data", status: "Planned" },
];

const X12 = [
  ["270 / 271", "Eligibility & benefits inquiry / response"],
  ["278", "Prior authorization request & response"],
  ["837P", "Professional claim"],
  ["276 / 277", "Claim status inquiry / response"],
  ["835", "Electronic remittance advice (payments & denials)"],
];

export default function InteropPage() {
  const [sampleId, setSampleId] = useState(SAMPLE_MESSAGES[0].id);
  const [text, setText] = useState(SAMPLE_MESSAGES[0].text);
  const [openSeg, setOpenSeg] = useState<number | null>(1);
  const [out, setOut] = useState<"fhir" | "ack">("fhir");
  const [server, setServer] = useState<{ ms: number; body: unknown; status: number } | null>(null);
  const [sending, setSending] = useState(false);
  const [ep, setEp] = useState(ENDPOINTS[1].url);
  const [epRes, setEpRes] = useState<{ status: number; ms: number; body: unknown } | null>(null);
  const [epLoading, setEpLoading] = useState(false);

  const parsed = useMemo(() => parseHl7(text), [text]);
  const fhir = useMemo(() => (parsed.segments.length ? hl7ToFhir(parsed) : null), [parsed]);
  const resourceCount = ((fhir?.entry as unknown[]) ?? []).length;

  const send = async () => {
    setSending(true);
    const t0 = performance.now();
    const r = await fetch("/api/hl7", { method: "POST", headers: { "Content-Type": "text/plain" }, body: text });
    const body = await r.json();
    setServer({ ms: Math.round(performance.now() - t0), body, status: r.status });
    setSending(false);
  };

  const call = async (url: string) => {
    setEp(url);
    setEpLoading(true);
    const t0 = performance.now();
    const r = await fetch(url);
    const body = await r.json();
    setEpRes({ status: r.status, ms: Math.round(performance.now() - t0), body });
    setEpLoading(false);
  };

  return (
    <div className="mx-auto max-w-7xl">
      <ToolHeader
        eyebrow="Data & integration · Healthcare interoperability"
        title="HL7 / FHIR Interoperability Lab"
        description="A working integration layer: paste any HL7 v2 message to parse it field by field, map it to FHIR R4 resources and get the ACK. Then query this app's live FHIR API. The same code powers the agent, the scheduler and the dashboards."
        tags={["HL7 v2.5.1", "FHIR R4 · US Core", "SMART on FHIR", "X12 5010", "MLLP / REST"]}
        impact={[{ value: `${parsed.segments.length}`, label: "segments parsed" }, { value: `${resourceCount}`, label: "FHIR resources" }]}
      />

      <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
        <Segmented value={sampleId} onChange={(v) => { setSampleId(v); setText(SAMPLE_MESSAGES.find((s) => s.id === v)!.text); setServer(null); setOpenSeg(1); }} options={SAMPLE_MESSAGES.map((s) => ({ value: s.id, label: s.label.split(" — ")[0] }))} />
        <p className="text-[12.5px] text-muted">{SAMPLE_MESSAGES.find((s) => s.id === sampleId)?.description}</p>
      </div>

      <div className="grid gap-6 xl:grid-cols-3">
        <Card className="flex flex-col">
          <CardHeader title="1 · Raw HL7 v2" subtitle={parsed.type ? `${parsed.type} · control ID ${parsed.controlId} · v${parsed.version}` : "Paste a message"} icon={<Terminal className="size-4" />} />
          <textarea value={text} onChange={(e) => { setText(e.target.value); setServer(null); }} spellCheck={false} className="scroll-thin min-h-[360px] w-full flex-1 resize-none bg-ink p-4 font-mono text-[11.5px] leading-relaxed whitespace-pre text-[#cfe1ee] outline-none" />
          <div className="flex items-center justify-between gap-2 border-t border-line p-3">
            {parsed.errors.length ? <Badge tone="danger">{parsed.errors[0]}</Badge> : <Badge tone="sage"><CheckCircle2 className="size-3" /> Valid structure</Badge>}
            <Button size="sm" onClick={send} disabled={sending || !parsed.segments.length}>{sending ? <Loader2 className="size-3.5 animate-spin" /> : <Play className="size-3.5" />} POST /api/hl7</Button>
          </div>
        </Card>

        <Card className="flex flex-col">
          <CardHeader title="2 · Parsed segments" subtitle="Click a segment to see named fields" icon={<Network className="size-4" />} />
          <div className="scroll-thin max-h-[440px] flex-1 overflow-y-auto">
            {parsed.segments.map((s, i) => (
              <div key={i} className="border-b border-line last:border-0">
                <button onClick={() => setOpenSeg(openSeg === i ? null : i)} className="flex w-full items-center gap-2 px-4 py-2.5 text-left hover:bg-cream">
                  <ChevronRight className={cx("size-3.5 text-faint transition", openSeg === i && "rotate-90")} />
                  <span className="rounded bg-navy px-1.5 py-0.5 font-mono text-[11px] font-semibold text-white">{s.id}</span>
                  <span className="text-[12.5px] text-ink">{s.label}</span>
                  <span className="ml-auto text-[11px] text-faint">{s.fields.length} fields</span>
                </button>
                {openSeg === i && (
                  <table className="w-full bg-cream/60 text-[11.5px]">
                    <tbody>
                      {s.fields.map((f) => (
                        <tr key={f.index} className="border-t border-line/70">
                          <td className="w-16 px-4 py-1.5 font-mono text-faint">{s.id}-{f.index}</td>
                          <td className="w-40 py-1.5 pr-2 font-semibold text-muted">{f.name}</td>
                          <td className="py-1.5 pr-4 font-mono break-all text-ink">{f.components.map((rep) => rep.filter(Boolean).join(" · ")).join(" ~ ")}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )}
              </div>
            ))}
          </div>
        </Card>

        <Card className="flex flex-col">
          <CardHeader title="3 · Output" subtitle={server ? `Server: HTTP ${server.status} · ${server.ms} ms round trip` : "Mapped client-side; POST to run on the server"} right={<Segmented value={out} onChange={setOut} options={[{ value: "fhir", label: "FHIR Bundle" }, { value: "ack", label: "ACK" }]} />} />
          <div className="flex-1 p-3">
            {out === "fhir" ? (
              <CodeBlock value={server ? (server.body as { fhir: unknown }).fhir : fhir ?? {}} maxH="max-h-[400px]" />
            ) : (
              <CodeBlock value={server ? String((server.body as { ack: string }).ack).replace(/\r/g, "\n") : "POST the message to receive the HL7 ACK (MSA|AA = accepted)."} maxH="max-h-[400px]" />
            )}
          </div>
        </Card>
      </div>

      <div className="mt-6 grid gap-6 xl:grid-cols-[1fr_1.2fr]">
        <Card>
          <CardHeader title="Live FHIR API" subtitle="Real endpoints served by this app (synthetic data)" icon={<Server className="size-4" />} />
          <ul className="divide-y divide-line">
            {ENDPOINTS.map((e) => (
              <li key={e.url}>
                <button onClick={() => call(e.url)} className={cx("flex w-full items-center justify-between gap-3 px-5 py-2.5 text-left transition", ep === e.url && epRes ? "bg-sky-2" : "hover:bg-cream")}>
                  <span>
                    <span className="block text-[13px] font-semibold text-ink">{e.label}</span>
                    <span className="block font-mono text-[11.5px] text-ocean">GET {e.url}</span>
                  </span>
                  <ArrowRight className="size-4 text-faint" />
                </button>
              </li>
            ))}
          </ul>
        </Card>
        <Card className="flex flex-col">
          <CardHeader title="Response" subtitle={epRes ? `HTTP ${epRes.status} · ${epRes.ms} ms · application/fhir+json` : "Select an endpoint"} right={epLoading && <Loader2 className="size-4 animate-spin text-ocean" />} />
          <div className="flex-1 p-3"><CodeBlock value={epRes?.body ?? "// response appears here"} maxH="max-h-[420px]" /></div>
        </Card>
      </div>

      <div className="mt-6 grid gap-6 lg:grid-cols-[1.4fr_1fr]">
        <Card className="overflow-hidden">
          <CardHeader title="EMR / EHR connectors" subtitle="How each system connects to the platform" />
          <div className="overflow-x-auto">
            <table className="w-full min-w-[560px] text-[13px]">
              <thead className="bg-cream text-left text-[11px] uppercase tracking-wide text-faint"><tr><th className="px-5 py-2.5">System</th><th className="px-2 py-2.5">Method</th><th className="px-2 py-2.5">Data</th><th className="px-5 py-2.5 text-right">Status</th></tr></thead>
              <tbody className="divide-y divide-line">
                {CONNECTORS.map((c) => (
                  <tr key={c.emr}>
                    <td className="px-5 py-3 font-semibold text-ink">{c.emr}</td>
                    <td className="px-2 py-3 text-muted">{c.method}</td>
                    <td className="px-2 py-3 text-muted">{c.feeds}</td>
                    <td className="px-5 py-3 text-right"><Badge tone={c.status === "Live" ? "sage" : c.status === "Pilot" ? "ocean" : "neutral"}>{c.status}</Badge></td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
        <Card>
          <CardHeader title="Payer transactions (X12 5010)" subtitle="Through a clearinghouse API" />
          <ul className="divide-y divide-line">
            {X12.map(([k, v]) => <li key={k} className="flex items-center gap-4 px-5 py-3 text-[13px]"><span className="w-20 font-mono font-semibold text-ocean">{k}</span><span className="text-muted">{v}</span></li>)}
          </ul>
        </Card>
      </div>
    </div>
  );
}
