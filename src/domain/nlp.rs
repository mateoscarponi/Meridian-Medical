//! Lightweight clinical NLP for spine/orthopedic notes: dictionary + pattern entity extraction,
//! NegEx-style negation, duration normalization, red-flag detection and ICD-10 suggestion.
//! Runs fully in the browser as WebAssembly (no PHI leaves the device). In production this layer
//! is backed by a transformer NER model / LLM with these rules kept as deterministic guardrails.
//!
//! All regular expressions are compiled once and cached (the original rebuilt them per call).

use super::util::{floor_boundary, round1, split_sentences};
use regex_lite::Regex;
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EntityType {
    Symptom,
    Anatomy,
    Duration,
    Medication,
    Therapy,
    Imaging,
    Redflag,
    Procedure,
}

impl EntityType {
    pub const ALL: [EntityType; 8] = [
        EntityType::Symptom,
        EntityType::Anatomy,
        EntityType::Duration,
        EntityType::Medication,
        EntityType::Therapy,
        EntityType::Imaging,
        EntityType::Redflag,
        EntityType::Procedure,
    ];
    pub fn key(self) -> &'static str {
        match self {
            EntityType::Symptom => "symptom",
            EntityType::Anatomy => "anatomy",
            EntityType::Duration => "duration",
            EntityType::Medication => "medication",
            EntityType::Therapy => "therapy",
            EntityType::Imaging => "imaging",
            EntityType::Redflag => "redflag",
            EntityType::Procedure => "procedure",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            EntityType::Symptom => "Symptom",
            EntityType::Anatomy => "Anatomy / level",
            EntityType::Duration => "Duration",
            EntityType::Medication => "Medication",
            EntityType::Therapy => "Conservative care",
            EntityType::Imaging => "Imaging finding",
            EntityType::Redflag => "Red flag",
            EntityType::Procedure => "Procedure",
        }
    }
    /// CSS swatch class (`ent-<type>`).
    pub fn swatch(self) -> &'static str {
        match self {
            EntityType::Symptom => "ent-symptom",
            EntityType::Anatomy => "ent-anatomy",
            EntityType::Duration => "ent-duration",
            EntityType::Medication => "ent-medication",
            EntityType::Therapy => "ent-therapy",
            EntityType::Imaging => "ent-imaging",
            EntityType::Redflag => "ent-redflag",
            EntityType::Procedure => "ent-procedure",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entity {
    pub kind: EntityType,
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub negated: bool,
    pub normalized: Option<String>,
}

fn re(s: &str) -> Regex {
    Regex::new(s).expect("valid NLP regex")
}

struct Dict {
    redflag: Vec<Regex>,
    symptom: Vec<Regex>,
    anatomy: Vec<Regex>,
    duration: Vec<Regex>,
    medication: Vec<Regex>,
    therapy: Vec<Regex>,
    imaging: Vec<Regex>,
    procedure: Vec<Regex>,
}

impl Dict {
    fn get(&self, t: EntityType) -> &[Regex] {
        match t {
            EntityType::Redflag => &self.redflag,
            EntityType::Symptom => &self.symptom,
            EntityType::Anatomy => &self.anatomy,
            EntityType::Duration => &self.duration,
            EntityType::Medication => &self.medication,
            EntityType::Therapy => &self.therapy,
            EntityType::Imaging => &self.imaging,
            EntityType::Procedure => &self.procedure,
        }
    }
}

static DICT: LazyLock<Dict> = LazyLock::new(|| Dict {
    redflag: [
        r"(?i)saddle (?:anesthesia|numbness)",
        r"(?i)(?:bowel|bladder) (?:incontinence|dysfunction|retention)",
        r"(?i)(?:loss of|losing) (?:bowel|bladder)(?: or (?:bowel|bladder))?(?: control)?",
        r"(?i)urinary retention",
        r"(?i)cauda equina",
        r"(?i)progressive (?:weakness|neurologic(?:al)? deficit)",
        r"(?i)foot drop",
        r"(?i)unexplained weight loss",
        r"(?i)(?:history of|hx of) (?:cancer|malignancy)",
        r"(?i)fever(?: and chills)?",
        r"(?i)night sweats",
        r"(?i)myelopathy|myelopathic",
    ]
    .iter()
    .map(|s| re(s))
    .collect(),
    symptom: [
        r"(?i)(?:low(?:er)?|lumbar) back pain",
        r"(?i)neck pain|cervicalgia",
        r"(?i)radicular (?:pain|symptoms)",
        r"(?i)radiculopathy",
        r"(?i)sciatica",
        r"(?i)numbness(?: and tingling)?",
        r"(?i)tingling|paresthesias?",
        r"(?i)weakness",
        r"(?i)neurogenic claudication|claudication",
        r"(?i)(?:pain|symptoms) radiat(?:es|ing) (?:down|into) the (?:right|left|bilateral)? ?(?:leg|arm|buttock|thigh|calf|foot|hand|shoulder)s?",
        r"(?i)shooting pain",
        r"(?i)gait (?:instability|disturbance)",
        r"(?i)difficulty walking",
    ]
    .iter()
    .map(|s| re(s))
    .collect(),
    anatomy: [
        r"\b[CTL]\d{1,2}\s?[-–]\s?[CTLS]?\d{1}\b",
        r"\bL5[-–]S1\b",
        r"(?i)\b(?:right|left|bilateral) (?:L\d|S1|C\d) (?:nerve root|dermatome)\b",
        r"(?i)lumbar spine|cervical spine|thoracic spine",
        r"(?i)lateral recess|neural foramen|foraminal|central canal",
    ]
    .iter()
    .map(|s| re(s))
    .collect(),
    duration: vec![re(r"(?i)\b(?:\d+|one|two|three|four|five|six|seven|eight|nine|ten|twelve)\+? (?:days?|weeks?|months?|years?)\b")],
    medication: [
        r"(?i)\b(?:ibuprofen|naproxen|meloxicam|diclofenac|celecoxib|NSAIDs?)\b",
        r"(?i)\b(?:gabapentin|pregabalin|cyclobenzaprine|methocarbamol|tizanidine)\b",
        r"(?i)\b(?:methylprednisolone|medrol dosepak|prednisone)\b",
        r"(?i)\b(?:tramadol|oxycodone|hydrocodone|acetaminophen)\b",
    ]
    .iter()
    .map(|s| re(s))
    .collect(),
    therapy: [
        r"(?i)physical therapy|\bPT\b",
        r"(?i)home exercise program|\bHEP\b",
        r"(?i)chiropractic(?: care| treatment)?",
        r"(?i)activity modification",
        r"(?i)acupuncture",
    ]
    .iter()
    .map(|s| re(s))
    .collect(),
    imaging: [
        r"(?i)disc (?:herniation|extrusion|protrusion|bulge)",
        r"(?i)(?:paracentral|foraminal|central) (?:disc )?(?:herniation|extrusion|protrusion)",
        r"(?i)(?:moderate|severe|mild) (?:central |foraminal |lateral recess )?(?:canal )?stenosis",
        r"(?i)spinal stenosis",
        r"(?i)nerve root (?:compression|impingement|contact)",
        r"(?i)contacting the (?:descending|exiting) (?:right|left) [CL]\d nerve root",
        r"(?i)spondylolisthesis",
        r"(?i)\bMRI\b(?: of the (?:lumbar|cervical) spine)?",
        r"(?i)\bX-?rays?\b",
        r"(?i)\bEMG\b(?:/NCS)?",
    ]
    .iter()
    .map(|s| re(s))
    .collect(),
    procedure: [
        r"(?i)(?:transforaminal|interlaminar|caudal)? ?epidural (?:steroid )?injections?|\bESI\b|\bTFESI\b",
        r"(?i)medial branch blocks?|facet (?:injections?|blocks?)",
        r"(?i)radiofrequency ablation|\bRFA\b",
        r"(?i)\bACDF\b|anterior cervical discectomy(?: and fusion)?",
        r"(?i)microdiscectomy|laminectomy|laminotomy|decompression",
        r"(?i)spinal fusion|\bTLIF\b|\bPLIF\b",
    ]
    .iter()
    .map(|s| re(s))
    .collect(),
});

static NEGATION_TRIGGERS: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?i)\b(?:denies|denied|no|not|without|negative for|absence of|free of|resolved)\b"));
static NEGATION_TERMINATORS: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?i)\b(?:but|however|although|except|reports|presents|complains)\b|[.;:]"));
static DURATION_NORM: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?i)(\d+|one|two|three|four|five|six|seven|eight|nine|ten|twelve)\+? (day|week|month|year)"));
static DURATION_ALL: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?i)(\d+|one|two|three|four|five|six|seven|eight|nine|ten|twelve)\+? (day|week|month|year)s?"));
static WS: LazyLock<Regex> = LazyLock::new(|| re(r"\s+"));

