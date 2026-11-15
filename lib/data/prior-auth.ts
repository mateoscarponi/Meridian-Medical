// Prior authorization cases + payer criteria evaluated against the chart with the NLP layer.
// Criteria are illustrative composites of common commercial / RBM policy language, not any
// specific payer's proprietary policy.
import { extractEntities, conservativeCareWeeks, redFlags } from "../nlp";

export type CritStatus = "met" | "missing" | "na";

export interface CriterionResult {
  label: string;
  status: CritStatus;
  evidence: string;
  fix?: string;
}

export interface AuthCase {
  id: string;
  patientId: string;
  patient: string;
  payer: string;
  payerType: string;
  cpt: string;
  service: string;
  icd10: string[];
  provider: string;
  receivedDaysAgo: number;
  urgency: "standard" | "expedited";
  pathway: "Payer portal / X12 278" | "Da Vinci PAS (FHIR)" | "NY WCB Medical Portal" | "Not required";
  note: string;
  criteria: (note: string) => CriterionResult[];
  attachments: string[];
}

const findSentence = (note: string, re: RegExp) => note.split(/(?<=[.!?])\s+/).find((s) => re.test(s))?.trim() ?? "";

function weeksCrit(note: string, min: number): CriterionResult {
  const { weeks, evidence } = conservativeCareWeeks(note);
  const ok = weeks >= min;
  return {
    label: `≥ ${min} weeks of failed conservative care (PT / HEP / meds)`,
    status: ok ? "met" : "missing",
    evidence: evidence[0] ? `${weeks} wk documented — “${evidence[0]}”` : "No dated conservative therapy found in the note.",
    fix: ok ? undefined : `Only ${weeks} wk documented. Copilot schedules a re-evaluation when week ${min} is reached, or flags the case if a red-flag exception applies.`,
  };
}

function redFlagExceptionCrit(note: string): CriterionResult {
  const rf = redFlags(extractEntities(note));
  return {
    label: "Red-flag exception (bypasses conservative-care requirement)",
    status: rf.urgent ? "met" : "na",
    evidence: rf.urgent
      ? `Active red flag: ${rf.active.map((r) => r.text).join(", ")}`
      : rf.negated.length
        ? `Red flags explicitly denied: ${rf.negated.map((r) => r.text).join(", ")}`
        : "No red flags documented.",
  };
}

