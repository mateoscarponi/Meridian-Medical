// Lightweight clinical NLP for spine/orthopedic notes: dictionary + pattern entity extraction,
// NegEx-style negation, duration normalization, red-flag detection and ICD-10 suggestion.
// Runs fully in the browser (no PHI leaves the device). In production this layer is backed by
// a transformer NER model / LLM with these rules kept as deterministic guardrails.

export type EntityType = "symptom" | "anatomy" | "duration" | "medication" | "therapy" | "imaging" | "redflag" | "procedure";

export interface Entity {
  type: EntityType;
  text: string;
  start: number;
  end: number;
  negated: boolean;
  normalized?: string;
}

export const ENTITY_LABELS: Record<EntityType, string> = {
  symptom: "Symptom",
  anatomy: "Anatomy / level",
  duration: "Duration",
  medication: "Medication",
  therapy: "Conservative care",
  imaging: "Imaging finding",
  redflag: "Red flag",
  procedure: "Procedure",
};

const DICT: Record<EntityType, (string | RegExp)[]> = {
  redflag: [
    /saddle (?:anesthesia|numbness)/i,
    /(?:bowel|bladder) (?:incontinence|dysfunction|retention)/i,
    /(?:loss of|losing) (?:bowel|bladder)(?: or (?:bowel|bladder))?(?: control)?/i,
    /urinary retention/i,
    /cauda equina/i,
    /progressive (?:weakness|neurologic(?:al)? deficit)/i,
    /foot drop/i,
    /unexplained weight loss/i,
    /(?:history of|hx of) (?:cancer|malignancy)/i,
    /fever(?: and chills)?/i,
    /night sweats/i,
    /myelopathy|myelopathic/i,
  ],
  symptom: [
    /(?:low(?:er)?|lumbar) back pain/i,
    /neck pain|cervicalgia/i,
    /radicular (?:pain|symptoms)/i,
    /radiculopathy/i,
    /sciatica/i,
    /numbness(?: and tingling)?/i,
    /tingling|paresthesias?/i,
    /weakness/i,
    /neurogenic claudication|claudication/i,
    /(?:pain|symptoms) radiat(?:es|ing) (?:down|into) the (?:right|left|bilateral)? ?(?:leg|arm|buttock|thigh|calf|foot|hand|shoulder)s?/i,
    /shooting pain/i,
    /gait (?:instability|disturbance)/i,
    /difficulty walking/i,
  ],
  anatomy: [
    /\b[CTL]\d{1,2}\s?[-–]\s?[CTLS]?\d{1}\b/,
    /\bL5[-–]S1\b/,
    /\b(?:right|left|bilateral) (?:L\d|S1|C\d) (?:nerve root|dermatome)\b/i,
    /lumbar spine|cervical spine|thoracic spine/i,
    /lateral recess|neural foramen|foraminal|central canal/i,
  ],
  duration: [
    /\b(?:\d+|one|two|three|four|five|six|seven|eight|nine|ten|twelve)\+? (?:days?|weeks?|months?|years?)\b/i,
  ],
  medication: [
    /\b(?:ibuprofen|naproxen|meloxicam|diclofenac|celecoxib|NSAIDs?)\b/i,
    /\b(?:gabapentin|pregabalin|cyclobenzaprine|methocarbamol|tizanidine)\b/i,
    /\b(?:methylprednisolone|medrol dosepak|prednisone)\b/i,
    /\b(?:tramadol|oxycodone|hydrocodone|acetaminophen)\b/i,
  ],
  therapy: [
    /physical therapy|\bPT\b/i,
    /home exercise program|\bHEP\b/i,
    /chiropractic(?: care| treatment)?/i,
    /activity modification/i,
    /acupuncture/i,
  ],
  imaging: [
    /disc (?:herniation|extrusion|protrusion|bulge)/i,
    /(?:paracentral|foraminal|central) (?:disc )?(?:herniation|extrusion|protrusion)/i,
    /(?:moderate|severe|mild) (?:central |foraminal |lateral recess )?(?:canal )?stenosis/i,
    /spinal stenosis/i,
    /nerve root (?:compression|impingement|contact)/i,
    /contacting the (?:descending|exiting) (?:right|left) [CL]\d nerve root/i,
    /spondylolisthesis/i,
    /\bMRI\b(?: of the (?:lumbar|cervical) spine)?/i,
    /\bX-?rays?\b/i,
    /\bEMG\b(?:\/NCS)?/i,
  ],
  procedure: [
    /(?:transforaminal|interlaminar|caudal)? ?epidural (?:steroid )?injections?|\bESI\b|\bTFESI\b/i,
    /medial branch blocks?|facet (?:injections?|blocks?)/i,
    /radiofrequency ablation|\bRFA\b/i,
    /\bACDF\b|anterior cervical discectomy(?: and fusion)?/i,
    /microdiscectomy|laminectomy|laminotomy|decompression/i,
    /spinal fusion|\bTLIF\b|\bPLIF\b/i,
  ],
};