fn word_num(w: &str) -> f64 {
    match w {
        "one" => 1.0,
        "two" => 2.0,
        "three" => 3.0,
        "four" => 4.0,
        "five" => 5.0,
        "six" => 6.0,
        "seven" => 7.0,
        "eight" => 8.0,
        "nine" => 9.0,
        "ten" => 10.0,
        "twelve" => 12.0,
        _ => f64::NAN,
    }
}

/// Weeks as a number (e.g. 10 weeks -> 10, 5 months -> 21.7).
fn duration_weeks(text: &str) -> Option<f64> {
    let m = DURATION_NORM.captures(text)?;
    let n_raw = m.get(1)?.as_str();
    let n = if n_raw.bytes().all(|b| b.is_ascii_digit()) { n_raw.parse::<f64>().ok()? } else { word_num(&n_raw.to_lowercase()) };
    let unit = m.get(2)?.as_str().to_lowercase();
    let weeks = match unit.as_str() {
        "day" => n / 7.0,
        "week" => n,
        "month" => n * 4.345,
        _ => n * 52.0,
    };
    Some(round1(weeks))
}

fn normalize_duration(text: &str) -> Option<String> {
    duration_weeks(text).map(|w| format!("{} wk", super::util::num(w)))
}

const ORDER: [EntityType; 8] = [
    EntityType::Redflag,
    EntityType::Procedure,
    EntityType::Imaging,
    EntityType::Symptom,
    EntityType::Medication,
    EntityType::Therapy,
    EntityType::Anatomy,
    EntityType::Duration,
];

