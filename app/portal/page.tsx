"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import { ArrowLeft, CalendarCheck2, CheckCircle2, KeyRound, Lock, ShieldCheck, Timer } from "lucide-react";
import { Monogram, DemoPill } from "@/components/site/brand";
import { Button, Field, inputCls, cx } from "@/components/ui";
import { useIdle } from "@/components/use-idle";

interface Rec { service: string; who: string; where: string; when: string; bring: string[] }
const EMPTY = { name: "", dob: "", phone: "", email: "", carrier: "", memberId: "", language: "English" };

export default function PortalPage() {
  const [rec, setRec] = useState<Rec | null>(null);
  const [form, setForm] = useState(EMPTY);
  const [npp, setNpp] = useState(false);
  const [sms, setSms] = useState(false);
  const [done, setDone] = useState(false);
  const [timedOut, setTimedOut] = useState(false);
  const { idle, reset } = useIdle(15 * 60 * 1000);

  useEffect(() => {
    try {
      const raw = sessionStorage.getItem("mos-intake");
      if (raw) setRec(JSON.parse(raw));
    } catch {}
  }, []);

  // Automatic logoff: wipe everything typed after 15 minutes of inactivity.
  useEffect(() => {
    if (!idle) return;
    setForm(EMPTY);
    setNpp(false);
    setSms(false);
    setTimedOut(true);
    try { sessionStorage.removeItem("mos-intake"); } catch {}
    reset();
  }, [idle, reset]);

  const set = (k: keyof typeof EMPTY) => (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => setForm((f) => ({ ...f, [k]: e.target.value }));
  const valid = form.name.trim().length > 2 && form.dob && form.phone.replace(/\D/g, "").length >= 10 && npp;

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!valid) return;
    // Demo: nothing is transmitted or stored. Clear the data from memory and session storage.
    setForm(EMPTY);
    setNpp(false);
    setSms(false);
    try { sessionStorage.removeItem("mos-intake"); } catch {}
    setDone(true);
  };

  return (
    <div className="min-h-screen bg-sand/60">
      <header className="border-b border-line bg-white">
        <div className="mx-auto flex max-w-5xl items-center justify-between gap-4 px-4 py-3 sm:px-6">
          <Link href="/" className="flex items-center gap-2.5">
            <Monogram className="size-9 text-[15px]" />
            <span className="leading-tight">
              <span className="block font-display text-[15px] font-semibold text-ink">Meridian Orthopedics</span>
              <span className="flex items-center gap-1 text-[11px] font-semibold text-sage"><Lock className="size-3" /> Secure Patient Portal</span>
            </span>
          </Link>
          <div className="flex items-center gap-3">
            <span className="hidden font-mono text-[11px] text-muted sm:inline">portal.meridian-ortho.example</span>
            <DemoPill className="!text-gold" />
          </div>
        </div>
      </header>

      <main className="mx-auto grid max-w-5xl gap-6 px-4 py-10 sm:px-6 lg:grid-cols-[1.3fr_1fr]">
        <section className="rounded-3xl border border-line bg-white p-6 shadow-[var(--shadow-soft)] sm:p-8">
          {done ? (
            <div className="animate-fade-up py-8 text-center">
              <div className="mx-auto grid size-14 place-items-center rounded-full bg-sage-soft text-sage"><CheckCircle2 className="size-7" /></div>
              <h1 className="mt-4 font-display text-2xl font-semibold text-ink">Request received</h1>
              <p className="mx-auto mt-2 max-w-md text-[14px] text-muted">We&apos;ll text you to confirm the time and send your forms in your preferred language.</p>
              <div className="mx-auto mt-5 max-w-md rounded-xl bg-gold-soft p-4 text-left text-[12.5px] text-ink">
                <b>Demo:</b> nothing you typed was transmitted or stored, and the form has been cleared. In production this form posts over TLS to a HIPAA-compliant portal vendor under a Business Associate Agreement, and the data goes straight to the EMR, never into website logs, analytics or email.
              </div>
              <Link href="/" className="mt-6 inline-flex items-center gap-1.5 text-[13px] font-semibold text-ocean hover:underline"><ArrowLeft className="size-4" /> Back to the website</Link>
            </div>
          ) : (
            <form onSubmit={submit} autoComplete="off" noValidate>
              <h1 className="font-display text-2xl font-semibold text-ink">Confirm your appointment request</h1>
              <p className="mt-1 text-[14px] text-muted">This is the only place we ask for personal details. The connection is encrypted and the page is never cached.</p>
              {timedOut && (
                <div className="mt-4 flex items-start gap-2 rounded-xl bg-warn-soft p-3 text-[12.5px] text-warn" role="status">
                  <Timer className="mt-0.5 size-4 shrink-0" /> For your privacy, the form was cleared after 15 minutes of inactivity.
                </div>
              )}

              <div className="mt-6 grid gap-4 sm:grid-cols-2">
                <Field label="Full legal name"><input className={inputCls} value={form.name} onChange={set("name")} autoComplete="off" /></Field>
                <Field label="Date of birth"><input type="date" className={inputCls} value={form.dob} onChange={set("dob")} autoComplete="off" /></Field>
                <Field label="Mobile phone"><input type="tel" inputMode="tel" className={inputCls} value={form.phone} onChange={set("phone")} autoComplete="off" placeholder="(516) 555-0100" /></Field>
                <Field label="Email (optional)"><input type="email" className={inputCls} value={form.email} onChange={set("email")} autoComplete="off" /></Field>
                <Field label="Insurance carrier"><input className={inputCls} value={form.carrier} onChange={set("carrier")} autoComplete="off" /></Field>
                <Field label="Member ID" hint="Masked on screen; encrypted at rest"><input type="password" className={inputCls} value={form.memberId} onChange={set("memberId")} autoComplete="off" /></Field>
                <Field label="Preferred language">
                  <select className={inputCls} value={form.language} onChange={set("language")}>
                    <option>English</option><option>Español</option>
                  </select>
                </Field>
              </div>

              <div className="mt-6 space-y-3 rounded-2xl bg-cream p-4 text-[13px]">
                <label className="flex cursor-pointer items-start gap-2.5">
                  <input type="checkbox" className="mt-0.5 accent-[#1f5c8f]" checked={npp} onChange={(e) => setNpp(e.target.checked)} />
                  <span className="text-ink">I acknowledge the <Link href="/privacy" className="font-semibold text-ocean underline">Notice of Privacy Practices</Link>. <span className="text-danger">*</span></span>
                </label>
                <label className="flex cursor-pointer items-start gap-2.5">
                  <input type="checkbox" className="mt-0.5 accent-[#1f5c8f]" checked={sms} onChange={(e) => setSms(e.target.checked)} />
                  <span className="text-muted">Text me appointment reminders. Messages never include diagnoses or test results. Reply STOP to opt out. (Optional)</span>
                </label>
              </div>

              <Button type="submit" variant="sage" className="mt-6 w-full" disabled={!valid}><Lock className="size-4" /> Submit securely</Button>
            </form>
          )}
        </section>

        <aside className="space-y-4">
          <div className="rounded-3xl border border-line bg-white p-6 shadow-[var(--shadow-soft)]">
            <div className="text-[12px] font-semibold uppercase tracking-wide text-sage">Your visit</div>
            {rec ? (
              <>
                <div className="mt-1 font-display text-lg font-semibold text-ink">{rec.service}</div>
                <div className="text-[13px] text-muted">{rec.who} · {rec.where}</div>
                <div className="mt-3 inline-flex items-center gap-2 rounded-lg bg-sky-2 px-3 py-1.5 text-[13px] font-semibold text-ink"><CalendarCheck2 className="size-4 text-sage" /> {rec.when}</div>
                <ul className="mt-4 space-y-1.5 text-[12.5px] text-muted">
                  {rec.bring.map((b) => <li key={b} className="flex gap-2"><CheckCircle2 className="mt-0.5 size-3.5 shrink-0 text-sage" />{b}</li>)}
                </ul>
              </>
            ) : (
              <p className="mt-2 text-[13px] text-muted">No visit selected yet. <Link href="/#request" className="font-semibold text-ocean">Start a smart appointment request</Link>.</p>
            )}
          </div>
          <div className="rounded-3xl bg-ink p-6 text-white">
            <div className="flex items-center gap-2 text-[13px] font-semibold"><ShieldCheck className="size-4 text-gold-2" /> How this page protects you</div>
            <ul className="mt-3 space-y-2 text-[12.5px] text-white/70">
              {[
                "Separated from the marketing site, which collects no personal data",
                "Encrypted in transit (TLS) and at rest (AES-256)",
                "Hosted by a vendor that signs a Business Associate Agreement",
                "No analytics, ad pixels or chat widgets on this page",
                "Never cached; clears itself after 15 minutes idle",
                "Every access is written to an audit log",
              ].map((t) => <li key={t} className="flex gap-2"><KeyRound className={cx("mt-0.5 size-3.5 shrink-0 text-sage-2")} />{t}</li>)}
            </ul>
          </div>
        </aside>
      </main>
    </div>
  );
}
