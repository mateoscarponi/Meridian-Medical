//! Parity harness: dumps the outputs of the Rust domain layer for fixed inputs so they can be
//! compared with the original TypeScript implementation (scripts/parity-ts.mjs).
//!   cargo test --release --features ssr --test parity -- --nocapture
use meridian_medical::domain::{hl7, nlp, phi, prior_auth};
use serde_json::{json, Value};

pub fn cases() -> Vec<(&'static str, String)> {
    let mut v = vec![
        ("sample_note", nlp::SAMPLE_NOTE.to_string()),
        ("urgent", "62-year-old female 3 weeks after L4-L5 microdiscectomy calls with new saddle anesthesia and urinary retention since this morning, with progressive weakness in both legs. Reports low back pain 8/10. No fever. Taking gabapentin and acetaminophen.".to_string()),
        ("cervical", "Patient with neck pain and pain radiating into the left arm for 4 months, numbness in the thumb. Completed 12 weeks of physical therapy and 8 weeks of gabapentin. MRI of the cervical spine shows a left foraminal disc herniation at C5-C6 with nerve root compression. No myelopathy. Considering ACDF.".to_string()),
    ];
    for c in prior_auth::AUTH_CASES.iter() {
        v.push((c.id, c.note.to_string()));
    }
    v
}

fn ent(e: &nlp::Entity) -> Value {
    json!({ "type": e.kind.key(), "text": e.text, "start": e.start, "end": e.end, "negated": e.negated, "normalized": e.normalized })
}

#[test]
fn dump() {
    let mut out = serde_json::Map::new();
    for (name, text) in cases() {
        let ents = nlp::extract_entities(&text);
        let (weeks, evidence) = nlp::conservative_care_weeks(&text);
        let icd = nlp::suggest_icd10(&text, &ents);
        let rf = nlp::red_flags(&ents);
        out.insert(name.into(), json!({
            "entities": ents.iter().map(ent).collect::<Vec<_>>(),
            "weeks": weeks, "evidence": evidence,
            "icd": icd.iter().map(|i| json!({ "code": i.code, "confidence": (i.confidence * 1e6).round() / 1e6 })).collect::<Vec<_>>(),
            "urgent": rf.urgent,
        }));
    }
    let hits = phi::find_phi(phi::SAMPLE_PHI_TEXT);
    out.insert("phi".into(), json!({
        "hits": hits.iter().map(|h| json!({ "type": h.kind, "text": h.text, "start": h.start, "end": h.end })).collect::<Vec<_>>(),
        "redacted": phi::redact(phi::SAMPLE_PHI_TEXT),
    }));
    for s in hl7::SAMPLE_MESSAGES.iter() {
        let m = hl7::parse_hl7(s.text);
        out.insert(format!("hl7_{}", s.id), json!({ "type": m.kind, "controlId": m.control_id, "fields": m.segments.iter().map(|g| g.fields.len()).collect::<Vec<_>>(), "fhir": hl7::hl7_to_fhir(&m) }));
    }
    for c in prior_auth::AUTH_CASES.iter() {
        let e = prior_auth::evaluate_case(c);
        out.insert(format!("pa_{}", c.id), json!({
            "readiness": e.readiness, "after": e.readiness_after_fix,
            "results": e.results.iter().map(|r| json!({ "label": r.label, "status": format!("{:?}", r.status).to_lowercase(), "evidence": r.evidence })).collect::<Vec<_>>(),
            "letter": prior_auth::medical_necessity_letter(c, &e.results),
        }));
    }
    out.insert("scribe_soap".into(), json!(meridian_medical::pages::platform::scribe::local_soap(meridian_medical::pages::platform::scribe::TRANSCRIPT)));
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/target/parity-rust.json");
    std::fs::write(path, serde_json::to_string_pretty(&Value::Object(out)).unwrap()).unwrap();
}
