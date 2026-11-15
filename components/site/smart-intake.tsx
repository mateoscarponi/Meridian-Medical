"use client";

import { useMemo, useState } from "react";
import { useRouter } from "next/navigation";
import { AlertTriangle, ArrowLeft, ArrowRight, CalendarCheck2, CheckCircle2, Lock, ShieldCheck, Sparkles } from "lucide-react";
import { cx } from "../ui";

type Area = "neck" | "back" | "leg" | "joint" | "injury";
type Dur = "lt2" | "2to6" | "6to12" | "gt12";
type Ins = "commercial" | "medicare" | "wc" | "nofault" | "medicaid" | "self";

const AREAS: { v: Area; label: string }[] = [
  { v: "neck", label: "Neck / arm pain" },
  { v: "back", label: "Lower back pain" },
  { v: "leg", label: "Sciatica / leg pain" },
  { v: "joint", label: "Joint pain" },
  { v: "injury", label: "New injury" },
];
const DURS: { v: Dur; label: string }[] = [
  { v: "lt2", label: "< 2 weeks" },
  { v: "2to6", label: "2–6 weeks" },
  { v: "6to12", label: "6–12 weeks" },
  { v: "gt12", label: "3+ months" },
];
const INS: { v: Ins; label: string }[] = [
  { v: "commercial", label: "Commercial" },
  { v: "medicare", label: "Medicare" },
  { v: "wc", label: "Workers' Comp" },
  { v: "nofault", label: "No-Fault (auto)" },
  { v: "medicaid", label: "Medicaid" },
  { v: "self", label: "Self-pay" },
];
const FLAGS = ["Loss of bladder or bowel control", "Numbness in the groin / inner thighs", "Rapidly worsening leg or arm weakness", "Fever with severe back pain"];

function Chip({ active, onClick, children }: { active: boolean; onClick: () => void; children: React.ReactNode }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cx(
        "rounded-xl px-3 py-2 text-[13px] font-semibold ring-1 ring-inset transition-all duration-200",
        active ? "bg-navy text-white ring-navy shadow-sm" : "bg-white text-ink ring-line-2 hover:ring-ocean hover:text-ocean",
      )}
    >
      {children}
    </button>
  );
}

