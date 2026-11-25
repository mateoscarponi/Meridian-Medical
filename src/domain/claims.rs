//! Claim scrubber: deterministic edit rules (NCCI-style bundling, modifiers, code validity,
//! payer-specific timely filing and NY workers' comp / no-fault requirements) + a denial-risk score.
//!
//! Fixes are a typed enum instead of stored closures, so every fix is explicit, testable and
//! serializable (handy for an audit trail).

use super::patients::PayerType;

#[derive(Clone, Debug, PartialEq)]
pub struct ClaimLine {
    pub cpt: String,
    pub desc: String,
    pub modifiers: Vec<String>,
    pub units: u32,
    pub icd10: Vec<String>,
    pub charge: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Claim {
    pub id: String,
    pub patient: String,
    pub payer: String,
    pub payer_type: PayerType,
    pub dos: String,
    pub days_since_dos: u32,
    pub provider: String,
    pub lines: Vec<ClaimLine>,
    pub auth_number: Option<String>,
    pub wcb_case_number: Option<String>,
    pub has_justification_letter: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Critical,
    Warning,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Critical => "critical",
            Severity::Warning => "warning",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Fix {
    RemoveFluoroscopy,
    AddModifier25,
    ReplaceRetiredLowBackPain,
    CollapseBilateral,
    LinkRadicularDx { cpt: String },
    AttachAuth,
    AttachJustification,
    AddWcbCase,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrubFinding {
    pub rule: &'static str,
    pub severity: Severity,
    pub message: String,
    pub fix: &'static str,
    pub weight: u32,
    pub apply: Option<Fix>,
}

fn is_em(cpt: &str) -> bool {
    // /^992[01]\d$/
    let b = cpt.as_bytes();
    b.len() == 5 && cpt.starts_with("992") && (b[3] == b'0' || b[3] == b'1') && b[4].is_ascii_digit()
}

fn auth_required(cpt: &str, pt: PayerType) -> bool {
    use PayerType::*;
    match cpt {
        "72148" | "64483" | "62321" => matches!(pt, Commercial | MedicareAdvantage),
        "22551" | "63047" => matches!(pt, Commercial | MedicareAdvantage | Medicaid),
        _ => false,
    }
}

pub fn apply_fix(c: &Claim, fix: &Fix) -> Claim {
    let mut y = c.clone();
    match fix {
        Fix::RemoveFluoroscopy => y.lines.retain(|l| l.cpt != "77003"),
        Fix::AddModifier25 => {
            for l in y.lines.iter_mut().filter(|l| is_em(&l.cpt)) {
                l.modifiers.push("25".into());
            }
        }
        Fix::ReplaceRetiredLowBackPain => {
            for l in y.lines.iter_mut() {
                for d in l.icd10.iter_mut() {
                    if d == "M54.5" {
                        *d = "M54.50".into();
                    }
                }
            }
        }
        Fix::CollapseBilateral => {
            let tf: Vec<&ClaimLine> = c.lines.iter().filter(|l| l.cpt == "64483").collect();
            let mut merged = tf[0].clone();
            merged.modifiers = vec!["50".into()];
            merged.units = 1;
            merged.charge = tf.iter().map(|l| l.charge).sum();
            let rest: Vec<ClaimLine> = c.lines.iter().filter(|l| l.cpt != "64483").cloned().collect();
            y.lines = std::iter::once(merged).chain(rest).collect();
        }
        Fix::LinkRadicularDx { cpt } => {
            for l in y.lines.iter_mut().filter(|l| &l.cpt == cpt) {
                let mut dx = vec!["M54.16".to_string()];
                dx.extend(l.icd10.iter().filter(|d| !d.starts_with("M54.5")).cloned());
                l.icd10 = dx;
            }
        }
        Fix::AttachAuth => y.auth_number = Some("AUTH-7714502".into()),
        Fix::AttachJustification => y.has_justification_letter = true,
        Fix::AddWcbCase => y.wcb_case_number = Some("G331 8042".into()),
    }
    y
}

pub fn scrub(c: &Claim) -> Vec<ScrubFinding> {
    let mut f = Vec::new();
    let has = |cpt: &str| c.lines.iter().any(|l| l.cpt == cpt);
    let injection = c.lines.iter().find(|l| ["64483", "64484", "62321", "62323", "64490", "64493"].contains(&l.cpt.as_str()));

    if let Some(inj) = injection {
        if has("77003") {
            f.push(ScrubFinding {
                rule: "NCCI bundling",
                severity: Severity::Critical,
                message: format!("77003 (fluoroscopic guidance) is bundled into {}; imaging guidance is included in the injection code.", inj.cpt),
                fix: "Remove the 77003 line.",
                weight: 34,
                apply: Some(Fix::RemoveFluoroscopy),
            });
        }
    }

    if let (Some(em), Some(inj)) = (c.lines.iter().find(|l| is_em(&l.cpt)), injection) {
        if !em.modifiers.iter().any(|m| m == "25") {
            f.push(ScrubFinding {
                rule: "Modifier 25",
                severity: Severity::Critical,
                message: format!("E/M {} billed on the same day as procedure {} without modifier 25.", em.cpt, inj.cpt),
                fix: "Append modifier 25: the note documents a separately identifiable evaluation (new complaint + decision for injection).",
                weight: 28,
                apply: Some(Fix::AddModifier25),
            });
        }
    }

    if c.lines.iter().any(|l| l.icd10.iter().any(|d| d == "M54.5")) {
        f.push(ScrubFinding {
            rule: "Invalid ICD-10",
            severity: Severity::Critical,
            message: "M54.5 has been invalid since Oct 1, 2021 (expanded to M54.50 / M54.51 / M54.59).".into(),
            fix: "Replace with M54.50 (low back pain, unspecified) — or a more specific code supported by the note.",
            weight: 40,
            apply: Some(Fix::ReplaceRetiredLowBackPain),
        });
    }

    let tfesi: Vec<&ClaimLine> = c.lines.iter().filter(|l| l.cpt == "64483").collect();
    if c.payer_type == PayerType::Medicare
        && tfesi.len() == 2
        && tfesi.iter().any(|l| l.modifiers.iter().any(|m| m == "RT"))
        && tfesi.iter().any(|l| l.modifiers.iter().any(|m| m == "LT"))
    {
        f.push(ScrubFinding {
            rule: "Bilateral billing",
            severity: Severity::Warning,
            message: "Bilateral 64483 billed as two lines (RT + LT). Medicare expects one line with modifier 50.".into(),
            fix: "Collapse into a single 64483-50 line, 1 unit, combined charge.",
            weight: 18,
            apply: Some(Fix::CollapseBilateral),
        });
    }

    if let Some(inj) = injection {
        if inj.icd10.iter().all(|d| d.starts_with("M54.5")) {
            f.push(ScrubFinding {
                rule: "Medical necessity",
                severity: Severity::Warning,
                message: format!("{} is linked only to non-specific low back pain; payers expect a radicular diagnosis for epidural injections.", inj.cpt),
                fix: "Link M54.16 (lumbar radiculopathy), documented in the 11/10 note by NLP chart review.",
                weight: 24,
                apply: Some(Fix::LinkRadicularDx { cpt: inj.cpt.clone() }),
            });
        }
    }

    let needs_auth = c.lines.iter().any(|l| auth_required(&l.cpt, c.payer_type));
    if needs_auth && c.auth_number.is_none() {
        f.push(ScrubFinding {
            rule: "Missing authorization",
            severity: Severity::Critical,
            message: "Service requires prior authorization but no auth number is on the claim (REF*G1).".into(),
            fix: "Attach approved auth from the Prior Auth module.",
            weight: 45,
            apply: Some(Fix::AttachAuth),
        });
    }

    if c.payer_type == PayerType::NoFault && c.days_since_dos > 45 && !c.has_justification_letter {
        f.push(ScrubFinding {
            rule: "NY no-fault 45-day rule",
            severity: Severity::Critical,
            message: format!("Bill is {} days from date of service; NY no-fault requires submission within 45 days unless a reasonable justification is provided.", c.days_since_dos),
            fix: "Attach the auto-drafted late-submission justification (records received late from the hospital).",
            weight: 38,
            apply: Some(Fix::AttachJustification),
        });
    }

    if c.payer_type == PayerType::WorkersComp && c.wcb_case_number.is_none() {
        f.push(ScrubFinding {
            rule: "WCB case number",
            severity: Severity::Critical,
            message: "Workers' compensation bill is missing the WCB case number and carrier claim number.".into(),
            fix: "Pull WCB case # from the patient's C-4 on file and the carrier claim # from the intake form.",
            weight: 36,
            apply: Some(Fix::AddWcbCase),
        });
    }

    f
}

pub fn denial_risk(c: &Claim) -> (Vec<ScrubFinding>, u32) {
    let findings = scrub(c);
    let raw = 6 + findings.iter().map(|x| x.weight).sum::<u32>();
    (findings, raw.min(97))
}

pub fn total(c: &Claim) -> f64 {
    c.lines.iter().map(|l| l.charge).sum()
}

pub fn line_code(l: &ClaimLine) -> String {
    if l.modifiers.is_empty() {
        l.cpt.clone()
    } else {
        format!("{}-{}", l.cpt, l.modifiers.join("-"))
    }
}

/// Simplified X12 837P claim loop.
pub fn to_837(c: &Claim) -> String {
    let dos = &c.dos;
    let d = |a: usize, b: usize| dos.get(a..b).unwrap_or("").to_string();
    let date = format!("{}{}{}", dos.get(6..).unwrap_or(""), d(0, 2), d(3, 5));
    let lines: Vec<String> = c
        .lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let mods = if l.modifiers.is_empty() { String::new() } else { format!(":{}", l.modifiers.join(":")) };
            let ptrs = (1..=l.icd10.len()).map(|k| k.to_string()).collect::<Vec<_>>().join(":");
            format!("LX*{}~\nSV1*HC:{}{mods}*{:.2}*UN*{}***{ptrs}~\nDTP*472*D8*{date}~", i + 1, l.cpt, l.charge, l.units)
        })
        .collect();
    let mut seen: Vec<String> = Vec::new();
    for dx in c.lines.iter().flat_map(|l| l.icd10.iter()) {
        if !seen.contains(dx) {
            seen.push(dx.clone());
        }
    }
    let dx = seen
        .iter()
        .enumerate()
        .map(|(i, d)| format!("{}:{}", if i == 0 { "ABK" } else { "ABF" }, d.replacen('.', "", 1)))
        .collect::<Vec<_>>()
        .join("*");
    let mut out = vec!["ST*837*0001*005010X222A1~".to_string(), format!("CLM*{}*{:.2}***11:B:1*Y*A*Y*Y~", c.id, total(c))];
    if let Some(a) = &c.auth_number {
        out.push(format!("REF*G1*{a}~"));
    }
    if let Some(w) = &c.wcb_case_number {
        out.push(format!("REF*Y4*{w}~"));
    }
    out.push(format!("HI*{dx}~"));
    out.extend(lines);
    out.push("SE*…*0001~".into());
    out.join("\n")
}

fn line(cpt: &str, desc: &str, mods: &[&str], icd: &[&str], charge: f64) -> ClaimLine {
    ClaimLine {
        cpt: cpt.into(),
        desc: desc.into(),
        modifiers: mods.iter().map(|s| s.to_string()).collect(),
        units: 1,
        icd10: icd.iter().map(|s| s.to_string()).collect(),
        charge,
    }
}

#[allow(clippy::too_many_arguments)]
fn claim(id: &str, patient: &str, payer: &str, pt: PayerType, dos: &str, days: u32, provider: &str, lines: Vec<ClaimLine>, auth: Option<&str>) -> Claim {
    Claim {
        id: id.into(),
        patient: patient.into(),
        payer: payer.into(),
        payer_type: pt,
        dos: dos.into(),
        days_since_dos: days,
        provider: provider.into(),
        lines,
        auth_number: auth.map(str::to_string),
        wcb_case_number: None,
        has_justification_letter: false,
    }
}

pub fn claims() -> Vec<Claim> {
    use PayerType::*;
    vec![
        claim(
            "CLM-90431", "James O'Connor", "Aetna PPO", Commercial, "11/10/2026", 6, "Dr. Okafor",
            vec![
                line("99214", "Office visit, est. patient, moderate MDM", &[], &["M51.16"], 210.0),
                line("64483", "TFESI lumbar/sacral, single level", &["RT"], &["M51.16"], 1150.0),
                line("77003", "Fluoroscopic guidance, spine injection", &[], &["M51.16"], 180.0),
            ],
            Some("AUTH-7714380"),
        ),
        claim(
            "CLM-90432", "Harold Becker", "Medicare Part B", Medicare, "11/09/2026", 7, "Dr. Okafor",
            vec![
                line("64483", "TFESI lumbar/sacral, single level", &["RT"], &["M54.5"], 1150.0),
                line("64483", "TFESI lumbar/sacral, single level", &["LT"], &["M54.5"], 1150.0),
            ],
            None,
        ),
        claim(
            "CLM-90433", "Carmen Diaz", "Empire BCBS", Commercial, "11/13/2026", 3, "D. Park, PA-C",
            vec![line("99203", "Office visit, new patient, low MDM", &[], &["M54.31"], 260.0)],
            None,
        ),
        claim(
            "CLM-90434", "Aisha Rahman", "GEICO No-Fault", NoFault, "09/22/2026", 55, "Dr. Ruiz",
            vec![
                line("99204", "Office visit, new patient, moderate MDM", &[], &["S13.4XXA", "M54.2"], 340.0),
                line("72040", "X-ray cervical spine, 2-3 views", &[], &["S13.4XXA"], 120.0),
            ],
            None,
        ),
        claim(
            "CLM-90435", "Robert Williams", "NYSIF", WorkersComp, "11/11/2026", 5, "Dr. Okafor",
            vec![line("62321", "Interlaminar ESI cervical/thoracic w/ imaging", &[], &["M54.12"], 980.0)],
            None,
        ),
        claim(
            "CLM-90436", "Priya Natarajan", "Cigna Open Access", Commercial, "11/12/2026", 4, "Dr. Hale",
            vec![
                line("99204", "Office visit, new patient, moderate MDM", &[], &["M41.26"], 340.0),
                line("72082", "X-ray entire spine, 2-3 views (scoliosis)", &[], &["M41.26"], 210.0),
            ],
            None,
        ),
        claim(
            "CLM-90437", "Linda Chen", "Humana Medicare Advantage", MedicareAdvantage, "11/06/2026", 10, "Dr. Okafor",
            vec![
                line("99214", "Office visit, est. patient, moderate MDM", &[], &["M48.062"], 210.0),
                line("62323", "Interlaminar ESI lumbar/sacral w/ imaging", &[], &["M48.062"], 1020.0),
            ],
            None,
        ),
        claim(
            "CLM-90438", "Kevin Murphy", "Oxford (UHC)", Commercial, "11/14/2026", 2, "D. Park, PA-C",
            vec![line("99213", "Office visit, est. patient, low MDM", &[], &["M54.2"], 150.0)],
            None,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixes_converge() {
        for c in claims() {
            let mut x = c.clone();
            for _ in 0..8 {
                let (f, _) = denial_risk(&x);
                match f.iter().find_map(|y| y.apply.clone()) {
                    Some(fix) => x = apply_fix(&x, &fix),
                    None => break,
                }
            }
            assert!(denial_risk(&x).0.is_empty(), "{} still has findings", c.id);
        }
        let c = &claims()[0];
        assert_eq!(denial_risk(c).1, 6 + 34 + 28);
    }
}