export const AUTH_CASES: AuthCase[] = [
  {
    id: "PA-24812",
    patientId: "pt-1002",
    patient: "James O'Connor",
    payer: "Aetna PPO",
    payerType: "Commercial",
    cpt: "64483",
    service: "Right L4-L5 transforaminal ESI with imaging guidance",
    icd10: ["M51.16"],
    provider: "Dr. Samuel Okafor, MD",
    receivedDaysAgo: 0,
    urgency: "standard",
    pathway: "Payer portal / X12 278",
    note: `48-year-old male with low back pain and pain radiating down the right leg for 10 weeks after lifting at work. Reports numbness and tingling in the right foot and mild weakness with great toe extension. Completed 8 weeks of physical therapy with home exercise program and 6 weeks of naproxen without lasting relief. Denies bowel or bladder incontinence, saddle anesthesia, fever, or unexplained weight loss. MRI of the lumbar spine shows a right paracentral disc extrusion at L4-L5 contacting the descending right L5 nerve root with moderate lateral recess stenosis. Positive straight leg raise on the right, diminished sensation in the right L5 dermatome. No prior epidural injections. Plan: right L4-L5 transforaminal epidural steroid injection under fluoroscopy.`,
    criteria: (n) => [
      {
        label: "Radicular pain in a dermatomal distribution",
        status: /radiat|radicul|dermatome|sciatica/i.test(n) ? "met" : "missing",
        evidence: findSentence(n, /radiat|radicul|dermatome/i),
      },
      weeksCrit(n, 4),
      {
        label: "Imaging corroborates nerve-root involvement at the treated level",
        status: /(nerve root|stenosis|herniation|extrusion)/i.test(n) && /L4-L5/.test(n) ? "met" : "missing",
        evidence: findSentence(n, /MRI/i),
      },
      {
        label: "No ESI in the previous 6 months / ≤ 4 sessions per 12 months",
        status: /no prior epidural/i.test(n) ? "met" : "missing",
        evidence: findSentence(n, /prior epidural/i) || "Injection history not documented.",
        fix: "Pull injection history from the EMR procedure log.",
      },
      {
        label: "Performed with fluoroscopic or CT guidance",
        status: /fluoroscop|CT guid|imaging guid/i.test(n) ? "met" : "missing",
        evidence: findSentence(n, /fluoroscop|guid/i),
      },
    ],
    attachments: ["Office note 11/12/2026", "MRI report — Nassau Imaging 11/15/2026", "PT discharge summary — Go Rehab"],
  },
  {
    id: "PA-24813",
    patientId: "pt-1006",
    patient: "Thomas Russo",
    payer: "UnitedHealthcare",
    payerType: "Commercial",
    cpt: "22551",
    service: "ACDF, single interspace C5-C6 (with 22853 interbody device, 22845 anterior instrumentation)",
    icd10: ["M50.122"],
    provider: "Dr. Adrian Mateo, DO",
    receivedDaysAgo: 3,
    urgency: "standard",
    pathway: "Payer portal / X12 278",
    note: `67-year-old male with neck pain and pain radiating into the left arm with numbness in the thumb and index finger for 5 months. Progressive weakness of left wrist extension (4/5). Completed 12 weeks of physical therapy and a trial of gabapentin for 8 weeks without improvement; one cervical interlaminar epidural injection gave 2 weeks of relief. MRI of the cervical spine shows a left foraminal disc herniation at C5-C6 with severe foraminal stenosis and nerve root compression concordant with C6 radiculopathy. Flexion-extension X-rays show no instability. Neck Disability Index 46%. Plan: anterior cervical discectomy and fusion at C5-C6.`,
    criteria: (n) => [
      {
        label: "Radiculopathy or myelopathy with objective neurologic findings",
        status: /weakness|reflex|myelopath|numbness/i.test(n) ? "met" : "missing",
        evidence: findSentence(n, /weakness/i),
      },
      weeksCrit(n, 6),
      {
        label: "Imaging shows compression concordant with symptoms",
        status: /concordant|compression/i.test(n) ? "met" : "missing",
        evidence: findSentence(n, /MRI/i),
      },
      {
        label: "Functional impairment documented (NDI / ODI)",
        status: /disability index|ODI|NDI/i.test(n) ? "met" : "missing",
        evidence: findSentence(n, /Disability Index|ODI|NDI/i),
      },
      {
        label: "Nicotine status documented (non-smoker or cessation ≥ 6 weeks)",
        status: /smok|nicotine|tobacco/i.test(n) ? "met" : "missing",
        evidence: findSentence(n, /smok|nicotine|tobacco/i) || "Not in this note.",
        fix: "Auto-pulled from EMR social history (FHIR Observation 72166-2 smoking status): “Never smoker”, recorded 09/2026. Added to packet.",
      },
    ],
    attachments: ["Surgical consult note", "MRI cervical spine report", "Flexion-extension X-ray report", "PT notes (12 wk)", "NDI questionnaire"],
  },
  {
    id: "PA-24814",
    patientId: "pt-1003",
    patient: "Linda Chen",
    payer: "Humana Medicare Advantage",
    payerType: "Medicare Advantage",
    cpt: "63047",
    service: "Lumbar laminectomy / decompression, single segment L4-L5",
    icd10: ["M48.062"],
    provider: "Dr. Adrian Mateo, DO",
    receivedDaysAgo: 1,
    urgency: "standard",
    pathway: "Da Vinci PAS (FHIR)",
    note: `74-year-old female with neurogenic claudication: bilateral buttock and thigh pain when walking more than one block, relieved by sitting and leaning forward, for 14 months. Completed 12 weeks of physical therapy, home exercise program, and two epidural steroid injections with only temporary relief. MRI of the lumbar spine shows severe central canal stenosis at L4-L5 with grade 1 degenerative spondylolisthesis. Oswestry Disability Index 52%. Denies bowel or bladder dysfunction. Plan: L4-L5 decompression.`,
    criteria: (n) => [
      {
        label: "Neurogenic claudication or radiculopathy limiting function",
        status: /claudication|radicul/i.test(n) ? "met" : "missing",
        evidence: findSentence(n, /claudication/i),
      },
      weeksCrit(n, 12),
      {
        label: "Imaging: moderate-to-severe stenosis at the operative level",
        status: /(moderate|severe).{0,30}stenosis/i.test(n) ? "met" : "missing",
        evidence: findSentence(n, /MRI/i),
      },
      {
        label: "Functional impairment documented (ODI ≥ 30%)",
        status: (() => {
          const m = n.match(/Oswestry Disability Index (\d+)%/i);
          return m && Number(m[1]) >= 30 ? "met" : "missing";
        })(),
        evidence: findSentence(n, /Oswestry/i),
      },
      redFlagExceptionCrit(n),
    ],
    attachments: ["Surgical consult note", "MRI lumbar report", "Injection procedure notes ×2", "ODI questionnaire"],
  },
  {
    id: "PA-24815",
    patientId: "pt-1007",
    patient: "Carmen Diaz",
    payer: "Empire BCBS",
    payerType: "Commercial",
    cpt: "72148",
    service: "MRI lumbar spine without contrast",
    icd10: ["M54.31"],
    provider: "Daniel Park, PA-C",
    receivedDaysAgo: 2,
    urgency: "standard",
    pathway: "Payer portal / X12 278",
    note: `54-year-old female with sciatica: pain radiating down the right leg to the calf for 5 weeks. Started physical therapy 4 weeks ago and takes meloxicam; partial improvement. No weakness, normal reflexes. Denies bowel or bladder dysfunction, fever, or history of cancer. Request MRI of the lumbar spine to evaluate for disc herniation.`,
    criteria: (n) => [
      {
        label: "Low back pain / radiculopathy symptoms documented",
        status: /sciatica|radiat|radicul/i.test(n) ? "met" : "missing",
        evidence: findSentence(n, /sciatica|radiat/i),
      },
      weeksCrit(n, 6),
      redFlagExceptionCrit(n),
      {
        label: "No lumbar MRI in the previous 12 months",
        status: "met",
        evidence: "Checked imaging history across EMR + HIE query: none found.",
      },
    ],
    attachments: ["Office note 11/13/2026", "PT progress note (week 4)"],
  },
  {
    id: "PA-24816",
    patientId: "pt-1004",
    patient: "Robert Williams",
    payer: "NYSIF (Workers' Comp)",
    payerType: "Workers' Comp",
    cpt: "62321",
    service: "Cervical interlaminar ESI with imaging guidance (C6-C7)",
    icd10: ["M54.12"],
    provider: "Dr. Samuel Okafor, MD",
    receivedDaysAgo: 0,
    urgency: "standard",
    pathway: "NY WCB Medical Portal",
    note: `45-year-old male injured at work lifting a pallet 9 weeks ago. Neck pain with pain radiating into the right arm and numbness of the middle finger. Completed 6 weeks of physical therapy and NSAIDs. MRI of the cervical spine shows a right paracentral disc protrusion at C6-C7 with nerve root contact. Plan: C6-C7 interlaminar epidural steroid injection under fluoroscopy. Causally related to the work injury.`,
    criteria: (n) => [
      {
        label: "Radicular symptoms with corroborating imaging (NY Neck Injury MTG)",
        status: /radiat|radicul/i.test(n) && /MRI/i.test(n) ? "met" : "missing",
        evidence: findSentence(n, /MRI/i),
      },
      weeksCrit(n, 4),
      {
        label: "Causal relationship to the work injury documented",
        status: /causally related|work injury|injured at work/i.test(n) ? "met" : "missing",
        evidence: findSentence(n, /causally related/i),
      },
      {
        label: "Within Medical Treatment Guidelines → pre-authorized (no variance PAR needed)",
        status: "met",
        evidence: "Request is consistent with the MTG; only an MTG variance would require a PAR in the WCB Medical Portal.",
      },
    ],
    attachments: ["C-4 / doctor's initial report", "MRI cervical report", "PT notes"],
  },
];