pub fn extract_entities(text: &str) -> Vec<Entity> {
    let mut found: Vec<Entity> = Vec::new();
    for kind in ORDER {
        for pat in DICT.get(kind) {
            for m in pat.find_iter(text) {
                if m.as_str().trim().is_empty() {
                    continue;
                }
                let (start, end) = (m.start(), m.end());
                if found.iter().any(|e| start < e.end && end > e.start) {
                    continue; // keep first (higher priority) match
                }
                found.push(Entity {
                    kind,
                    text: m.as_str().to_string(),
                    start,
                    end,
                    negated: false,
                    normalized: if kind == EntityType::Duration { normalize_duration(m.as_str()) } else { None },
                });
            }
        }
    }
    // NegEx-lite: a trigger within the same clause and <= 10 tokens before the entity
    // (wide enough to cover lists such as "denies A, B, C, or D").
    for e in found.iter_mut() {
        if matches!(e.kind, EntityType::Duration | EntityType::Anatomy) {
            continue;
        }
        let from = floor_boundary(text, e.start.saturating_sub(90));
        let window = &text[from..e.start];
        let clause = NEGATION_TERMINATORS.split(window).last().unwrap_or("");
        let toks: Vec<&str> = WS.split(clause.trim()).collect();
        let tail = toks[toks.len().saturating_sub(10)..].join(" ");
        if NEGATION_TRIGGERS.is_match(&tail) {
            e.negated = true;
        }
    }
    found.sort_by_key(|e| e.start);
    found
}

/// Total weeks of documented conservative therapy (PT/HEP/chiro/meds) — key prior-auth criterion.
pub fn conservative_care_weeks(text: &str) -> (f64, Vec<String>) {
    let mut weeks: f64 = 0.0;
    let mut evidence = Vec::new();
    for s in split_sentences(text) {
        let has_therapy = DICT.therapy.iter().any(|p| p.is_match(s)) || DICT.medication.iter().any(|p| p.is_match(s));
        if !has_therapy {
            continue;
        }
        let w = DURATION_ALL.find_iter(s).filter_map(|d| duration_weeks(d.as_str())).fold(f64::NEG_INFINITY, f64::max);
        if w == f64::NEG_INFINITY {
            continue;
        }
        if w > 0.0 {
            weeks = weeks.max(w);
            evidence.push(s.trim().to_string());
        }
    }
    (round1(weeks), evidence)
}

#[derive(Clone, Debug, PartialEq)]
pub struct IcdSuggestion {
    pub code: &'static str,
    pub display: &'static str,
    pub why: &'static str,
    pub confidence: f64,
}

struct IcdRule {
    test: Regex,
    code: &'static str,
    display: &'static str,
    why: &'static str,
}

