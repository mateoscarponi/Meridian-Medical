"use client";

import { useState } from "react";
import { AlertOctagon, Bot, CheckCircle2, Languages, Loader2, Megaphone, MessageSquare, Mic, Phone, Send, Smartphone, Sparkles } from "lucide-react";
import { ToolHeader } from "@/components/platform/shell";
import { Badge, Button, Card, CardHeader, cx, type Tone } from "@/components/ui";

type Intent = "Urgent: clinical" | "Appointment" | "Prior auth status" | "Billing" | "Records" | "Refill";

interface Msg { id: number; from: string; channel: "SMS" | "Portal" | "Voicemail"; time: string; lang: "EN" | "ES"; text: string; intent: Intent; conf: number; draft: string; route: string }

const MSGS: Msg[] = [
  {
    id: 1, from: "Maria Gonzalez", channel: "SMS", time: "7:52 AM", lang: "ES",
    text: "Hola, necesito cambiar mi cita del martes. ¿Tienen algo el jueves en Garden City? Gracias.",
    intent: "Appointment", conf: 0.97, route: "Front desk · Garden City",
    draft: "Hola Maria, con gusto. Tenemos el jueves 19 de noviembre a las 2:00 PM o 3:15 PM en Garden City. Responda 1 o 2 para confirmar y cancelamos su cita del martes. — Equipo de Meridian Orthopedics & Spine Care",
  },
  {
    id: 2, from: "Thomas Russo", channel: "Portal", time: "8:05 AM", lang: "EN",
    text: "I had my neck surgery 9 days ago. Since last night I have a fever of 101.8 and the incision is red and draining. Is that normal?",
    intent: "Urgent: clinical", conf: 0.99, route: "On-call surgeon · callback < 15 min",
    draft: "Mr. Russo, thank you for letting us know. Fever with redness and drainage after surgery needs to be checked today. Our on-call surgeon has been notified and will call you within 15 minutes. If you have trouble breathing or swallowing, or feel very unwell, call 911 or go to the nearest emergency room now. — Meridian Orthopedics & Spine Care Team",
  },
  {
    id: 3, from: "Robert Williams", channel: "Voicemail", time: "8:21 AM", lang: "EN",
    text: "(transcribed) Hi, this is Robert Williams, workers' comp patient. Just checking if my neck injection was approved yet. Thanks.",
    intent: "Prior auth status", conf: 0.95, route: "Auto-answered from auth tracker",
    draft: "Hi Robert, good news: your neck injection falls within the New York workers' comp treatment guidelines, so no separate approval is needed. You're scheduled for Tuesday, Nov 17 at 1:00 PM in Garden City. Please bring your WCB case number. — Meridian Orthopedics & Spine Care Team",
  },
  {
    id: 4, from: "Kevin Murphy", channel: "SMS", time: "8:40 AM", lang: "EN",
    text: "Got a bill for $150 but I thought my insurance covered the visit?",
    intent: "Billing", conf: 0.93, route: "Billing team",
    draft: "Hi Kevin, thanks for reaching out. We're reviewing your 11/14 visit with Oxford now and will text you within one business day with an explanation of benefits breakdown. No payment is due while we review. — Meridian Orthopedics & Spine Care Team",
  },
  {
    id: 5, from: "Carmen Diaz", channel: "Portal", time: "9:02 AM", lang: "ES",
    text: "¿Puedo subir los discos de mi resonancia antes de la cita? Me los dieron en otro centro.",
    intent: "Records", conf: 0.94, route: "Auto-answered · imaging upload link",
    draft: "Hola Carmen, ¡sí! Use este enlace seguro para subir sus imágenes antes de la cita: [enlace seguro]. También puede traer el disco ese día. Así el doctor podrá revisarlas con usted. — Equipo de Meridian Orthopedics & Spine Care",
  },
];

const INTENT_TONE: Record<Intent, Tone> = { "Urgent: clinical": "danger", Appointment: "ocean", "Prior auth status": "gold", Billing: "neutral", Records: "sage", Refill: "neutral" };
const CHANNEL_ICON = { SMS: Smartphone, Portal: MessageSquare, Voicemail: Mic };

