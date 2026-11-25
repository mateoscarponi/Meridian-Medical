//! HIPAA Safe Harbor-style de-identification (subset of the 18 identifiers) for text.
//! Used before sending text to analytics or to any model that is not covered by a BAA.
//! Patterns are compiled once (lazily) and reused, instead of being rebuilt on every call.

use regex_lite::Regex;
use std::sync::LazyLock;

#[derive(Clone, Debug, PartialEq)]
pub struct PhiHit {
    pub kind: &'static str,
    pub text: String,
    pub start: usize,
    pub end: usize,
}

static PATTERNS: LazyLock<Vec<(&'static str, Regex)>> = LazyLock::new(|| {
    let p = |k: &'static str, re: &str| (k, Regex::new(re).expect("valid PHI regex"));
    vec![
        p("SSN", r"\b\d{3}-\d{2}-\d{4}\b"),
        p("MRN", r"(?i)\b(?:MRN|MR#|Medical Record(?: Number)?)[:#\s]*[A-Z]{0,4}-?\d{5,10}\b|\bMOS-\d{6}\b"),
        p("MEMBER_ID", r"(?i)\b(?:Member ID|Policy(?: No\.?| Number)?|Subscriber ID)[:#\s]*[A-Z0-9-]{6,16}\b"),
        p("EMAIL", r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b"),
        p("PHONE", r"(?:\+1[\s.-]?)?\(?\b\d{3}\)?[\s.-]?\d{3}[\s.-]\d{4}\b"),
        p("URL", r"(?i)\bhttps?://\S+"),
        p("IP", r"\b(?:\d{1,3}\.){3}\d{1,3}\b"),
        p("DATE", r"\b(?:0?[1-9]|1[0-2])[/-](?:0?[1-9]|[12]\d|3[01])[/-](?:19|20)\d{2}\b|\b(?:Jan(?:uary)?|Feb(?:ruary)?|Mar(?:ch)?|Apr(?:il)?|May|Jun(?:e)?|Jul(?:y)?|Aug(?:ust)?|Sep(?:tember)?|Oct(?:ober)?|Nov(?:ember)?|Dec(?:ember)?)\.? \d{1,2},? (?:19|20)\d{2}\b"),
        p("ADDRESS", r"\b\d{1,5} [A-Z][a-z]+(?: [A-Z][a-z]+)* (?:St|Street|Ave|Avenue|Rd|Road|Blvd|Boulevard|Ln|Lane|Dr|Drive|Hwy|Highway|Ct|Court|Pl|Place)\b\.?(?:,? (?:Apt|Suite|Ste)\.? ?\w+)?"),
        p("ZIP", r"\b\d{5}(?:-\d{4})?\b"),
        p("NAME", r"\b(?:Mr|Mrs|Ms|Miss|Dr)\.? [A-Z][a-z]+(?: [A-Z][a-z]+)?\b"),
        p("NAME", r"\b(?:Patient|Pt|Name)[:\s]+[A-Z][a-z]+(?: [A-Z]\.)?(?: [A-Z][a-z'-]+)\b"),
        p("AGE_90+", r"(?i)\b(?:9\d|1[01]\d)[- ]year[- ]old\b"),
    ]
});

pub fn find_phi(text: &str) -> Vec<PhiHit> {
    let mut hits: Vec<PhiHit> = Vec::new();
    for (kind, re) in PATTERNS.iter() {
        for m in re.find_iter(text) {
            let (start, end) = (m.start(), m.end());
            if hits.iter().any(|h| start < h.end && end > h.start) {
                continue;
            }
            hits.push(PhiHit { kind, text: m.as_str().to_string(), start, end });
        }
    }
    hits.sort_by_key(|h| h.start);
    hits
}

pub fn redact_with(text: &str, hits: &[PhiHit]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for h in hits {
        out.push_str(&text[last..h.start]);
        out.push('[');
        out.push_str(h.kind);
        out.push(']');
        last = h.end;
    }
    out.push_str(&text[last..]);
    out
}

pub fn redact(text: &str) -> String {
    redact_with(text, &find_phi(text))
}

pub const SAMPLE_PHI_TEXT: &str = "Patient: Maria L. Gonzalez, DOB 04/12/1968, MRN MOS-204871, phone (516) 555-0191, lives at 142 Fulton Ave, Hempstead NY 11550. Member ID FC88310442. Seen 11/16/2026 by Dr. Hale for lumbar radiculopathy. Prior PT at Meridian Motion PT. Email for portal invite: maria.g.demo@example.com. Daughter Ms. Carla Gonzalez will drive her to the injection on Nov 24, 2026.";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn redacts_sample() {
        let r = redact(SAMPLE_PHI_TEXT);
        assert!(r.contains("[MRN]"));
        assert!(r.contains("[PHONE]"));
        assert!(r.contains("[EMAIL]"));
        assert!(r.contains("[DATE]"));
        assert!(!r.contains("555-0191"));
    }
}