static ICD_RULES: LazyLock<Vec<IcdRule>> = LazyLock::new(|| {
    let r = |t: &str, code, display, why| IcdRule { test: re(t), code, display, why };
    vec![
        r(r"(?i)cauda equina", "G83.4", "Cauda equina syndrome", "Explicit mention of cauda equina"),
        r(
            r"(?i)(L4[-–]L5|L5[-–]S1|lumbar).{0,80}(herniation|extrusion|protrusion).{0,120}(radicul|nerve root|sciatica)|(radicul|sciatica).{0,160}(L4[-–]L5|L5[-–]S1).{0,60}(herniation|extrusion)",
            "M51.16",
            "Intervertebral disc disorders with radiculopathy, lumbar region",
            "Lumbar disc herniation documented with radicular findings",
        ),
        r(r"(?i)stenosis.{0,120}claudication|claudication.{0,120}stenosis", "M48.062", "Spinal stenosis, lumbar region with neurogenic claudication", "Stenosis with neurogenic claudication"),
        r(r"(?i)C5[-–]C6.{0,120}(herniation|protrusion|disc).{0,120}radicul|radicul.{0,120}C5[-–]C6", "M50.122", "Cervical disc disorder at C5-C6 level with radiculopathy", "C5-C6 disc pathology with radiculopathy"),
        r(r"(?i)(cervical|arm|C\d).{0,80}radicul|radicul.{0,40}(cervical|arm)", "M54.12", "Radiculopathy, cervical region", "Cervical radicular symptoms"),
        r(r"(?i)(lumbar|leg|L\d|S1).{0,80}radicul|radicul.{0,60}(lumbar|leg)|sciatica", "M54.16", "Radiculopathy, lumbar region", "Lumbar radicular symptoms / sciatica"),
        r(r"(?i)myelopath", "M47.12", "Other spondylosis with myelopathy, cervical region", "Myelopathic findings documented"),
        r(r"(?i)(low(er)? back pain|lumbar back pain)", "M54.50", "Low back pain, unspecified", "Low back pain (M54.5 retired Oct 2021 — use M54.5x)"),
        r(r"(?i)neck pain|cervicalgia", "M54.2", "Cervicalgia", "Neck pain documented"),
        r(r"(?i)scoliosis", "M41.26", "Other idiopathic scoliosis, lumbar region", "Adult scoliosis documented"),
    ]
});

pub fn suggest_icd10(text: &str, entities: &[Entity]) -> Vec<IcdSuggestion> {
    let negated: Vec<String> = entities.iter().filter(|e| e.negated).map(|e| e.text.to_lowercase()).collect();
    let mut out: Vec<IcdSuggestion> = Vec::new();
    for r in ICD_RULES.iter() {
        let Some(m) = r.test.find(text) else { continue };
        let m_lower = m.as_str().to_lowercase();
        if negated.iter().any(|n| m_lower.contains(n.as_str()) && n.chars().count() > 6) {
            continue;
        }
        if out.iter().any(|o| o.code == r.code) {
            continue;
        }
        let len = m.as_str().chars().count() as f64;
        out.push(IcdSuggestion { code: r.code, display: r.display, why: r.why, confidence: (0.7 + len / 600.0).min(0.97) });
    }
    // Specific codes supersede generic ones.
    let has = |c: &str| out.iter().any(|o| o.code == c);
    let (m5116, m50122, m48062) = (has("M51.16"), has("M50.122"), has("M48.062"));
    out.into_iter()
        .filter(|o| !(o.code == "M54.16" && m5116))
        .filter(|o| !(o.code == "M54.12" && m50122))
        .filter(|o| !(o.code == "M54.50" && (m5116 || m48062)))
        .take(4)
        .collect()
}

pub struct RedFlagResult {
    pub active: Vec<Entity>,
    pub negated: Vec<Entity>,
    pub urgent: bool,
}

pub fn red_flags(entities: &[Entity]) -> RedFlagResult {
    let rf: Vec<&Entity> = entities.iter().filter(|e| e.kind == EntityType::Redflag).collect();
    let active: Vec<Entity> = rf.iter().filter(|e| !e.negated).map(|e| (*e).clone()).collect();
    let negated: Vec<Entity> = rf.iter().filter(|e| e.negated).map(|e| (*e).clone()).collect();
    let urgent = !active.is_empty();
    RedFlagResult { active, negated, urgent }
}

pub const SAMPLE_NOTE: &str = "56-year-old male presents with low back pain and pain radiating down the right leg for 10 weeks after lifting at work. Reports numbness and tingling in the right foot and mild weakness with great toe extension. Completed 8 weeks of physical therapy with home exercise program and 6 weeks of naproxen without lasting relief; a Medrol Dosepak gave temporary improvement. Denies bowel or bladder incontinence, saddle anesthesia, fever, or unexplained weight loss. MRI of the lumbar spine shows a right paracentral disc extrusion at L4-L5 contacting the descending right L5 nerve root with moderate lateral recess stenosis. Exam: positive straight leg raise on the right, diminished sensation in the right L5 dermatome. Plan: right L4-L5 transforaminal epidural steroid injection; continue HEP; surgical consult if no improvement.";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sample_note() {
        let ents = extract_entities(SAMPLE_NOTE);
        assert!(ents.iter().any(|e| e.kind == EntityType::Redflag && e.negated));
        let (w, ev) = conservative_care_weeks(SAMPLE_NOTE);
        assert_eq!(w, 8.0);
        assert_eq!(ev.len(), 1);
        let icd = suggest_icd10(SAMPLE_NOTE, &ents);
        assert_eq!(icd[0].code, "M51.16");
        assert!(!red_flags(&ents).urgent);
    }
}
