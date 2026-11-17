// HIPAA Safe Harbor-style de-identification (subset of the 18 identifiers) for text.
// Used before sending text to analytics or to any model that is not covered by a BAA.

export interface PhiHit {
  type: string;
  text: string;
  start: number;
  end: number;
}

const PATTERNS: { type: string; re: RegExp }[] = [
  { type: "SSN", re: /\b\d{3}-\d{2}-\d{4}\b/g },
  { type: "MRN", re: /\b(?:MRN|MR#|Medical Record(?: Number)?)[:#\s]*[A-Z]{0,4}-?\d{5,10}\b|\bMOS-\d{6}\b/gi },
  { type: "MEMBER_ID", re: /\b(?:Member ID|Policy(?: No\.?| Number)?|Subscriber ID)[:#\s]*[A-Z0-9-]{6,16}\b/gi },
  { type: "EMAIL", re: /\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b/g },
  { type: "PHONE", re: /(?:\+1[\s.-]?)?\(?\b\d{3}\)?[\s.-]?\d{3}[\s.-]\d{4}\b/g },
  { type: "URL", re: /\bhttps?:\/\/\S+/gi },
  { type: "IP", re: /\b(?:\d{1,3}\.){3}\d{1,3}\b/g },
  { type: "DATE", re: /\b(?:0?[1-9]|1[0-2])[/-](?:0?[1-9]|[12]\d|3[01])[/-](?:19|20)\d{2}\b|\b(?:Jan(?:uary)?|Feb(?:ruary)?|Mar(?:ch)?|Apr(?:il)?|May|Jun(?:e)?|Jul(?:y)?|Aug(?:ust)?|Sep(?:tember)?|Oct(?:ober)?|Nov(?:ember)?|Dec(?:ember)?)\.? \d{1,2},? (?:19|20)\d{2}\b/g },
  { type: "ADDRESS", re: /\b\d{1,5} [A-Z][a-z]+(?: [A-Z][a-z]+)* (?:St|Street|Ave|Avenue|Rd|Road|Blvd|Boulevard|Ln|Lane|Dr|Drive|Hwy|Highway|Ct|Court|Pl|Place)\b\.?(?:,? (?:Apt|Suite|Ste)\.? ?\w+)?/g },
  { type: "ZIP", re: /\b\d{5}(?:-\d{4})?\b/g },
  { type: "NAME", re: /\b(?:Mr|Mrs|Ms|Miss|Dr)\.? [A-Z][a-z]+(?: [A-Z][a-z]+)?\b/g },
  { type: "NAME", re: /\b(?:Patient|Pt|Name)[:\s]+[A-Z][a-z]+(?: [A-Z]\.)?(?: [A-Z][a-z'-]+)\b/g },
  { type: "AGE_90+", re: /\b(?:9\d|1[01]\d)[- ]year[- ]old\b/gi },
];

export function findPhi(text: string): PhiHit[] {
  const hits: PhiHit[] = [];
  for (const { type, re } of PATTERNS) {
    const r = new RegExp(re.source, re.flags);
    let m: RegExpExecArray | null;
    while ((m = r.exec(text))) {
      const start = m.index;
      const end = start + m[0].length;
      if (hits.some((h) => start < h.end && end > h.start)) continue;
      hits.push({ type, text: m[0], start, end });
    }
  }
  return hits.sort((a, b) => a.start - b.start);
}

export function redact(text: string, hits = findPhi(text)) {
  let out = "";
  let last = 0;
  for (const h of hits) {
    out += text.slice(last, h.start) + `[${h.type}]`;
    last = h.end;
  }
  return out + text.slice(last);
}

export const SAMPLE_PHI_TEXT = `Patient: Maria L. Gonzalez, DOB 04/12/1968, MRN MOS-204871, phone (516) 555-0191, lives at 142 Fulton Ave, Hempstead NY 11550. Member ID FC88310442. Seen 11/16/2026 by Dr. Hale for lumbar radiculopathy. Prior PT at Meridian Motion PT. Email for portal invite: maria.g.demo@example.com. Daughter Ms. Carla Gonzalez will drive her to the injection on Nov 24, 2026.`;