export function evaluateCase(c: AuthCase) {
  const results = c.criteria(c.note);
  const applicable = results.filter((r) => r.status !== "na");
  const met = applicable.filter((r) => r.status === "met").length;
  const fixable = applicable.filter((r) => r.status === "missing" && r.fix?.startsWith("Auto-pulled")).length;
  const readiness = applicable.length ? Math.round((met / applicable.length) * 100) : 0;
  const readinessAfterFix = applicable.length ? Math.round(((met + fixable) / applicable.length) * 100) : 0;
  return { results, readiness, readinessAfterFix, blocking: applicable.filter((r) => r.status === "missing" && !r.fix?.startsWith("Auto-pulled")) };
}

export function medicalNecessityLetter(c: AuthCase, results: CriterionResult[]) {
  const met = results.filter((r) => r.status === "met");
  return `RE: Request for prior authorization — CPT ${c.cpt} (${c.service})
Patient: ${c.patient}   Payer: ${c.payer}   Dx: ${c.icd10.join(", ")}

To the Utilization Review Department,

I am requesting authorization for the following service: ${c.service} (CPT ${c.cpt}) for the patient above, who is under my care at Mateo Orthopedic & Spine Care.

Clinical summary:
${met.map((r) => `• ${r.label}: ${r.evidence.replace(/^“|”$/g, "")}`).join("\n")}

Based on these findings and the failure of appropriate conservative management, the requested service is medically necessary and consistent with your published coverage criteria. The supporting documentation is attached: ${c.attachments.join("; ")}.

Please contact our authorization desk at (516) 555-0148 for any peer-to-peer review.

Sincerely,
${c.provider}
[Draft generated by Prior Auth Copilot — requires provider review and signature]`;
}
