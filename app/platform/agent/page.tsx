"use client";

import { useEffect, useRef, useState } from "react";
import { Bot, Lock, Play, RotateCcw, ShieldCheck, SkipForward, User, Wrench, AlertOctagon } from "lucide-react";
import { ToolHeader } from "@/components/platform/shell";
import { Badge, Button, Card, CardHeader, Segmented, cx } from "@/components/ui";

type Step =
  | { kind: "user"; text: string }
  | { kind: "agent"; text: string; escalate?: boolean }
  | { kind: "tool"; name: string; args: Record<string, unknown>; fetchUrl?: string; result?: unknown; summarize?: (r: unknown) => unknown };

const pickBundle = (r: unknown, max = 2) => {
  const b = r as { total?: number; entry?: { resource: Record<string, unknown> }[] };
  return { total: b.total, entry: (b.entry ?? []).slice(0, max).map((e) => e.resource) };
};

const SCENARIOS: Record<string, { title: string; channel: string; steps: Step[] }> = {
  reschedule: {
    title: "Reschedule + outside MRI (Spanish)",
    channel: "SMS",
    steps: [
      { kind: "user", text: "Hola, soy Maria Gonzalez. Necesito mover mi cita del martes y tengo una resonancia de otro lugar." },
      { kind: "agent", text: "¡Hola Maria! Con gusto le ayudo. Para proteger su información, ¿me confirma su fecha de nacimiento?" },
      { kind: "user", text: "12 de abril de 1968" },
      { kind: "tool", name: "lookup_patient", args: { name: "Gonzalez", birthdate: "1968-04-12" }, fetchUrl: "/api/fhir/Patient?name=gonzalez", summarize: (r) => pickBundle(r, 1) },
      { kind: "tool", name: "get_appointments", args: { patient: "pt-1001", date: "2026-11-17" }, fetchUrl: "/api/fhir/Appointment?patient=pt-1001", summarize: (r) => pickBundle(r, 1) },
      { kind: "tool", name: "find_open_slots", args: { location: "Westbury", provider: "Dr. Okafor", from: "2026-11-18", visit: "follow-up", language: "es" }, result: { slots: ["2026-11-19T14:00", "2026-11-19T15:15", "2026-11-20T09:30"] } },
      { kind: "agent", text: "Gracias, Maria, ya la encontré. Su cita es el martes 17 a las 8:15 AM con el Dr. Okafor. Tengo disponible el jueves 19 a las 2:00 PM o 3:15 PM, o el viernes 20 a las 9:30 AM en Westbury. ¿Cuál prefiere?" },
      { kind: "user", text: "El jueves a las 2 está perfecto." },
      { kind: "tool", name: "reschedule_appointment", args: { appointment: "appt-3000", newStart: "2026-11-19T14:00:00-05:00" }, result: { status: "booked", id: "appt-3000", previous: "cancelled → slot released to waitlist", hl7: "SIU^S14 sent to EMR" } },
      { kind: "tool", name: "send_secure_link", args: { patient: "pt-1001", purpose: "imaging_upload", language: "es", expiresHours: 72 }, result: { sent: true, channel: "SMS" } },
      { kind: "agent", text: "¡Listo! Su cita quedó el jueves 19 de noviembre a las 2:00 PM en Westbury. Le envié un enlace seguro para subir su resonancia antes de la cita, así el doctor la revisa con usted. ¿Algo más en lo que le pueda ayudar?" },
    ],
  },
  auth: {
    title: "Workers' comp authorization status",
    channel: "Web chat",
    steps: [
      { kind: "user", text: "Hi, Robert Williams here. Is my neck injection approved? Workers' comp case." },
      { kind: "agent", text: "Hi Robert, happy to check. To verify it's you, can you share your date of birth?" },
      { kind: "user", text: "June 2, 1981" },
      { kind: "tool", name: "lookup_patient", args: { name: "Williams", birthdate: "1981-06-02" }, fetchUrl: "/api/fhir/Patient?name=williams", summarize: (r) => pickBundle(r, 1) },
      { kind: "tool", name: "get_coverage", args: { patient: "pt-1004" }, fetchUrl: "/api/fhir/Coverage?patient=pt-1004", summarize: (r) => pickBundle(r, 1) },
      { kind: "tool", name: "get_prior_auth_status", args: { order: "62321", patient: "pt-1004" }, result: { case: "PA-24816", status: "not_required", reason: "Within NY WCB Medical Treatment Guidelines", scheduled: "2026-11-17T13:00 Westbury" } },
      { kind: "agent", text: "Thanks, Robert. Good news: your cervical injection is within the New York workers' comp treatment guidelines, so it doesn't need a separate approval. You're scheduled for Tuesday, Nov 17 at 1:00 PM in Westbury. Please bring your WCB case number and arrange a ride home. Anything else?" },
    ],
  },
  redflag: {
    title: "Red flag → immediate human handoff",
    channel: "Web chat",
    steps: [
      { kind: "user", text: "My back pain got a lot worse since yesterday and now I can't really feel when I need to pee. Can I get an appointment this week?" },
      { kind: "tool", name: "safety_screen", args: { text: "…can't really feel when I need to pee…" }, result: { red_flags: ["possible urinary retention / loss of bladder sensation"], risk: "emergent", policy: "no scheduling; escalate" } },
      { kind: "tool", name: "handoff_to_human", args: { queue: "on-call clinician", priority: "emergent", transcript: true }, result: { paged: "Dr. Okafor (on call)", eta: "callback < 10 min" } },
      { kind: "agent", escalate: true, text: "I'm concerned about what you're describing. New changes in bladder feeling with worsening back pain can be a medical emergency. Please go to the nearest emergency room now or call 911. I've alerted our on-call doctor, who will call you within 10 minutes. Can you confirm the best number to reach you?" },
    ],
  },
};