const NEGATION_TRIGGERS = /\b(?:denies|denied|no|not|without|negative for|absence of|free of|resolved)\b/i;
const NEGATION_TERMINATORS = /\b(?:but|however|although|except|reports|presents|complains)\b|[.;:]/i;

const WORD_NUM: Record<string, number> = { one: 1, two: 2, three: 3, four: 4, five: 5, six: 6, seven: 7, eight: 8, nine: 9, ten: 10, twelve: 12 };

function normalizeDuration(text: string): string | undefined {
  const m = text.match(/(\d+|one|two|three|four|five|six|seven|eight|nine|ten|twelve)\+? (day|week|month|year)/i);
  if (!m) return undefined;
  const n = /^\d+$/.test(m[1]) ? Number(m[1]) : WORD_NUM[m[1].toLowerCase()];
  const unit = m[2].toLowerCase();
  const weeks = unit === "day" ? n / 7 : unit === "week" ? n : unit === "month" ? n * 4.345 : n * 52;
  return `${Math.round(weeks * 10) / 10} wk`;
}

export function extractEntities(text: string): Entity[] {
  const found: Entity[] = [];
  const order: EntityType[] = ["redflag", "procedure", "imaging", "symptom", "medication", "therapy", "anatomy", "duration"];
  for (const type of order) {
    for (const pat of DICT[type]) {
      const re = pat instanceof RegExp ? new RegExp(pat.source, pat.flags.includes("g") ? pat.flags : pat.flags + "g") : new RegExp(pat, "gi");
      let m: RegExpExecArray | null;
      while ((m = re.exec(text))) {
        if (!m[0].trim()) {
          re.lastIndex++;
          continue;
        }
        const start = m.index;
        const end = start + m[0].length;
        if (found.some((e) => start < e.end && end > e.start)) continue; // keep first (higher priority) match
        found.push({ type, text: m[0], start, end, negated: false, normalized: type === "duration" ? normalizeDuration(m[0]) : undefined });
      }
    }
  }
  // NegEx-lite: a trigger within the same clause and <= 10 tokens before the entity
  // (wide enough to cover lists such as "denies A, B, C, or D").
  for (const e of found) {
    if (e.type === "duration" || e.type === "anatomy") continue;
    const windowText = text.slice(Math.max(0, e.start - 90), e.start);
    const clause = windowText.split(NEGATION_TERMINATORS).pop() ?? "";
    const tokens = clause.trim().split(/\s+/).slice(-10).join(" ");
    if (NEGATION_TRIGGERS.test(tokens)) e.negated = true;
  }
  return found.sort((a, b) => a.start - b.start);
}

/** Total weeks of documented conservative therapy (PT/HEP/chiro/meds) — key prior-auth criterion. */
export function conservativeCareWeeks(text: string): { weeks: number; evidence: string[] } {
  const sentences = text.split(/(?<=[.!?])\s+/);
  let weeks = 0;
  const evidence: string[] = [];
  for (const s of sentences) {
    const hasTherapy = DICT.therapy.some((p) => (p instanceof RegExp ? p.test(s) : s.includes(p))) || DICT.medication.some((p) => (p instanceof RegExp ? p.test(s) : s.includes(p)));
    if (!hasTherapy) continue;
    const durs = [...s.matchAll(/(\d+|one|two|three|four|five|six|seven|eight|nine|ten|twelve)\+? (day|week|month|year)s?/gi)];
    if (!durs.length) continue;
    const w = Math.max(...durs.map((d) => parseFloat(normalizeDuration(d[0]) ?? "0")));
    if (w > 0) {
      weeks = Math.max(weeks, w);
      evidence.push(s.trim());
    }
  }
  return { weeks: Math.round(weeks * 10) / 10, evidence };
}

export interface IcdSuggestion {
  code: string;
  display: string;
  why: string;
  confidence: number;
}

