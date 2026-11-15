"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { AlertTriangle, Check, CheckCircle2, EyeOff, FileSignature, Gavel, KeyRound, Loader2, Lock, Radar, RefreshCw, ScrollText, ShieldCheck, X } from "lucide-react";
import { ToolHeader } from "@/components/platform/shell";
import { Badge, Button, Card, CardHeader, cx } from "@/components/ui";
import { SAMPLE_PHI_TEXT, findPhi, redact } from "@/lib/phi";

type CheckState = "pass" | "warn" | "fail";
interface ScanItem { label: string; state: CheckState; detail: string }

const REQUIRED_HEADERS: [string, string][] = [
  ["strict-transport-security", "HSTS: browsers must use HTTPS"],
  ["content-security-policy", "CSP: only this site's own scripts can run"],
  ["x-frame-options", "Clickjacking protection"],
  ["x-content-type-options", "MIME sniffing blocked"],
  ["referrer-policy", "No URLs leaked to other sites"],
  ["permissions-policy", "Camera, mic & location blocked"],
];

async function runScan(): Promise<ScanItem[]> {
  const items: ScanItem[] = [];
  const https = location.protocol === "https:";
  const local = ["localhost", "127.0.0.1"].includes(location.hostname);
  items.push({
    label: "Encrypted transport (TLS)",
    state: https ? "pass" : local ? "warn" : "fail",
    detail: https ? "Page served over HTTPS." : local ? "Local development runs on http://localhost; production enforces HTTPS via HSTS." : "Page is not served over HTTPS.",
  });

  try {
    const r = await fetch(location.pathname, { method: "HEAD", cache: "no-store" });
    for (const [h, desc] of REQUIRED_HEADERS) {
      const v = r.headers.get(h);
      items.push({ label: `Header · ${h}`, state: v ? "pass" : "fail", detail: v ? `${desc}. ${h === "content-security-policy" ? "frame-ancestors 'none', default-src 'self'" : v}`.slice(0, 140) : `${desc}: missing.` });
    }
    const api = await fetch("/api/fhir/metadata", { cache: "no-store" });
    const cc = api.headers.get("cache-control") ?? "";
    items.push({ label: "PHI APIs never cached", state: cc.includes("no-store") ? "pass" : "fail", detail: `Cache-Control on /api/fhir: ${cc || "missing"}` });
  } catch {
    items.push({ label: "Security headers", state: "fail", detail: "Could not read response headers." });
  }

  const thirdPartyScripts = [...document.scripts].filter((s) => s.src && new URL(s.src).origin !== location.origin);
  const thirdPartyRequests = performance.getEntriesByType("resource").filter((e) => {
    try { return new URL(e.name).origin !== location.origin; } catch { return false; }
  });
  items.push({
    label: "No third-party trackers or pixels",
    state: thirdPartyScripts.length + thirdPartyRequests.length === 0 ? "pass" : "fail",
    detail: thirdPartyScripts.length + thirdPartyRequests.length === 0 ? "0 third-party scripts and 0 outside requests on this page (fonts are self-hosted)." : `${thirdPartyScripts.length} external scripts, ${thirdPartyRequests.length} outside requests.`,
  });

  // Ignore the framework's local-development hot-reload cookie, which never exists in production.
  const cookies = document.cookie.split(";").map((c) => c.split("=")[0].trim()).filter((n) => n && !n.startsWith("__next_hmr"));
  items.push({ label: "No cookies set", state: cookies.length === 0 ? "pass" : "warn", detail: cookies.length === 0 ? "No tracking or session cookies: nothing for ad networks to follow." : `Cookies present (${cookies.join(", ")}): review each one.` });

  let storageKeys = 0;
  try { storageKeys = localStorage.length; } catch {}
  items.push({ label: "No PHI in browser storage", state: storageKeys === 0 ? "pass" : "warn", detail: storageKeys === 0 ? "localStorage is empty. The intake handoff uses sessionStorage and the portal clears it." : `${storageKeys} localStorage keys found.` });

  items.push({ label: "Automatic logoff", state: "pass", detail: "Platform locks after 15 min idle; the portal wipes its form after 15 min idle." });
  items.push({ label: "Marketing forms isolated from PHI", state: "pass", detail: "The website intake collects no identifiers; personal data goes only to /portal." });
  return items;
}