export function SmartIntake() {
  const [step, setStep] = useState(0);
  const [area, setArea] = useState<Area | null>(null);
  const [dur, setDur] = useState<Dur | null>(null);
  const [flags, setFlags] = useState<string[]>([]);
  const [ins, setIns] = useState<Ins | null>(null);
  const router = useRouter();

  const rec = useMemo(() => {
    if (!area || !dur || !ins) return null;
    let service = "Spine evaluation";
    let who = "Daniel Park, PA-C";
    let where = "Westbury";
    let when = "Tue, Nov 17 · 10:45 AM";
    if (area === "injury" || (dur === "lt2" && area === "joint")) {
      service = "Orthopedic urgent care";
      who = "Walk-in team";
      where = "East Meadow";
      when = "Today · walk in, avg. wait 25 min";
    } else if (area === "joint") {
      service = "Orthopedic consultation";
      who = "Dr. Elena Ruiz, MD";
      when = "Wed, Nov 18 · 9:15 AM";
    } else if (dur === "gt12" || dur === "6to12") {
      service = area === "neck" ? "Spine surgery consultation (cervical)" : "Spine & pain management consultation";
      who = dur === "gt12" ? "Dr. Adrian Hale, DO" : "Dr. Samuel Okafor, MD";
      where = dur === "gt12" ? "Lindenhurst" : "Westbury";
      when = dur === "gt12" ? "Thu, Nov 19 · 1:00 PM" : "Wed, Nov 18 · 2:30 PM";
    }
    const bring = ["Photo ID & insurance card", "Any MRI / X-ray discs and reports", "Medication list"];
    if (ins === "wc") bring.push("WCB case # and carrier claim #", "Date of injury & employer info");
    if (ins === "nofault") bring.push("No-fault claim # and NF-2 application", "Date of accident & police report #");
    if (ins === "commercial") bring.push("PCP referral if your plan requires one");
    if (ins === "medicaid") bring.push("Note: Medicaid is accepted for established patients & hospital follow-ups");
    return { service, who, where, when, bring };
  }, [area, dur, ins]);

  const urgent = flags.length > 0;
  const canNext = (step === 0 && area) || (step === 1 && dur) || step === 2 || (step === 3 && ins);

  return (
    <div id="request" className="relative scroll-mt-32 rounded-3xl border border-line bg-white/90 p-5 shadow-[var(--shadow-lift)] backdrop-blur sm:p-6">
      <div className="flex items-center justify-between">
        <div>
          <div className="flex items-center gap-2 text-[12px] font-semibold uppercase tracking-[0.14em] text-sage">
            <Sparkles className="size-3.5" /> Smart appointment request
          </div>
          <h2 className="mt-1 font-display text-xl font-semibold text-ink">Find the right visit in under a minute</h2>
        </div>
        {step < 4 && <span className="text-[12px] font-semibold text-faint">Step {Math.min(step + 1, 4)} / 4</span>}
      </div>

      <div className="mt-4 h-1 overflow-hidden rounded-full bg-sand">
        <div className="h-full rounded-full bg-sage transition-[width] duration-500" style={{ width: `${(Math.min(step, 4) / 4) * 100}%` }} />
      </div>
      <p className="mt-3 flex items-start gap-1.5 text-[11.5px] leading-snug text-muted">
        <ShieldCheck className="mt-px size-3.5 shrink-0 text-sage" />
        <span>Private by design: your answers stay in this browser. We never ask for your name, contact details or insurance ID on this page; those go only into our encrypted patient portal.</span>
      </p>

      {step < 4 ? (
        <div key={step} className="animate-fade-up">
          {step === 0 && (
            <fieldset className="mt-5">
              <legend className="mb-3 text-[14px] font-semibold text-ink">What brings you in?</legend>
              <div className="flex flex-wrap gap-2">
                {AREAS.map((a) => <Chip key={a.v} active={area === a.v} onClick={() => setArea(a.v)}>{a.label}</Chip>)}
              </div>
            </fieldset>
          )}
          {step === 1 && (
            <fieldset className="mt-5">
              <legend className="mb-3 text-[14px] font-semibold text-ink">How long has it been going on?</legend>
              <div className="flex flex-wrap gap-2">
                {DURS.map((d) => <Chip key={d.v} active={dur === d.v} onClick={() => setDur(d.v)}>{d.label}</Chip>)}
              </div>
            </fieldset>
          )}
          {step === 2 && (
            <fieldset className="mt-5">
              <legend className="mb-3 text-[14px] font-semibold text-ink">Any of these right now? <span className="font-normal text-muted">(select all that apply)</span></legend>
              <div className="grid gap-2 sm:grid-cols-2">
                {FLAGS.map((f) => (
                  <label key={f} className={cx("flex cursor-pointer items-center gap-2 rounded-xl px-3 py-2 text-[13px] ring-1 ring-inset transition", flags.includes(f) ? "bg-danger-soft text-danger ring-danger/30" : "bg-white ring-line-2 hover:ring-ocean")}>
                    <input type="checkbox" className="accent-[#b4483c]" checked={flags.includes(f)} onChange={(e) => setFlags((x) => (e.target.checked ? [...x, f] : x.filter((y) => y !== f)))} />
                    {f}
                  </label>
                ))}
              </div>
              {urgent && (
                <div className="mt-3 flex gap-2 rounded-xl bg-danger-soft p-3 text-[13px] text-danger" role="alert">
                  <AlertTriangle className="mt-0.5 size-4 shrink-0" />
                  <span><b>These symptoms can be an emergency.</b> Please go to the nearest emergency room or call 911 now. Our on-call team is also notified.</span>
                </div>
              )}
            </fieldset>
          )}
          {step === 3 && (
            <fieldset className="mt-5">
              <legend className="mb-3 text-[14px] font-semibold text-ink">How will this visit be covered?</legend>
              <div className="flex flex-wrap gap-2">
                {INS.map((i) => <Chip key={i.v} active={ins === i.v} onClick={() => setIns(i.v)}>{i.label}</Chip>)}
              </div>
            </fieldset>
          )}
          <div className="mt-6 flex items-center justify-between">
            <button type="button" onClick={() => setStep((s) => Math.max(0, s - 1))} className={cx("inline-flex items-center gap-1 text-[13px] font-semibold text-muted hover:text-ink", step === 0 && "invisible")}>
              <ArrowLeft className="size-4" /> Back
            </button>
            <button
              type="button"
              disabled={!canNext || (step === 2 && urgent)}
              onClick={() => setStep((s) => s + 1)}
              className="inline-flex items-center gap-1.5 rounded-xl bg-navy px-4 py-2.5 text-[14px] font-semibold text-white transition hover:bg-ocean disabled:opacity-40"
            >
              {step === 3 ? "See my options" : "Continue"} <ArrowRight className="size-4" />
            </button>
          </div>
        </div>
      ) : (
        rec && (
          <div className="animate-fade-up mt-5">
            <div className="rounded-2xl bg-gradient-to-br from-sky-2 to-sage-soft p-4">
              <div className="text-[12px] font-semibold uppercase tracking-wide text-ocean">Recommended</div>
              <div className="mt-1 font-display text-lg font-semibold text-ink">{rec.service}</div>
              <div className="mt-1 text-[14px] text-muted">{rec.who} · {rec.where}</div>
              <div className="mt-3 inline-flex items-center gap-2 rounded-lg bg-white px-3 py-1.5 text-[13px] font-semibold text-ink shadow-sm">
                <CalendarCheck2 className="size-4 text-sage" /> Earliest: {rec.when}
              </div>
            </div>
            <div className="mt-4">
              <div className="text-[13px] font-semibold text-ink">Bring to your visit</div>
              <ul className="mt-2 grid gap-1.5 text-[13px] text-muted">
                {rec.bring.map((b) => (
                  <li key={b} className="flex gap-2"><CheckCircle2 className="mt-0.5 size-4 shrink-0 text-sage" /> {b}</li>
                ))}
              </ul>
            </div>
            <div className="mt-5 flex items-center justify-between">
              <button type="button" onClick={() => setStep(3)} className="inline-flex items-center gap-1 text-[13px] font-semibold text-muted hover:text-ink">
                <ArrowLeft className="size-4" /> Back
              </button>
              <button
                type="button"
                onClick={() => {
                  // Hand off to the isolated portal through sessionStorage, never the URL (no health data in query strings or logs).
                  try { sessionStorage.setItem("mos-intake", JSON.stringify(rec)); } catch {}
                  router.push("/portal");
                }}
                className="inline-flex items-center gap-1.5 rounded-xl bg-sage px-4 py-2.5 text-[14px] font-semibold text-white transition hover:bg-[#25724f]"
              >
                <Lock className="size-4" /> Continue in secure portal
              </button>
            </div>
          </div>
        )
      )}
    </div>
  );
}