const ICD_RULES: { test: RegExp; code: string; display: string; why: string }[] = [
  { test: /cauda equina/i, code: "G83.4", display: "Cauda equina syndrome", why: "Explicit mention of cauda equina" },
  { test: /(L4[-–]L5|L5[-–]S1|lumbar).{0,80}(herniation|extrusion|protrusion).{0,120}(radicul|nerve root|sciatica)|(radicul|sciatica).{0,160}(L4[-–]L5|L5[-–]S1).{0,60}(herniation|extrusion)/i, code: "M51.16", display: "Intervertebral disc disorders with radiculopathy, lumbar region", why: "Lumbar disc herniation documented with radicular findings" },
  { test: /stenosis.{0,120}claudication|claudication.{0,120}stenosis/i, code: "M48.062", display: "Spinal stenosis, lumbar region with neurogenic claudication", why: "Stenosis with neurogenic claudication" },
  { test: /C5[-–]C6.{0,120}(herniation|protrusion|disc).{0,120}radicul|radicul.{0,120}C5[-–]C6/i, code: "M50.122", display: "Cervical disc disorder at C5-C6 level with radiculopathy", why: "C5-C6 disc pathology with radiculopathy" },
  { test: /(cervical|arm|C\d).{0,80}radicul|radicul.{0,40}(cervical|arm)/i, code: "M54.12", display: "Radiculopathy, cervical region", why: "Cervical radicular symptoms" },
  { test: /(lumbar|leg|L\d|S1).{0,80}radicul|radicul.{0,60}(lumbar|leg)|sciatica/i, code: "M54.16", display: "Radiculopathy, lumbar region", why: "Lumbar radicular symptoms / sciatica" },
  { test: /myelopath/i, code: "M47.12", display: "Other spondylosis with myelopathy, cervical region", why: "Myelopathic findings documented" },
  { test: /(low(er)? back pain|lumbar back pain)/i, code: "M54.50", display: "Low back pain, unspecified", why: "Low back pain (M54.5 retired Oct 2021 — use M54.5x)" },
  { test: /neck pain|cervicalgia/i, code: "M54.2", display: "Cervicalgia", why: "Neck pain documented" },
  { test: /scoliosis/i, code: "M41.26", display: "Other idiopathic scoliosis, lumbar region", why: "Adult scoliosis documented" },
];

export function suggestIcd10(text: string, entities: Entity[]): IcdSuggestion[] {
  const negatedSpans = entities.filter((e) => e.negated).map((e) => e.text.toLowerCase());
  const out: IcdSuggestion[] = [];
  for (const r of ICD_RULES) {
    const m = text.match(r.test);
    if (!m) continue;
    if (negatedSpans.some((n) => m[0].toLowerCase().includes(n) && n.length > 6)) continue;
    if (out.some((o) => o.code === r.code)) continue;
    out.push({ code: r.code, display: r.display, why: r.why, confidence: Math.min(0.97, 0.7 + m[0].length / 600) });
  }
  // Specific codes supersede generic ones.
  const codes = new Set(out.map((o) => o.code));
  return out
    .filter((o) => !(o.code === "M54.16" && codes.has("M51.16")))
    .filter((o) => !(o.code === "M54.12" && codes.has("M50.122")))
    .filter((o) => !(o.code === "M54.50" && (codes.has("M51.16") || codes.has("M48.062"))))
    .slice(0, 4);
}

export interface RedFlagResult {
  active: Entity[];
  negated: Entity[];
  urgent: boolean;
}

export function redFlags(entities: Entity[]): RedFlagResult {
  const rf = entities.filter((e) => e.type === "redflag");
  const active = rf.filter((e) => !e.negated);
  return { active, negated: rf.filter((e) => e.negated), urgent: active.length > 0 };
}

export const SAMPLE_NOTE = `56-year-old male presents with low back pain and pain radiating down the right leg for 10 weeks after lifting at work. Reports numbness and tingling in the right foot and mild weakness with great toe extension. Completed 8 weeks of physical therapy with home exercise program and 6 weeks of naproxen without lasting relief; a Medrol Dosepak gave temporary improvement. Denies bowel or bladder incontinence, saddle anesthesia, fever, or unexplained weight loss. MRI of the lumbar spine shows a right paracentral disc extrusion at L4-L5 contacting the descending right L5 nerve root with moderate lateral recess stenosis. Exam: positive straight leg raise on the right, diminished sensation in the right L5 dermatome. Plan: right L4-L5 transforaminal epidural steroid injection; continue HEP; surgical consult if no improvement.`;