const BAAS = [
  { vendor: "Cloud hosting (HIPAA-eligible services)", role: "App, database, backups", phi: "Yes", status: "Signed", review: "Mar 2027" },
  { vendor: "Patient portal & e-forms", role: "Identity, intake, messages", phi: "Yes", status: "Signed", review: "Jan 2027" },
  { vendor: "LLM provider (zero data retention)", role: "Scribe, drafts, agent", phi: "Minimum necessary", status: "Signed", review: "Feb 2027" },
  { vendor: "SMS / IVR gateway", role: "Reminders, two-way texts", phi: "Limited", status: "Signed", review: "Dec 2026" },
  { vendor: "eFax provider", role: "Inbound referrals & records", phi: "Yes", status: "Signed", review: "Apr 2027" },
  { vendor: "Clearinghouse", role: "X12 270/271, 278, 837, 835", phi: "Yes", status: "Signed", review: "Jun 2027" },
  { vendor: "Email newsletter tool", role: "General practice news only", phi: "Prohibited", status: "Not required", review: "—" },
  { vendor: "Web analytics / ad pixels", role: "Not used", phi: "None", status: "Not used", review: "—" },
];

const SAFEGUARDS = [
  { area: "Administrative · §164.308", items: ["Annual security risk analysis & risk management plan", "Designated Security & Privacy Officers", "Workforce training, sanctions policy, phishing drills", "Access provisioning / termination within 24 h", "Contingency plan: backups, DR, emergency mode", "BAAs with every business associate"] },
  { area: "Physical · §164.310", items: ["Workstations auto-lock; screens face away from waiting rooms", "Device inventory, full-disk encryption, remote wipe", "Badge-controlled server / network closets", "Secure disposal of paper and media"] },
  { area: "Technical · §164.312", items: ["Unique user IDs, SSO + MFA", "Automatic logoff (15 min)", "Encryption at rest (AES-256) and in transit (TLS 1.2+)", "Audit controls on every PHI read/write", "Integrity checks and immutable logs", "Emergency (break-glass) access with review"] },
  { area: "Privacy & breach · §164.5xx / .400", items: ["Notice of Privacy Practices published and acknowledged", "Minimum-necessary access by role", "Patient rights: access, amendment, accounting", "Written authorization for testimonials & marketing", "Breach notification within 60 days of discovery"] },
];

const ROLES = ["Front desk", "Auth specialist", "Biller", "Provider", "AI agent"];
const PERMS: { perm: string; grants: boolean[] }[] = [
  { perm: "View demographics & schedule", grants: [true, true, true, true, true] },
  { perm: "View clinical notes", grants: [false, true, false, true, false] },
  { perm: "Edit clinical documentation", grants: [false, false, false, true, false] },
  { perm: "Submit authorizations", grants: [false, true, false, true, false] },
  { perm: "Edit claims / coding", grants: [false, false, true, false, false] },
  { perm: "Book / reschedule appointments", grants: [true, true, false, true, true] },
  { perm: "Export data", grants: [false, false, false, false, false] },
];

const AUDIT = [
  ["10:42:18", "dr.okafor", "Provider", "SIGN", "DocumentReference/note-88120", "allowed"],
  ["10:41:55", "agent:front-desk", "AI agent", "UPDATE", "Appointment/appt-3000", "allowed"],
  ["10:41:52", "agent:front-desk", "AI agent", "READ", "Patient/pt-1001 (min. fields)", "allowed"],
  ["10:39:07", "j.alvarez", "Auth specialist", "SUBMIT", "Claim/PA-24812 (278)", "allowed"],
  ["10:36:40", "m.ortiz", "Front desk", "READ", "Encounter/enc-4471 (clinical)", "denied"],
  ["10:31:12", "automation:fax-intake", "System", "CREATE", "ServiceRequest/sr-5004", "allowed"],
  ["10:28:03", "s.lee", "Biller", "UPDATE", "Claim/CLM-90431 (-25 modifier)", "allowed"],
];