const TOOLS = [
  { name: "lookup_patient", desc: "FHIR Patient search + identity verification (name + DOB)" },
  { name: "get_appointments / find_open_slots", desc: "FHIR Appointment + scheduling rules" },
  { name: "reschedule_appointment", desc: "Writes back to the EMR (HL7 SIU^S14)" },
  { name: "get_coverage / get_prior_auth_status", desc: "Coverage + Prior Auth Copilot" },
  { name: "send_secure_link", desc: "Forms, imaging upload, payments (expiring)" },
  { name: "safety_screen / handoff_to_human", desc: "Red-flag detection → on-call escalation" },
];

export default function AgentPage() {
  const [key, setKey] = useState<keyof typeof SCENARIOS>("reschedule");
  const [shown, setShown] = useState(0);
  const [results, setResults] = useState<Record<number, unknown>>({});
  const [auto, setAuto] = useState(false);
  const chatRef = useRef<HTMLDivElement>(null);
  const sc = SCENARIOS[key];

  const reset = () => { setShown(0); setResults({}); setAuto(false); };
  useEffect(reset, [key]);

  const next = async () => {
    if (shown >= sc.steps.length) return;
    const i = shown;
    const s = sc.steps[i];
    if (s.kind === "tool") {
      let r: unknown = s.result;
      if (s.fetchUrl) {
        try {
          const res = await fetch(s.fetchUrl);
          const j = await res.json();
          r = s.summarize ? s.summarize(j) : j;
        } catch {
          r = { error: "network" };
        }
      }
      setResults((x) => ({ ...x, [i]: r }));
    }
    setShown(i + 1);
  };

  useEffect(() => {
    if (!auto) return;
    if (shown >= sc.steps.length) { setAuto(false); return; }
    const t = setTimeout(next, sc.steps[shown].kind === "tool" ? 650 : 1100);
    return () => clearTimeout(t);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [auto, shown, key]);

  useEffect(() => { chatRef.current?.scrollTo({ top: chatRef.current.scrollHeight, behavior: "smooth" }); }, [shown]);

  const visible = sc.steps.slice(0, shown);

  return (
    <div className="mx-auto max-w-7xl">
      <ToolHeader
        eyebrow="Front office · AI agents"
        title="AI Front-Desk Agent"
        description="A tool-using agent that handles routine patient requests end to end, around the clock and in the patient's language. Its tools read and write real systems (the patient lookups below call this app's live FHIR API), and its guardrails escalate anything clinical to a human right away."
        tags={["LLM tool use", "FHIR R4 read/write", "identity verification", "guardrails", "human handoff"]}
        impact={[{ value: "62%", label: "requests resolved end-to-end" }, { value: "24/7", label: "EN / ES coverage" }]}
      />

      <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
        <Segmented value={key} onChange={setKey} options={[{ value: "reschedule", label: "Reschedule (ES)" }, { value: "auth", label: "Auth status (WC)" }, { value: "redflag", label: "Red flag" }]} />
        <div className="flex gap-2">
          <Button size="sm" variant="secondary" onClick={reset}><RotateCcw className="size-3.5" /> Reset</Button>
          <Button size="sm" variant="secondary" onClick={next} disabled={shown >= sc.steps.length || auto}><SkipForward className="size-3.5" /> Step</Button>
          <Button size="sm" onClick={() => setAuto(true)} disabled={shown >= sc.steps.length || auto}><Play className="size-3.5" /> Play conversation</Button>
        </div>
      </div>

      <div className="grid gap-6 xl:grid-cols-[1fr_1fr]">
        <Card className="flex h-[620px] flex-col overflow-hidden">
          <CardHeader title={sc.title} subtitle={`Channel: ${sc.channel}`} icon={<Bot className="size-4" />} right={<Badge tone="sage"><Lock className="size-3" /> Verified session</Badge>} />
          <div ref={chatRef} className="scroll-thin flex-1 space-y-3 overflow-y-auto bg-cream/50 p-5">
            {visible.length === 0 && <div className="grid h-full place-items-center text-[13px] text-muted">Press “Play conversation” or step through it.</div>}
            {visible.map((s, i) =>
              s.kind === "user" ? (
                <div key={i} className="animate-fade-up flex justify-end gap-2">
                  <div className="max-w-[80%] rounded-2xl rounded-br-sm bg-navy px-4 py-2.5 text-[13.5px] text-white">{s.text}</div>
                  <div className="grid size-7 shrink-0 place-items-center rounded-full bg-sand text-muted"><User className="size-3.5" /></div>
                </div>
              ) : s.kind === "agent" ? (
                <div key={i} className="animate-fade-up flex gap-2">
                  <div className={cx("grid size-7 shrink-0 place-items-center rounded-full text-white", s.escalate ? "bg-danger" : "bg-sage")}>{s.escalate ? <AlertOctagon className="size-3.5" /> : <Bot className="size-3.5" />}</div>
                  <div className={cx("max-w-[80%] rounded-2xl rounded-bl-sm px-4 py-2.5 text-[13.5px] shadow-sm", s.escalate ? "bg-danger-soft text-danger ring-1 ring-danger/20" : "bg-white text-ink ring-1 ring-line")}>{s.text}</div>
                </div>
              ) : (
                <div key={i} className="animate-fade-up ml-9 flex items-center gap-2 text-[11.5px] text-muted">
                  <Wrench className="size-3.5 text-gold" /> called <code className="rounded bg-white px-1.5 py-0.5 font-mono text-[11px] text-ink ring-1 ring-line">{s.name}</code>
                </div>
              ),
            )}
          </div>
        </Card>

        <Card className="flex h-[620px] flex-col overflow-hidden">
          <CardHeader title="Agent trace" subtitle="Every tool call, its arguments and the live result" icon={<Wrench className="size-4" />} />
          <div className="scroll-thin flex-1 space-y-3 overflow-y-auto bg-ink p-4 font-mono text-[11.5px] leading-relaxed text-[#cfe1ee]">
            {visible.filter((s) => s.kind === "tool").length === 0 && <div className="text-white/40">{"// tool calls will appear here"}</div>}
            {visible.map((s, i) =>
              s.kind === "tool" ? (
                <div key={i} className="animate-fade-up rounded-lg border border-white/10 bg-white/[0.03] p-3">
                  <div className="text-[#e6c88a]">→ {s.name}({JSON.stringify(s.args)})</div>
                  {s.fetchUrl && <div className="mt-1 text-white/40">GET {s.fetchUrl}</div>}
                  <pre className="mt-2 whitespace-pre-wrap text-[#9fd8b8]">← {JSON.stringify(results[i], null, 2)}</pre>
                </div>
              ) : null,
            )}
          </div>
        </Card>
      </div>

      <div className="mt-6 grid gap-6 lg:grid-cols-2">
        <Card>
          <CardHeader title="Tools available to the agent" />
          <ul className="divide-y divide-line">
            {TOOLS.map((t) => (
              <li key={t.name} className="flex items-start justify-between gap-4 px-5 py-3 text-[13px]"><code className="font-mono text-[12px] text-ocean">{t.name}</code><span className="text-right text-muted">{t.desc}</span></li>
            ))}
          </ul>
        </Card>
        <Card>
          <CardHeader title="Guardrails" icon={<ShieldCheck className="size-4" />} />
          <ul className="space-y-2.5 p-5 text-[13px] text-muted">
            <li>• <b className="text-ink">No clinical advice.</b> Symptoms are screened first; red flags skip scheduling and go straight to the on-call clinician.</li>
            <li>• <b className="text-ink">Identity before PHI:</b> name + DOB match, and write actions need a verified session.</li>
            <li>• <b className="text-ink">Minimum necessary:</b> tools return only the fields the task needs.</li>
            <li>• <b className="text-ink">Scoped writes:</b> the agent can book, reschedule and send links, never edit clinical data.</li>
            <li>• <b className="text-ink">Full audit trail</b> of every message and tool call, plus a human takeover button in the staff console.</li>
            <li>• <b className="text-ink">LLM under a BAA</b> with zero data retention, prompt-injection filtering on inbound text, and evals run before each release.</li>
          </ul>
        </Card>
      </div>
    </div>
  );
}
