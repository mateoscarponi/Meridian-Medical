// Claim scrubber: deterministic edit rules (NCCI-style bundling, modifiers, code validity,
// payer-specific timely filing and NY workers' comp / no-fault requirements) + a denial-risk score.

export interface ClaimLine {
  cpt: string;
  desc: string;
  modifiers: string[];
  units: number;
  icd10: string[];
  charge: number;
}

export interface Claim {
  id: string;
  patient: string;
  payer: string;
  payerType: "Commercial" | "Medicare" | "Medicare Advantage" | "Medicaid" | "Workers' Comp" | "No-Fault";
  dos: string;
  daysSinceDos: number;
  provider: string;
  lines: ClaimLine[];
  authNumber?: string;
  wcbCaseNumber?: string;
  hasJustificationLetter?: boolean;
}

export interface ScrubFinding {
  rule: string;
  severity: "critical" | "warning";
  message: string;
  fix: string;
  weight: number;
  apply?: (c: Claim) => Claim;
}

const clone = (c: Claim): Claim => JSON.parse(JSON.stringify(c));
const has = (c: Claim, cpt: string) => c.lines.some((l) => l.cpt === cpt);

const AUTH_REQUIRED: Record<string, Claim["payerType"][]> = {
  "72148": ["Commercial", "Medicare Advantage"],
  "64483": ["Commercial", "Medicare Advantage"],
  "62321": ["Commercial", "Medicare Advantage"],
  "22551": ["Commercial", "Medicare Advantage", "Medicaid"],
  "63047": ["Commercial", "Medicare Advantage", "Medicaid"],
};

export function scrub(c: Claim): ScrubFinding[] {
  const f: ScrubFinding[] = [];
  const injection = c.lines.find((l) => ["64483", "64484", "62321", "62323", "64490", "64493"].includes(l.cpt));

  if (injection && has(c, "77003")) {
    f.push({
      rule: "NCCI bundling",
      severity: "critical",
      message: `77003 (fluoroscopic guidance) is bundled into ${injection.cpt}; imaging guidance is included in the injection code.`,
      fix: "Remove the 77003 line.",
      weight: 34,
      apply: (x) => ({ ...clone(x), lines: x.lines.filter((l) => l.cpt !== "77003") }),
    });
  }

  const em = c.lines.find((l) => /^992[01]\d$/.test(l.cpt));
  if (em && injection && !em.modifiers.includes("25")) {
    f.push({
      rule: "Modifier 25",
      severity: "critical",
      message: `E/M ${em.cpt} billed on the same day as procedure ${injection.cpt} without modifier 25.`,
      fix: "Append modifier 25: the note documents a separately identifiable evaluation (new complaint + decision for injection).",
      weight: 28,
      apply: (x) => {
        const y = clone(x);
        y.lines.forEach((l) => /^992[01]\d$/.test(l.cpt) && l.modifiers.push("25"));
        return y;
      },
    });
  }

  if (c.lines.some((l) => l.icd10.includes("M54.5"))) {
    f.push({
      rule: "Invalid ICD-10",
      severity: "critical",
      message: "M54.5 has been invalid since Oct 1, 2021 (expanded to M54.50 / M54.51 / M54.59).",
      fix: "Replace with M54.50 (low back pain, unspecified) — or a more specific code supported by the note.",
      weight: 40,
      apply: (x) => {
        const y = clone(x);
        y.lines.forEach((l) => (l.icd10 = l.icd10.map((d) => (d === "M54.5" ? "M54.50" : d))));
        return y;
      },
    });
  }

  const tfesi = c.lines.filter((l) => l.cpt === "64483");
  if (c.payerType === "Medicare" && tfesi.length === 2 && tfesi.some((l) => l.modifiers.includes("RT")) && tfesi.some((l) => l.modifiers.includes("LT"))) {
    f.push({
      rule: "Bilateral billing",
      severity: "warning",
      message: "Bilateral 64483 billed as two lines (RT + LT). Medicare expects one line with modifier 50.",
      fix: "Collapse into a single 64483-50 line, 1 unit, combined charge.",
      weight: 18,
      apply: (x) => {
        const y = clone(x);
        const lines = y.lines.filter((l) => l.cpt === "64483");
        const merged: ClaimLine = { ...lines[0], modifiers: ["50"], units: 1, charge: lines.reduce((s, l) => s + l.charge, 0) };
        y.lines = [merged, ...y.lines.filter((l) => l.cpt !== "64483")];
        return y;
      },
    });
  }

  if (injection && injection.icd10.every((d) => d.startsWith("M54.5"))) {
    f.push({
      rule: "Medical necessity",
      severity: "warning",
      message: `${injection.cpt} is linked only to non-specific low back pain; payers expect a radicular diagnosis for epidural injections.`,
      fix: "Link M54.16 (lumbar radiculopathy), documented in the 11/10 note by NLP chart review.",
      weight: 24,
      apply: (x) => {
        const y = clone(x);
        y.lines.forEach((l) => l.cpt === injection.cpt && (l.icd10 = ["M54.16", ...l.icd10.filter((d) => !d.startsWith("M54.5"))]));
        return y;
      },
    });
  }

  const needsAuth = c.lines.some((l) => AUTH_REQUIRED[l.cpt]?.includes(c.payerType));
  if (needsAuth && !c.authNumber) {
    f.push({
      rule: "Missing authorization",
      severity: "critical",
      message: "Service requires prior authorization but no auth number is on the claim (REF*G1).",
      fix: "Attach approved auth from the Prior Auth module.",
      weight: 45,
      apply: (x) => ({ ...clone(x), authNumber: "AUTH-7714502" }),
    });
  }

  if (c.payerType === "No-Fault" && c.daysSinceDos > 45 && !c.hasJustificationLetter) {
    f.push({
      rule: "NY no-fault 45-day rule",
      severity: "critical",
      message: `Bill is ${c.daysSinceDos} days from date of service; NY no-fault requires submission within 45 days unless a reasonable justification is provided.`,
      fix: "Attach the auto-drafted late-submission justification (records received late from the hospital).",
      weight: 38,
      apply: (x) => ({ ...clone(x), hasJustificationLetter: true }),
    });
  }

  if (c.payerType === "Workers' Comp" && !c.wcbCaseNumber) {
    f.push({
      rule: "WCB case number",
      severity: "critical",
      message: "Workers' compensation bill is missing the WCB case number and carrier claim number.",
      fix: "Pull WCB case # from the patient's C-4 on file and the carrier claim # from the intake form.",
      weight: 36,
      apply: (x) => ({ ...clone(x), wcbCaseNumber: "G331 8042" }),
    });
  }

  return f;
}