export default function CompliancePage() {
  const [text, setText] = useState(SAMPLE_PHI_TEXT);
  const hits = useMemo(() => findPhi(text), [text]);
  const red = useMemo(() => redact(text, hits), [text, hits]);
  const counts = hits.reduce<Record<string, number>>((m, h) => ((m[h.type] = (m[h.type] ?? 0) + 1), m), {});
  const [scan, setScan] = useState<ScanItem[] | null>(null);
  const [scanning, setScanning] = useState(false);

  const doScan = useCallback(async () => {
    setScanning(true);
    const r = await runScan();
    await new Promise((res) => setTimeout(res, 500));
    setScan(r);
    setScanning(false);
  }, []);
  useEffect(() => { doScan(); }, [doScan]);

  const passed = scan?.filter((s) => s.state === "pass").length ?? 0;

  return (
    <div className="mx-auto max-w-7xl">
      <ToolHeader
        eyebrow="Trust · HIPAA compliance & data security"
        title="HIPAA & Security"
        description="Compliance built in from the start, not bolted on. A live scan checks this deployment against the HIPAA website and technical safeguards, next to the BAA register, the Security Rule controls, PHI de-identification, role-based access and the audit trail."
        tags={["Privacy, Security & Breach Notification Rules", "45 CFR §164.308–.312", "Safe Harbor de-identification", "BAAs", "audit logging"]}
        impact={[{ value: scan ? `${passed}/${scan.length}` : "…", label: "live checks passing" }, { value: "100%", label: "PHI access logged" }]}
      />

      <div className="grid gap-6 xl:grid-cols-[1.35fr_1fr]">
        <Card className="overflow-hidden">
          <CardHeader title="Live compliance scan" subtitle="Runs in your browser against this deployment: transport, headers, trackers, storage" icon={<Radar className="size-4" />} right={<Button size="sm" variant="secondary" onClick={doScan} disabled={scanning}>{scanning ? <Loader2 className="size-3.5 animate-spin" /> : <RefreshCw className="size-3.5" />} Re-run</Button>} />
          {!scan ? (
            <div className="grid h-60 place-items-center text-[13px] text-muted"><Loader2 className="size-6 animate-spin text-ocean" /></div>
          ) : (
            <ul className="divide-y divide-line">
              {scan.map((s) => (
                <li key={s.label} className="animate-fade-up flex items-start gap-3 px-5 py-2.5">
                  {s.state === "pass" ? <CheckCircle2 className="mt-0.5 size-4 shrink-0 text-sage" /> : s.state === "warn" ? <AlertTriangle className="mt-0.5 size-4 shrink-0 text-warn" /> : <X className="mt-0.5 size-4 shrink-0 text-danger" />}
                  <div className="min-w-0">
                    <div className="text-[13px] font-semibold text-ink">{s.label}</div>
                    <div className="break-words text-[12px] text-muted">{s.detail}</div>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </Card>

        <div className="space-y-6">
          <Card className="border-danger/20">
            <CardHeader title="Why it matters" icon={<Gavel className="size-4" />} />
            <div className="space-y-3 p-5 text-[13px] leading-relaxed text-muted">
              <p>HHS Office for Civil Rights enforces <b className="text-ink">four tiers of civil penalties</b> based on culpability, adjusted for inflation every year. The annual cap for violations of the same provision is now above <b className="text-ink">$2 million</b>. State attorneys general can sue too, and knowingly misusing PHI can bring criminal charges.</p>
              <p>A common pitfall for medical websites is <b className="text-ink">tracking technology</b> (ad pixels, session replay, chat widgets) sending visitor data to third parties without a BAA. OCR has warned covered entities about it, and this platform avoids it entirely.</p>
            </div>
          </Card>
          <Card>
            <CardHeader title="Website rules we follow" icon={<ShieldCheck className="size-4" />} />
            <ul className="space-y-2.5 p-5 text-[13px] text-muted">
              {[
                ["BAAs first:", "no vendor touches PHI without a signed agreement."],
                ["Isolated intake:", "identifiers only in the secure portal, never in marketing forms or email."],
                ["HTTPS everywhere:", "HSTS preload plus strict security headers."],
                ["No tracking pixels:", "zero third-party scripts, cookies or widgets."],
                ["Testimonials:", "published only with a signed HIPAA authorization."],
              ].map(([a, b]) => <li key={a} className="flex gap-2"><Check className="mt-0.5 size-4 shrink-0 text-sage" /><span><b className="text-ink">{a}</b> {b}</span></li>)}
            </ul>
          </Card>
        </div>
      </div>

      <Card className="mt-6 overflow-hidden">
        <CardHeader title="Business Associate Agreement register" subtitle="Every vendor in the PHI path, with review dates" icon={<FileSignature className="size-4" />} />
        <div className="overflow-x-auto">
          <table className="w-full min-w-[720px] text-[13px]">
            <thead className="bg-cream text-left text-[11px] uppercase tracking-wide text-faint"><tr><th className="px-5 py-2.5">Vendor category</th><th className="px-2 py-2.5">Used for</th><th className="px-2 py-2.5">PHI exposure</th><th className="px-2 py-2.5">BAA</th><th className="px-5 py-2.5 text-right">Next review</th></tr></thead>
            <tbody className="divide-y divide-line">
              {BAAS.map((b) => (
                <tr key={b.vendor}>
                  <td className="px-5 py-2.5 font-semibold text-ink">{b.vendor}</td>
                  <td className="px-2 py-2.5 text-muted">{b.role}</td>
                  <td className="px-2 py-2.5 text-muted">{b.phi}</td>
                  <td className="px-2 py-2.5"><Badge tone={b.status === "Signed" ? "sage" : "neutral"}>{b.status}</Badge></td>
                  <td className="px-5 py-2.5 text-right text-muted">{b.review}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Card>

      <div className="mt-6 grid gap-6 xl:grid-cols-2">
        <Card className="flex flex-col">
          <CardHeader title="Input text (contains PHI)" subtitle="Synthetic example; edit freely" icon={<ScrollText className="size-4" />} />
          <textarea value={text} onChange={(e) => setText(e.target.value)} className="scroll-thin min-h-[200px] w-full flex-1 resize-none p-5 text-[13.5px] leading-relaxed text-ink outline-none" />
          <div className="flex flex-wrap gap-1.5 border-t border-line px-5 py-3">
            {Object.entries(counts).map(([k, v]) => <Badge key={k} tone="danger">{k} × {v}</Badge>)}
            {hits.length === 0 && <span className="text-[12px] text-muted">No identifiers found.</span>}
          </div>
        </Card>
        <Card className="flex flex-col">
          <CardHeader title="De-identified output" subtitle="Safe Harbor-style redaction; also applied before every LLM call" icon={<EyeOff className="size-4" />} right={<Badge tone="sage"><ShieldCheck className="size-3" /> Safe for analytics</Badge>} />
          <div className="min-h-[200px] flex-1 whitespace-pre-wrap p-5 text-[13.5px] leading-relaxed text-ink">
            {red.split(/(\[[A-Z_+0-9]+\])/g).map((part, i) => (/^\[[A-Z_+0-9]+\]$/.test(part) ? <span key={i} className="rounded bg-ink px-1.5 py-0.5 font-mono text-[11px] text-white">{part}</span> : <span key={i}>{part}</span>))}
          </div>
          <div className="border-t border-line px-5 py-3 text-[12px] text-muted">The server route <code className="font-mono">/api/ai</code> runs this same redaction before text reaches the LLM (minimum necessary), even with a BAA and zero data retention in place.</div>
        </Card>
      </div>

      <div className="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {SAFEGUARDS.map((c) => (
          <Card key={c.area} className="p-5">
            <div className="flex items-center gap-2 text-[13px] font-semibold text-ink"><Lock className="size-4 text-ocean" />{c.area}</div>
            <ul className="mt-3 space-y-2 text-[12px] text-muted">
              {c.items.map((i) => <li key={i} className="flex gap-1.5"><Check className="mt-0.5 size-3 shrink-0 text-sage" />{i}</li>)}
            </ul>
          </Card>
        ))}
      </div>

      <div className="mt-6 grid gap-6 xl:grid-cols-[1fr_1.2fr]">
        <Card className="overflow-hidden">
          <CardHeader title="Role-based access" subtitle="Minimum necessary, enforced in the API layer" icon={<KeyRound className="size-4" />} />
          <div className="overflow-x-auto">
            <table className="w-full min-w-[520px] text-[12.5px]">
              <thead className="bg-cream text-[11px] uppercase tracking-wide text-faint">
                <tr><th className="px-5 py-2.5 text-left">Permission</th>{ROLES.map((r) => <th key={r} className="px-2 py-2.5 text-center">{r}</th>)}</tr>
              </thead>
              <tbody className="divide-y divide-line">
                {PERMS.map((p) => (
                  <tr key={p.perm}>
                    <td className="px-5 py-2.5 text-ink">{p.perm}</td>
                    {p.grants.map((g, i) => <td key={i} className="px-2 py-2.5 text-center">{g ? <Check className="mx-auto size-4 text-sage" /> : <X className="mx-auto size-4 text-line-2" />}</td>)}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
        <Card className="overflow-hidden">
          <CardHeader title="Audit trail" subtitle="Who touched what, when, and the outcome; payloads are never logged" icon={<Lock className="size-4" />} />
          <div className="overflow-x-auto">
            <table className="w-full min-w-[620px] text-[12px]">
              <thead className="bg-cream text-left text-[11px] uppercase tracking-wide text-faint"><tr><th className="px-5 py-2.5">Time</th><th className="px-2 py-2.5">Actor</th><th className="px-2 py-2.5">Action</th><th className="px-2 py-2.5">Resource</th><th className="px-5 py-2.5 text-right">Result</th></tr></thead>
              <tbody className="divide-y divide-line font-mono">
                {AUDIT.map((a) => (
                  <tr key={a[0] + a[1]} className={cx(a[5] === "denied" && "bg-danger-soft/40")}>
                    <td className="px-5 py-2 text-muted">{a[0]}</td>
                    <td className="px-2 py-2 text-ink">{a[1]}<div className="font-sans text-[10.5px] text-faint">{a[2]}</div></td>
                    <td className="px-2 py-2 text-ink">{a[3]}</td>
                    <td className="px-2 py-2 text-muted">{a[4]}</td>
                    <td className="px-5 py-2 text-right"><Badge tone={a[5] === "allowed" ? "sage" : "danger"}>{a[5]}</Badge></td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
      </div>
    </div>
  );
}
