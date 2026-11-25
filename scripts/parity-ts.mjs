// Runs the ORIGINAL TypeScript domain code (Next.js project) on the same inputs as tests/parity.rs.
//   node scripts/parity-ts.mjs [path/to/original/project]
import path from "node:path";
import fs from "node:fs";
import { pathToFileURL, fileURLToPath } from "node:url";
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const orig = path.resolve(process.argv[2] ?? path.join(root, "..", "Mateo Orthopedic and Spine Care"));
const imp = (p) => import(pathToFileURL(path.join(orig, p)).href);
const nlp = await imp("lib/nlp.ts");
const phi = await imp("lib/phi.ts");
const hl7 = await imp("lib/hl7.ts");
// prior-auth.ts imports "../nlp" without an extension; load a patched copy.
const paSrc = fs.readFileSync(path.join(orig, "lib/data/prior-auth.ts"), "utf8").replace('from "../nlp"', `from ${JSON.stringify(pathToFileURL(path.join(orig, "lib/nlp.ts")).href)}`);
const tmp = path.join(root, "target", "prior-auth.parity.ts");
fs.writeFileSync(tmp, paSrc);
const pa = await import(pathToFileURL(tmp).href);

const cases = [
  ["sample_note", nlp.SAMPLE_NOTE],
  ["urgent", "62-year-old female 3 weeks after L4-L5 microdiscectomy calls with new saddle anesthesia and urinary retention since this morning, with progressive weakness in both legs. Reports low back pain 8/10. No fever. Taking gabapentin and acetaminophen."],
  ["cervical", "Patient with neck pain and pain radiating into the left arm for 4 months, numbness in the thumb. Completed 12 weeks of physical therapy and 8 weeks of gabapentin. MRI of the cervical spine shows a left foraminal disc herniation at C5-C6 with nerve root compression. No myelopathy. Considering ACDF."],
  ...pa.AUTH_CASES.map((c) => [c.id, c.note]),
];
const out = {};
for (const [name, text] of cases) {
  const ents = nlp.extractEntities(text);
  const cc = nlp.conservativeCareWeeks(text);
  out[name] = {
    entities: ents.map((e) => ({ type: e.type, text: e.text, start: e.start, end: e.end, negated: e.negated, normalized: e.normalized ?? null })),
    weeks: cc.weeks, evidence: cc.evidence,
    icd: nlp.suggestIcd10(text, ents).map((i) => ({ code: i.code, confidence: Math.round(i.confidence * 1e6) / 1e6 })),
    urgent: nlp.redFlags(ents).urgent,
  };
}
const hits = phi.findPhi(phi.SAMPLE_PHI_TEXT);
out.phi = { hits: hits.map((h) => ({ type: h.type, text: h.text, start: h.start, end: h.end })), redacted: phi.redact(phi.SAMPLE_PHI_TEXT) };
for (const s of hl7.SAMPLE_MESSAGES) {
  const m = hl7.parseHl7(s.text);
  out[`hl7_${s.id}`] = { type: m.type, controlId: m.controlId, fields: m.segments.map((g) => g.fields.length), fhir: hl7.hl7ToFhir(m) };
}
for (const c of pa.AUTH_CASES) {
  const e = pa.evaluateCase(c);
  out[`pa_${c.id}`] = { readiness: e.readiness, after: e.readinessAfterFix, results: e.results.map((r) => ({ label: r.label, status: r.status, evidence: r.evidence })), letter: pa.medicalNecessityLetter(c, e.results) };
}
// localSoap + TRANSCRIPT straight from the original Clinical Scribe page.
const page = fs.readFileSync(path.join(orig, "app/platform/scribe/page.tsx"), "utf8");
const tr = page.slice(page.indexOf("const TRANSCRIPT"), page.indexOf("function localSoap"));
const fn = page.slice(page.indexOf("function localSoap"), page.indexOf("export default function"));
const soapMod = `import { conservativeCareWeeks, extractEntities, redFlags, suggestIcd10 } from ${JSON.stringify(pathToFileURL(path.join(orig, "lib/nlp.ts")).href)};
${tr}
export ${fn}
export { TRANSCRIPT };`;
const soapTmp = path.join(root, "target", "scribe.parity.ts");
fs.writeFileSync(soapTmp, soapMod);
const soap = await import(pathToFileURL(soapTmp).href);
out.scribe_soap = soap.localSoap(soap.TRANSCRIPT);
fs.writeFileSync(path.join(root, "target", "parity-ts.json"), JSON.stringify(out, null, 2));
console.log("wrote target/parity-ts.json");