export function denialRisk(c: Claim) {
  const findings = scrub(c);
  const raw = 6 + findings.reduce((s, x) => s + x.weight, 0);
  return { findings, risk: Math.min(97, raw) };
}

export function total(c: Claim) {
  return c.lines.reduce((s, l) => s + l.charge, 0);
}

export const CLAIMS: Claim[] = [
  {
    id: "CLM-90431", patient: "James O'Connor", payer: "Aetna PPO", payerType: "Commercial", dos: "11/10/2026", daysSinceDos: 6, provider: "Dr. Okafor",
    lines: [
      { cpt: "99214", desc: "Office visit, est. patient, moderate MDM", modifiers: [], units: 1, icd10: ["M51.16"], charge: 210 },
      { cpt: "64483", desc: "TFESI lumbar/sacral, single level", modifiers: ["RT"], units: 1, icd10: ["M51.16"], charge: 1150 },
      { cpt: "77003", desc: "Fluoroscopic guidance, spine injection", modifiers: [], units: 1, icd10: ["M51.16"], charge: 180 },
    ],
    authNumber: "AUTH-7714380",
  },
  {
    id: "CLM-90432", patient: "Harold Becker", payer: "Medicare Part B", payerType: "Medicare", dos: "11/09/2026", daysSinceDos: 7, provider: "Dr. Okafor",
    lines: [
      { cpt: "64483", desc: "TFESI lumbar/sacral, single level", modifiers: ["RT"], units: 1, icd10: ["M54.5"], charge: 1150 },
      { cpt: "64483", desc: "TFESI lumbar/sacral, single level", modifiers: ["LT"], units: 1, icd10: ["M54.5"], charge: 1150 },
    ],
  },
  {
    id: "CLM-90433", patient: "Carmen Diaz", payer: "Empire BCBS", payerType: "Commercial", dos: "11/13/2026", daysSinceDos: 3, provider: "D. Park, PA-C",
    lines: [{ cpt: "99203", desc: "Office visit, new patient, low MDM", modifiers: [], units: 1, icd10: ["M54.31"], charge: 260 }],
  },
  {
    id: "CLM-90434", patient: "Aisha Rahman", payer: "GEICO No-Fault", payerType: "No-Fault", dos: "09/22/2026", daysSinceDos: 55, provider: "Dr. Ruiz",
    lines: [
      { cpt: "99204", desc: "Office visit, new patient, moderate MDM", modifiers: [], units: 1, icd10: ["S13.4XXA", "M54.2"], charge: 340 },
      { cpt: "72040", desc: "X-ray cervical spine, 2-3 views", modifiers: [], units: 1, icd10: ["S13.4XXA"], charge: 120 },
    ],
  },
  {
    id: "CLM-90435", patient: "Robert Williams", payer: "NYSIF", payerType: "Workers' Comp", dos: "11/11/2026", daysSinceDos: 5, provider: "Dr. Okafor",
    lines: [{ cpt: "62321", desc: "Interlaminar ESI cervical/thoracic w/ imaging", modifiers: [], units: 1, icd10: ["M54.12"], charge: 980 }],
  },
  {
    id: "CLM-90436", patient: "Priya Natarajan", payer: "Cigna Open Access", payerType: "Commercial", dos: "11/12/2026", daysSinceDos: 4, provider: "Dr. Mateo",
    lines: [
      { cpt: "99204", desc: "Office visit, new patient, moderate MDM", modifiers: [], units: 1, icd10: ["M41.26"], charge: 340 },
      { cpt: "72082", desc: "X-ray entire spine, 2-3 views (scoliosis)", modifiers: [], units: 1, icd10: ["M41.26"], charge: 210 },
    ],
  },
  {
    id: "CLM-90437", patient: "Linda Chen", payer: "Humana Medicare Advantage", payerType: "Medicare Advantage", dos: "11/06/2026", daysSinceDos: 10, provider: "Dr. Okafor",
    lines: [
      { cpt: "99214", desc: "Office visit, est. patient, moderate MDM", modifiers: [], units: 1, icd10: ["M48.062"], charge: 210 },
      { cpt: "62323", desc: "Interlaminar ESI lumbar/sacral w/ imaging", modifiers: [], units: 1, icd10: ["M48.062"], charge: 1020 },
    ],
  },
  {
    id: "CLM-90438", patient: "Kevin Murphy", payer: "Oxford (UHC)", payerType: "Commercial", dos: "11/14/2026", daysSinceDos: 2, provider: "D. Park, PA-C",
    lines: [{ cpt: "99213", desc: "Office visit, est. patient, low MDM", modifiers: [], units: 1, icd10: ["M54.2"], charge: 150 }],
  },
];
