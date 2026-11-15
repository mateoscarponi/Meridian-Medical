"use client";

import { useMemo, useState } from "react";
import { Bot, CheckCircle2, ClipboardCheck, FileText, Loader2, Mic, PenLine, Send, Sparkles, Stethoscope } from "lucide-react";
import { ToolHeader } from "@/components/platform/shell";
import { Badge, Button, Card, CardHeader, Segmented, cx } from "@/components/ui";
import { conservativeCareWeeks, extractEntities, redFlags, suggestIcd10 } from "@/lib/nlp";

const TRANSCRIPT = `DR: Good morning, Mr. O'Connor. What brings you back today?
PT: The pain in my lower back is still going down my right leg. It's been about 10 weeks now since I hurt it lifting boxes at work.
DR: Any numbness or tingling?
PT: Yes, numbness and tingling on the top of my right foot. And my big toe feels weak.
DR: How did physical therapy go?
PT: I did 8 weeks of physical therapy and the home exercises. I also took naproxen for 6 weeks. It helps a little but the leg pain comes back.
DR: Any trouble controlling your bladder or bowels, or numbness in the groin?
PT: No, nothing like that. No fever either.
DR: On exam, straight leg raise is positive on the right at 40 degrees. Strength is 4 out of 5 in right great toe extension. Reflexes are symmetric. Sensation is decreased in the right L5 dermatome.
DR: Your MRI shows a disc extrusion at L4-L5 pressing on the right L5 nerve root, which matches your symptoms.
DR: Plan: I'd like to do a right L4-L5 transforaminal epidural steroid injection. Continue the home exercise program, and we'll see you 2 weeks after the injection. If you're not better, we'll talk about a surgical consult.
PT: Okay, let's do the injection.`;

function localSoap(t: string) {
  const ents = extractEntities(t);
  const pt = t.split("\n").filter((l) => l.startsWith("PT:")).map((l) => l.slice(3).trim()).join(" ");
  const dr = t.split("\n").filter((l) => l.startsWith("DR:")).map((l) => l.slice(3).trim());
  const cc = conservativeCareWeeks(pt);
  const rf = redFlags(extractEntities(pt));
  const icd = suggestIcd10(t, ents);
  const dur = ents.find((e) => e.type === "duration")?.text ?? "not discussed";
  const symptoms = [...new Set(extractEntities(pt).filter((e) => e.type === "symptom" && !e.negated).map((e) => e.text.toLowerCase()))];
  const exam = dr.filter((l) => /straight leg|strength|reflex|sensation|exam/i.test(l)).join(" ").replace(/^On exam,\s*/i, "");
  const imaging = dr.find((l) => /MRI|X-?ray|CT/i.test(l)) ?? "Not discussed.";
  const plan = dr.find((l) => /^Plan/i.test(l))?.replace(/^Plan:\s*/i, "") ?? "Not discussed.";
  const procs = [...new Set(extractEntities(plan).filter((e) => e.type === "procedure").map((e) => e.text))];
  const denied = [...rf.negated.map((r) => r.text), ...extractEntities(pt).filter((e) => e.negated && e.type !== "redflag").map((e) => e.text)];
  return `SUBJECTIVE
Low back pain radiating to the right leg for ${dur}, onset after lifting at work. Associated ${symptoms.filter((s) => !/back pain/.test(s)).join(", ") || "symptoms not specified"}. Conservative care: ${cc.weeks} weeks documented (${cc.evidence.join(" ").replace(/^I /, "") || "not discussed"}). Denies ${denied.join(", ") || "red-flag symptoms"}.

OBJECTIVE
${exam || "Exam not dictated."}
Imaging: ${imaging}

ASSESSMENT
${icd.map((i) => `${i.code} – ${i.display}`).join("\n") || "No codable assessment found."}
Radicular symptoms concordant with imaging; failed ${cc.weeks} weeks of conservative care.

PLAN
${plan}
Orders: ${procs.map((p) => `${p} (CPT 64483, prior authorization required for commercial plans)`).join("; ") || "none"}.
Follow-up 2 weeks post-procedure. Return precautions reviewed.`;
}

