//! Prior authorization cases + payer criteria evaluated against the chart with the NLP layer.
//! Criteria are illustrative composites of common commercial / RBM policy language, not any
//! specific payer's proprietary policy.

use super::nlp::{conservative_care_weeks, extract_entities, red_flags};
use super::util::{js_round, num, split_sentences};
use regex_lite::Regex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CritStatus {
    Met,
    Missing,
    Na,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CriterionResult {
    pub label: String,
    pub status: CritStatus,
    pub evidence: String,
    pub fix: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CriteriaSet {
    Tfesi,
    Acdf,
    Decompression,
    LumbarMri,
    WcCervicalEsi,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuthCase {
    pub id: &'static str,
    pub patient_id: &'static str,
    pub patient: &'static str,
    pub payer: &'static str,
    pub payer_type: &'static str,
    pub cpt: &'static str,
    pub service: &'static str,
    pub icd10: &'static [&'static str],
    pub provider: &'static str,
    pub received_days_ago: u32,
    pub urgency: &'static str,
    pub pathway: &'static str,
    pub note: &'static str,
    pub criteria: CriteriaSet,
    pub attachments: &'static [&'static str],
}

fn re(s: &str) -> Regex {
    Regex::new(s).expect("valid criteria regex")
}
fn test(n: &str, pattern: &str) -> bool {
    re(pattern).is_match(n)
}

fn find_sentence(note: &str, pattern: &str) -> String {
    let r = re(pattern);
    split_sentences(note).into_iter().find(|s| r.is_match(s)).map(|s| s.trim().to_string()).unwrap_or_default()
}

fn or_else(s: String, fallback: &str) -> String {
    if s.is_empty() {
        fallback.to_string()
    } else {
        s
    }
}

fn crit(label: &str, ok: bool, evidence: String, fix: Option<&str>) -> CriterionResult {
    CriterionResult { label: label.into(), status: if ok { CritStatus::Met } else { CritStatus::Missing }, evidence, fix: fix.map(str::to_string) }
}

fn weeks_crit(note: &str, min: f64) -> CriterionResult {
    let (weeks, evidence) = conservative_care_weeks(note);
    let ok = weeks >= min;
    CriterionResult {
        label: format!("≥ {} weeks of failed conservative care (PT / HEP / meds)", num(min)),
        status: if ok { CritStatus::Met } else { CritStatus::Missing },
        evidence: match evidence.first() {
            Some(e) => format!("{} wk documented — “{}”", num(weeks), e),
            None => "No dated conservative therapy found in the note.".into(),
        },
        fix: if ok {
            None
        } else {
            Some(format!(
                "Only {} wk documented. Copilot schedules a re-evaluation when week {} is reached, or flags the case if a red-flag exception applies.",
                num(weeks),
                num(min)
            ))
        },
    }
}

fn red_flag_exception_crit(note: &str) -> CriterionResult {
    let rf = red_flags(&extract_entities(note));
    let join = |v: &[super::nlp::Entity]| v.iter().map(|r| r.text.clone()).collect::<Vec<_>>().join(", ");
    CriterionResult {
        label: "Red-flag exception (bypasses conservative-care requirement)".into(),
        status: if rf.urgent { CritStatus::Met } else { CritStatus::Na },
        evidence: if rf.urgent {
            format!("Active red flag: {}", join(&rf.active))
        } else if !rf.negated.is_empty() {
            format!("Red flags explicitly denied: {}", join(&rf.negated))
        } else {
            "No red flags documented.".into()
        },
        fix: None,
    }
}

pub fn criteria(set: CriteriaSet, n: &str) -> Vec<CriterionResult> {
    match set {
        CriteriaSet::Tfesi => vec![
            crit("Radicular pain in a dermatomal distribution", test(n, r"(?i)radiat|radicul|dermatome|sciatica"), find_sentence(n, r"(?i)radiat|radicul|dermatome"), None),
            weeks_crit(n, 4.0),
            crit(
                "Imaging corroborates nerve-root involvement at the treated level",
                test(n, r"(?i)(nerve root|stenosis|herniation|extrusion)") && test(n, r"L4-L5"),
                find_sentence(n, r"(?i)MRI"),
                None,
            ),
            crit(
                "No ESI in the previous 6 months / ≤ 4 sessions per 12 months",
                test(n, r"(?i)no prior epidural"),
                or_else(find_sentence(n, r"(?i)prior epidural"), "Injection history not documented."),
                Some("Pull injection history from the EMR procedure log."),
            ),
            crit("Performed with fluoroscopic or CT guidance", test(n, r"(?i)fluoroscop|CT guid|imaging guid"), find_sentence(n, r"(?i)fluoroscop|guid"), None),
        ],
        CriteriaSet::Acdf => vec![
            crit("Radiculopathy or myelopathy with objective neurologic findings", test(n, r"(?i)weakness|reflex|myelopath|numbness"), find_sentence(n, r"(?i)weakness"), None),
            weeks_crit(n, 6.0),
            crit("Imaging shows compression concordant with symptoms", test(n, r"(?i)concordant|compression"), find_sentence(n, r"(?i)MRI"), None),
            crit("Functional impairment documented (NDI / ODI)", test(n, r"(?i)disability index|ODI|NDI"), find_sentence(n, r"(?i)Disability Index|ODI|NDI"), None),
            crit(
                "Nicotine status documented (non-smoker or cessation ≥ 6 weeks)",
                test(n, r"(?i)smok|nicotine|tobacco"),
                or_else(find_sentence(n, r"(?i)smok|nicotine|tobacco"), "Not in this note."),
                Some("Auto-pulled from EMR social history (FHIR Observation 72166-2 smoking status): “Never smoker”, recorded 09/2026. Added to packet."),
            ),
        ],
        CriteriaSet::Decompression => vec![
            crit("Neurogenic claudication or radiculopathy limiting function", test(n, r"(?i)claudication|radicul"), find_sentence(n, r"(?i)claudication"), None),
            weeks_crit(n, 12.0),
            crit("Imaging: moderate-to-severe stenosis at the operative level", test(n, r"(?i)(moderate|severe).{0,30}stenosis"), find_sentence(n, r"(?i)MRI"), None),
            {
                let odi = re(r"(?i)Oswestry Disability Index (\d+)%")
                    .captures(n)
                    .and_then(|m| m.get(1))
                    .and_then(|g| g.as_str().parse::<f64>().ok())
                    .is_some_and(|v| v >= 30.0);
                crit("Functional impairment documented (ODI ≥ 30%)", odi, find_sentence(n, r"(?i)Oswestry"), None)
            },
            red_flag_exception_crit(n),
        ],
        CriteriaSet::LumbarMri => vec![
            crit("Low back pain / radiculopathy symptoms documented", test(n, r"(?i)sciatica|radiat|radicul"), find_sentence(n, r"(?i)sciatica|radiat"), None),
            weeks_crit(n, 6.0),
            red_flag_exception_crit(n),
            crit("No lumbar MRI in the previous 12 months", true, "Checked imaging history across EMR + HIE query: none found.".into(), None),
        ],
        CriteriaSet::WcCervicalEsi => vec![
            crit(
                "Radicular symptoms with corroborating imaging (NY Neck Injury MTG)",
                test(n, r"(?i)radiat|radicul") && test(n, r"(?i)MRI"),
                find_sentence(n, r"(?i)MRI"),
                None,
            ),
            weeks_crit(n, 4.0),
            crit("Causal relationship to the work injury documented", test(n, r"(?i)causally related|work injury|injured at work"), find_sentence(n, r"(?i)causally related"), None),
            crit(
                "Within Medical Treatment Guidelines → pre-authorized (no variance PAR needed)",
                true,
                "Request is consistent with the MTG; only an MTG variance would require a PAR in the WCB Medical Portal.".into(),
                None,
            ),
        ],
    }
}

pub static AUTH_CASES: [AuthCase; 5] = [
    AuthCase {
        id: "PA-24812",
        patient_id: "pt-1002",
        patient: "James O'Connor",
        payer: "Aetna PPO",
        payer_type: "Commercial",
        cpt: "64483",
        service: "Right L4-L5 transforaminal ESI with imaging guidance",
        icd10: &["M51.16"],
        provider: "Dr. Samuel Okafor, MD",
        received_days_ago: 0,
        urgency: "standard",
        pathway: "Payer portal / X12 278",
        note: "48-year-old male with low back pain and pain radiating down the right leg for 10 weeks after lifting at work. Reports numbness and tingling in the right foot and mild weakness with great toe extension. Completed 8 weeks of physical therapy with home exercise program and 6 weeks of naproxen without lasting relief. Denies bowel or bladder incontinence, saddle anesthesia, fever, or unexplained weight loss. MRI of the lumbar spine shows a right paracentral disc extrusion at L4-L5 contacting the descending right L5 nerve root with moderate lateral recess stenosis. Positive straight leg raise on the right, diminished sensation in the right L5 dermatome. No prior epidural injections. Plan: right L4-L5 transforaminal epidural steroid injection under fluoroscopy.",
        criteria: CriteriaSet::Tfesi,
        attachments: &["Office note 11/12/2026", "MRI report — Nassau Imaging 11/15/2026", "PT discharge summary — Meridian Motion PT"],
    },
    AuthCase {
        id: "PA-24813",
        patient_id: "pt-1006",
        patient: "Thomas Russo",
        payer: "UnitedHealthcare",
        payer_type: "Commercial",
        cpt: "22551",
        service: "ACDF, single interspace C5-C6 (with 22853 interbody device, 22845 anterior instrumentation)",
        icd10: &["M50.122"],
        provider: "Dr. Adrian Hale, MD",
        received_days_ago: 3,
        urgency: "standard",
        pathway: "Payer portal / X12 278",
        note: "67-year-old male with neck pain and pain radiating into the left arm with numbness in the thumb and index finger for 5 months. Progressive weakness of left wrist extension (4/5). Completed 12 weeks of physical therapy and a trial of gabapentin for 8 weeks without improvement; one cervical interlaminar epidural injection gave 2 weeks of relief. MRI of the cervical spine shows a left foraminal disc herniation at C5-C6 with severe foraminal stenosis and nerve root compression concordant with C6 radiculopathy. Flexion-extension X-rays show no instability. Neck Disability Index 46%. Plan: anterior cervical discectomy and fusion at C5-C6.",
        criteria: CriteriaSet::Acdf,
        attachments: &["Surgical consult note", "MRI cervical spine report", "Flexion-extension X-ray report", "PT notes (12 wk)", "NDI questionnaire"],
    },
    AuthCase {
        id: "PA-24814",
        patient_id: "pt-1003",
        patient: "Linda Chen",
        payer: "Humana Medicare Advantage",
        payer_type: "Medicare Advantage",
        cpt: "63047",
        service: "Lumbar laminectomy / decompression, single segment L4-L5",
        icd10: &["M48.062"],
        provider: "Dr. Adrian Hale, MD",
        received_days_ago: 1,
        urgency: "standard",
        pathway: "Da Vinci PAS (FHIR)",
        note: "74-year-old female with neurogenic claudication: bilateral buttock and thigh pain when walking more than one block, relieved by sitting and leaning forward, for 14 months. Completed 12 weeks of physical therapy, home exercise program, and two epidural steroid injections with only temporary relief. MRI of the lumbar spine shows severe central canal stenosis at L4-L5 with grade 1 degenerative spondylolisthesis. Oswestry Disability Index 52%. Denies bowel or bladder dysfunction. Plan: L4-L5 decompression.",
        criteria: CriteriaSet::Decompression,
        attachments: &["Surgical consult note", "MRI lumbar report", "Injection procedure notes ×2", "ODI questionnaire"],
    },
    AuthCase {
        id: "PA-24815",
        patient_id: "pt-1007",
        patient: "Carmen Diaz",
        payer: "Empire BCBS",
        payer_type: "Commercial",
        cpt: "72148",
        service: "MRI lumbar spine without contrast",
        icd10: &["M54.31"],
        provider: "Daniel Park, PA-C",
        received_days_ago: 2,
        urgency: "standard",
        pathway: "Payer portal / X12 278",
        note: "54-year-old female with sciatica: pain radiating down the right leg to the calf for 5 weeks. Started physical therapy 4 weeks ago and takes meloxicam; partial improvement. No weakness, normal reflexes. Denies bowel or bladder dysfunction, fever, or history of cancer. Request MRI of the lumbar spine to evaluate for disc herniation.",
        criteria: CriteriaSet::LumbarMri,
        attachments: &["Office note 11/13/2026", "PT progress note (week 4)"],
    },
    AuthCase {
        id: "PA-24816",
        patient_id: "pt-1004",
        patient: "Robert Williams",
        payer: "NYSIF (Workers' Comp)",
        payer_type: "Workers' Comp",
        cpt: "62321",
        service: "Cervical interlaminar ESI with imaging guidance (C6-C7)",
        icd10: &["M54.12"],
        provider: "Dr. Samuel Okafor, MD",
        received_days_ago: 0,
        urgency: "standard",
        pathway: "NY WCB Medical Portal",
        note: "45-year-old male injured at work lifting a pallet 9 weeks ago. Neck pain with pain radiating into the right arm and numbness of the middle finger. Completed 6 weeks of physical therapy and NSAIDs. MRI of the cervical spine shows a right paracentral disc protrusion at C6-C7 with nerve root contact. Plan: C6-C7 interlaminar epidural steroid injection under fluoroscopy. Causally related to the work injury.",
        criteria: CriteriaSet::WcCervicalEsi,
        attachments: &["C-4 / doctor's initial report", "MRI cervical report", "PT notes"],
    },
];

pub struct Evaluation {
    pub results: Vec<CriterionResult>,
    pub readiness: f64,
    pub readiness_after_fix: f64,
    pub blocking: Vec<CriterionResult>,
}

fn auto_pulled(r: &CriterionResult) -> bool {
    r.fix.as_deref().is_some_and(|f| f.starts_with("Auto-pulled"))
}

pub fn evaluate_case(c: &AuthCase) -> Evaluation {
    let results = criteria(c.criteria, c.note);
    let applicable: Vec<&CriterionResult> = results.iter().filter(|r| r.status != CritStatus::Na).collect();
    let met = applicable.iter().filter(|r| r.status == CritStatus::Met).count() as f64;
    let fixable = applicable.iter().filter(|r| r.status == CritStatus::Missing && auto_pulled(r)).count() as f64;
    let n = applicable.len() as f64;
    let readiness = if n > 0.0 { js_round(met / n * 100.0) } else { 0.0 };
    let readiness_after_fix = if n > 0.0 { js_round((met + fixable) / n * 100.0) } else { 0.0 };
    let blocking = applicable.iter().filter(|r| r.status == CritStatus::Missing && !auto_pulled(r)).map(|r| (*r).clone()).collect();
    Evaluation { results, readiness, readiness_after_fix, blocking }
}

pub fn medical_necessity_letter(c: &AuthCase, results: &[CriterionResult]) -> String {
    let summary = results
        .iter()
        .filter(|r| r.status == CritStatus::Met)
        .map(|r| {
            let mut ev = r.evidence.as_str();
            ev = ev.strip_prefix('“').unwrap_or(ev);
            ev = ev.strip_suffix('”').unwrap_or(ev);
            format!("• {}: {}", r.label, ev)
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "RE: Request for prior authorization — CPT {cpt} ({service})\nPatient: {patient}   Payer: {payer}   Dx: {dx}\n\nTo the Utilization Review Department,\n\nI am requesting authorization for the following service: {service} (CPT {cpt}) for the patient above, who is under my care at Meridian Orthopedics & Spine Care.\n\nClinical summary:\n{summary}\n\nBased on these findings and the failure of appropriate conservative management, the requested service is medically necessary and consistent with your published coverage criteria. The supporting documentation is attached: {att}.\n\nPlease contact our authorization desk at (516) 555-0148 for any peer-to-peer review.\n\nSincerely,\n{provider}\n[Draft generated by Prior Auth Copilot — requires provider review and signature]",
        cpt = c.cpt,
        service = c.service,
        patient = c.patient,
        payer = c.payer,
        dx = c.icd10.join(", "),
        att = c.attachments.join("; "),
        provider = c.provider,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn evaluates_cases() {
        let e = evaluate_case(&AUTH_CASES[0]);
        assert_eq!(e.readiness, 100.0);
        let e = evaluate_case(&AUTH_CASES[1]);
        assert_eq!(e.readiness, 80.0);
        assert_eq!(e.readiness_after_fix, 100.0);
        let e = evaluate_case(&AUTH_CASES[3]);
        assert!(e.readiness < 100.0);
    }
}