export default function MessagesPage() {
  const [sel, setSel] = useState(2);
  const [sent, setSent] = useState<number[]>([]);
  const [drafts, setDrafts] = useState<Record<number, string>>(() => Object.fromEntries(MSGS.map((m) => [m.id, m.draft])));
  const [engine, setEngine] = useState<Record<number, string>>({});
  const [loading, setLoading] = useState(false);
  const m = MSGS.find((x) => x.id === sel)!;

  const regenerate = async () => {
    setLoading(true);
    try {
      const r = await fetch("/api/ai", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ task: "reply", input: `Patient ${m.from} via ${m.channel}: ${m.text}` }) });
      const j = await r.json();
      if (j.engine === "claude" && j.text) {
        setDrafts((d) => ({ ...d, [m.id]: j.text }));
        setEngine((e) => ({ ...e, [m.id]: "Claude" }));
      } else {
        setEngine((e) => ({ ...e, [m.id]: "Template library (no API key configured)" }));
      }
    } catch {
      setEngine((e) => ({ ...e, [m.id]: "Template library (offline)" }));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="mx-auto max-w-7xl">
      <ToolHeader
        eyebrow="Front office · Patient communication systems"
        title="Patient Messages"
        description="One inbox for SMS, portal and voicemail. AI classifies intent, catches emergencies first, drafts replies in the patient's language, and answers routine questions straight from live data. Staff approve before anything is sent."
        tags={["intent classification", "red-flag escalation", "EN / ES", "voicemail transcription", "LLM drafts + approval"]}
        impact={[{ value: "−23 h", label: "phone & inbox / week" }, { value: "4 min", label: "median first response" }]}
      />

      <div className="grid gap-6 xl:grid-cols-[380px_1fr]">
        <Card className="h-fit overflow-hidden">
          <CardHeader title="Inbox" subtitle="Sorted by urgency, then time" />
          <ul className="divide-y divide-line">
            {[...MSGS].sort((a, b) => (a.intent === "Urgent: clinical" ? -1 : b.intent === "Urgent: clinical" ? 1 : 0)).map((x) => {
              const Icon = CHANNEL_ICON[x.channel];
              return (
                <li key={x.id}>
                  <button onClick={() => setSel(x.id)} className={cx("w-full px-4 py-3.5 text-left transition", sel === x.id ? "bg-sky-2 shadow-[inset_3px_0_0_var(--color-ocean)]" : "hover:bg-cream", x.intent === "Urgent: clinical" && sel !== x.id && "bg-danger-soft/40")}>
                    <div className="flex items-center justify-between gap-2">
                      <span className="flex items-center gap-1.5 text-[13px] font-semibold text-ink"><Icon className="size-3.5 text-muted" />{x.from}</span>
                      <span className="text-[11px] text-faint">{x.time}</span>
                    </div>
                    <p className="mt-1 line-clamp-2 text-[12.5px] text-muted">{x.text}</p>
                    <div className="mt-2 flex flex-wrap items-center gap-1.5">
                      <Badge tone={INTENT_TONE[x.intent]}>{x.intent === "Urgent: clinical" && <AlertOctagon className="size-3" />}{x.intent}</Badge>
                      <Badge>{x.lang}</Badge>
                      {sent.includes(x.id) && <Badge tone="sage"><CheckCircle2 className="size-3" /> Replied</Badge>}
                    </div>
                  </button>
                </li>
              );
            })}
          </ul>
        </Card>

        <div className="space-y-6">
          {m.intent === "Urgent: clinical" && (
            <div className="animate-fade-up flex items-start gap-3 rounded-2xl border border-danger/30 bg-danger-soft p-4 text-danger" role="alert">
              <Phone className="mt-0.5 size-5 shrink-0" />
              <div className="text-[13px]"><b className="text-[14px]">Escalated automatically at {m.time}.</b> Possible post-operative infection. On-call surgeon paged; callback required within 15 minutes; the message was pulled out of the routine queue.</div>
            </div>
          )}
          <Card>
            <CardHeader title={m.from} subtitle={`${m.channel} · ${m.time} · ${m.lang === "ES" ? "Spanish" : "English"}`} right={<Badge tone={INTENT_TONE[m.intent]}>{m.intent} · {Math.round(m.conf * 100)}%</Badge>} />
            <div className="p-5">
              <div className="max-w-xl rounded-2xl rounded-tl-sm bg-sand px-4 py-3 text-[14px] text-ink">{m.text}</div>
              <div className="mt-2 text-[12px] text-muted">Routed to: <b className="text-ink">{m.route}</b></div>
            </div>
          </Card>
          <Card>
            <CardHeader
              title="Suggested reply"
              subtitle={engine[m.id] ? `Drafted by: ${engine[m.id]}` : "Drafted from the template library + live schedule / auth data"}
              icon={<Sparkles className="size-4" />}
              right={<Button size="sm" variant="secondary" onClick={regenerate} disabled={loading}>{loading ? <Loader2 className="size-3.5 animate-spin" /> : <Bot className="size-3.5" />} Regenerate with AI</Button>}
            />
            <textarea value={drafts[m.id]} onChange={(e) => setDrafts((d) => ({ ...d, [m.id]: e.target.value }))} className="h-36 w-full resize-none p-5 text-[14px] leading-relaxed text-ink outline-none" />
            <div className="flex flex-wrap items-center justify-between gap-3 border-t border-line p-4">
              <span className="inline-flex items-center gap-1.5 text-[12px] text-muted"><Languages className="size-3.5" /> Reply language matches the patient ({m.lang})</span>
              <Button variant="sage" onClick={() => setSent((s) => [...s, m.id])} disabled={sent.includes(m.id)}>
                {sent.includes(m.id) ? <><CheckCircle2 className="size-4" /> Sent</> : <><Send className="size-4" /> Approve &amp; send</>}
              </Button>
            </div>
          </Card>

          <Card>
            <CardHeader title="Outbound campaigns" subtitle="Proactive messages that prevent inbound calls" icon={<Megaphone className="size-4" />} />
            <div className="grid gap-px bg-line sm:grid-cols-3">
              {[
                ["Pre-visit checklist", "72 h before visit: forms link, bring MRI discs, ID & referral", "−31% day-of reschedules"],
                ["Imaging upload nudge", "New patients with outside imaging get a secure upload link", "64% upload before visit"],
                ["Post-procedure check-in", "Day 1 & day 7 after injections or surgery, red-flag questions", "Early issues caught in 9 cases"],
              ].map(([t, d, k]) => (
                <div key={t} className="bg-white p-4">
                  <div className="text-[13px] font-semibold text-ink">{t}</div>
                  <div className="mt-1 text-[12px] text-muted">{d}</div>
                  <div className="mt-2 text-[12px] font-semibold text-sage">{k}</div>
                </div>
              ))}
            </div>
          </Card>
        </div>
      </div>
    </div>
  );
}