export default function ScribePage() {
  const [transcript, setTranscript] = useState(TRANSCRIPT);
  const [note, setNote] = useState("");
  const [engine, setEngine] = useState<"claude" | "local" | null>(null);
  const [engineNote, setEngineNote] = useState("");
  const [loading, setLoading] = useState(false);
  const [signed, setSigned] = useState(false);
  const [lang, setLang] = useState<"en" | "es">("en");

  const icd = useMemo(() => suggestIcd10(transcript, extractEntities(transcript)), [transcript]);

  const generate = async () => {
    setLoading(true);
    setSigned(false);
    try {
      const r = await fetch("/api/ai", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ task: "scribe", input: transcript }) });
      const j = await r.json();
      await new Promise((res) => setTimeout(res, 900));
      if (j.engine === "claude" && j.text) {
        setNote(j.text);
        setEngine("claude");
        setEngineNote(`${j.model ?? "Claude"} · ${j.phiRedacted ?? 0} identifiers redacted before the call`);
      } else {
        setNote(localSoap(transcript));
        setEngine("local");
        setEngineNote(j.note ?? "On-device clinical NLP (no API key configured)");
      }
    } catch {
      setNote(localSoap(transcript));
      setEngine("local");
      setEngineNote("Offline: on-device clinical NLP");
    } finally {
      setLoading(false);
    }
  };

  const instructions = {
    en: "You are scheduled for a right L4-L5 epidural steroid injection. Arrange a driver for the day of the procedure. Keep doing your home exercises. Call us or go to the ER right away if you have trouble controlling your bladder or bowels, numbness in the groin, or new weakness in your legs.",
    es: "Tiene programada una inyección epidural de esteroides en L4-L5 del lado derecho. Organice que alguien lo lleve el día del procedimiento. Continúe con sus ejercicios en casa. Llámenos o vaya a la sala de emergencias de inmediato si tiene problemas para controlar la vejiga o los intestinos, adormecimiento en la ingle o debilidad nueva en las piernas.",
  };

  return (
    <div className="mx-auto max-w-7xl">
      <ToolHeader
        eyebrow="Clinical AI · Clinical documentation assistance"
        title="Clinical Scribe"
        description="Ambient visit transcript → structured SOAP note with ICD-10 suggestions, E/M support, orders that kick off prior authorization automatically, and bilingual after-visit instructions. The provider reviews and signs; nothing files without them."
        tags={["LLM (Claude) with on-device fallback", "speech-to-text ready", "ICD-10 / CPT", "EN / ES", "human-in-the-loop"]}
        impact={[{ value: "−6 min", label: "documentation per visit" }, { value: "Same day", label: "chart closure" }]}
      />

      <div className="grid gap-6 xl:grid-cols-2">
        <Card className="flex flex-col">
          <CardHeader
            title="Visit transcript"
            subtitle="Follow-up · James O'Connor · Dr. Okafor · 11/16/2026"
            icon={<Mic className="size-4" />}
            right={<Badge tone="danger"><span className="size-1.5 animate-pulse-soft rounded-full bg-danger" /> Recorded with consent</Badge>}
          />
          <textarea value={transcript} onChange={(e) => setTranscript(e.target.value)} className="scroll-thin min-h-[420px] w-full flex-1 resize-none bg-cream/40 p-5 font-mono text-[12.5px] leading-relaxed text-ink outline-none" />
          <div className="flex items-center justify-between gap-3 border-t border-line p-4">
            <div className="text-[12px] text-muted">Speaker-labeled transcript (DR / PT). Editable for the demo.</div>
            <Button onClick={generate} disabled={loading}>
              {loading ? <Loader2 className="size-4 animate-spin" /> : <Sparkles className="size-4" />} {loading ? "Drafting note…" : "Generate SOAP note"}
            </Button>
          </div>
        </Card>

        <Card className="flex flex-col">
          <CardHeader
            title="Draft note"
            subtitle={engine ? (engine === "claude" ? `Generated with Claude ${engineNote ? `(${engineNote})` : ""}` : engineNote) : "Generate to draft the note"}
            icon={<FileText className="size-4" />}
            right={engine && <Badge tone={engine === "claude" ? "ocean" : "sage"}>{engine === "claude" ? <Bot className="size-3" /> : <Stethoscope className="size-3" />}{engine === "claude" ? "LLM" : "On-device"}</Badge>}
          />
          {note ? (
            <textarea value={note} onChange={(e) => setNote(e.target.value)} className="scroll-thin min-h-[420px] w-full flex-1 resize-none p-5 font-mono text-[12.5px] leading-relaxed text-ink outline-none" />
          ) : (
            <div className="grid min-h-[420px] flex-1 place-items-center p-8 text-center text-[13px] text-muted">
              <div>
                <PenLine className="mx-auto size-8 text-line-2" />
                <p className="mt-3 max-w-xs">The note appears here for review. Edits are tracked, and signing files it to the EMR as a FHIR DocumentReference.</p>
              </div>
            </div>
          )}
          <div className="flex items-center justify-between gap-3 border-t border-line p-4">
            <div className="text-[12px] text-muted">{signed ? "Signed by Dr. Okafor · filed to EMR · 11/16/2026 10:42" : "Requires provider signature"}</div>
            <Button variant={signed ? "secondary" : "sage"} disabled={!note || signed} onClick={() => setSigned(true)}>
              {signed ? <CheckCircle2 className="size-4" /> : <Send className="size-4" />} {signed ? "Signed" : "Sign & file"}
            </Button>
          </div>
        </Card>
      </div>

      {note && (
        <div className="animate-fade-up mt-6 grid gap-6 lg:grid-cols-3">
          <Card>
            <CardHeader title="Coding support" subtitle="Coder confirms before claim creation" />
            <div className="space-y-3 p-5 text-[13px]">
              {icd.map((i) => (
                <div key={i.code} className="flex items-start gap-2"><span className="font-mono font-semibold text-ocean">{i.code}</span><span className="text-ink">{i.display}</span></div>
              ))}
              <div className="rounded-lg bg-sand p-3">
                <div className="font-semibold text-ink">E/M suggestion: 99214</div>
                <div className="mt-0.5 text-[12px] text-muted">Moderate MDM: chronic illness with progression, decision for an elective procedure, prescription drug management. Add modifier 25 if billed with the injection on the same day.</div>
              </div>
            </div>
          </Card>
          <Card>
            <CardHeader title="Actions triggered on signing" subtitle="Automations, each logged to the audit trail" />
            <ul className="space-y-3 p-5 text-[13px]">
              {[
                ["Order 64483 created", "→ Prior Auth Copilot case PA-24812 opened with evidence pre-attached"],
                ["Procedure scheduling", "→ Westbury suite, next available after expected auth decision"],
                ["After-visit summary", "→ sent to patient portal in their preferred language"],
                ["Quality measures", "→ pain score & functional status captured for MIPS reporting"],
              ].map(([a, b]) => (
                <li key={a} className={cx("flex gap-2", !signed && "opacity-60")}>
                  {signed ? <CheckCircle2 className="mt-0.5 size-4 shrink-0 text-sage" /> : <ClipboardCheck className="mt-0.5 size-4 shrink-0 text-faint" />}
                  <span><b className="text-ink">{a}</b> <span className="text-muted">{b}</span></span>
                </li>
              ))}
            </ul>
          </Card>
          <Card>
            <CardHeader title="Patient instructions" subtitle="Plain language, 6th-grade reading level" right={<Segmented value={lang} onChange={setLang} options={[{ value: "en", label: "EN" }, { value: "es", label: "ES" }]} />} />
            <p className="p-5 text-[13px] leading-relaxed text-ink">{instructions[lang]}</p>
          </Card>
        </div>
      )}
    </div>
  );
}
