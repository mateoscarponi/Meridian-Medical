//! Synthetic patients, coverages and appointments. No real people, no real PHI.
//! Shared by the FHIR API, the scheduling model, the AI agent and the dashboards.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PayerType {
    Commercial,
    Medicare,
    MedicareAdvantage,
    Medicaid,
    WorkersComp,
    NoFault,
}

impl PayerType {
    pub fn as_str(self) -> &'static str {
        match self {
            PayerType::Commercial => "Commercial",
            PayerType::Medicare => "Medicare",
            PayerType::MedicareAdvantage => "Medicare Advantage",
            PayerType::Medicaid => "Medicaid",
            PayerType::WorkersComp => "Workers' Comp",
            PayerType::NoFault => "No-Fault",
        }
    }
}

#[derive(Debug)]
pub struct DemoPatient {
    pub id: &'static str,
    pub mrn: &'static str,
    pub given: &'static str,
    pub family: &'static str,
    pub birth_date: &'static str,
    pub gender: &'static str,
    pub phone: &'static str,
    pub language: &'static str,
    pub city: &'static str,
    pub payer: &'static str,
    pub payer_type: PayerType,
    pub member_id: &'static str,
    pub condition: &'static str,
    pub icd10: &'static str,
}

macro_rules! pt {
    ($id:expr, $mrn:expr, $given:expr, $family:expr, $dob:expr, $gender:expr, $phone:expr, $lang:expr, $city:expr, $payer:expr, $pt:ident, $member:expr, $cond:expr, $icd:expr) => {
        DemoPatient {
            id: $id, mrn: $mrn, given: $given, family: $family, birth_date: $dob, gender: $gender, phone: $phone,
            language: $lang, city: $city, payer: $payer, payer_type: PayerType::$pt, member_id: $member, condition: $cond, icd10: $icd,
        }
    };
}

pub static PATIENTS: [DemoPatient; 10] = [
    pt!("pt-1001", "MOS-204871", "Maria", "Gonzalez", "1968-04-12", "female", "(516) 555-0191", "es", "Hempstead", "Fidelis Care", Medicaid, "FC88310442", "Lumbar radiculopathy", "M54.16"),
    pt!("pt-1002", "MOS-198344", "James", "O'Connor", "1975-09-30", "male", "(516) 555-0127", "en", "Levittown", "Aetna PPO", Commercial, "W219-55810", "L4-L5 disc herniation", "M51.16"),
    pt!("pt-1003", "MOS-187205", "Linda", "Chen", "1952-01-18", "female", "(718) 555-0164", "en", "Flushing", "Humana Medicare Advantage", MedicareAdvantage, "H5619-0042", "Lumbar stenosis with claudication", "M48.062"),
    pt!("pt-1004", "MOS-210933", "Robert", "Williams", "1981-06-02", "male", "(631) 555-0118", "en", "Babylon", "NYSIF (Workers' Comp)", WorkersComp, "WCB G3318042", "Cervical radiculopathy, work injury", "M54.12"),
    pt!("pt-1005", "MOS-211476", "Aisha", "Rahman", "1990-11-23", "female", "(718) 555-0150", "en", "Jackson Heights", "GEICO No-Fault", NoFault, "NF-77120934", "Cervical strain after MVA", "S13.4XXA"),
    pt!("pt-1006", "MOS-176620", "Thomas", "Russo", "1959-03-07", "male", "(516) 555-0133", "en", "Hicksville", "UnitedHealthcare", Commercial, "UHC 90418827", "C5-C6 disc with radiculopathy", "M50.122"),
    pt!("pt-1007", "MOS-205518", "Carmen", "Diaz", "1972-08-15", "female", "(516) 555-0172", "es", "Garden City", "Empire BCBS", Commercial, "YRP 451207733", "Sciatica", "M54.31"),
    pt!("pt-1008", "MOS-169087", "Harold", "Becker", "1946-12-01", "male", "(631) 555-0145", "en", "Huntington", "Medicare Part B", Medicare, "1EG4-TE5-MK72", "Lumbar spondylosis", "M47.816"),
    pt!("pt-1009", "MOS-213390", "Priya", "Natarajan", "1986-05-27", "female", "(516) 555-0109", "en", "Garden City", "Cigna Open Access", Commercial, "U7730 18842", "Adult idiopathic scoliosis", "M41.26"),
    pt!("pt-1010", "MOS-199812", "Kevin", "Murphy", "1979-02-14", "male", "(631) 555-0186", "en", "Massapequa", "Oxford (UHC)", Commercial, "OXF 33019276", "Neck pain", "M54.2"),
];

pub struct Provider {
    pub id: &'static str,
    pub name: &'static str,
    pub short: &'static str,
    pub specialty: &'static str,
    pub npi: &'static str,
}

pub const PROVIDERS: [Provider; 4] = [
    Provider { id: "pr-01", name: "Dr. Adrian Hale, MD", short: "Dr. Hale", specialty: "Spine Surgery", npi: "1999999901" },
    Provider { id: "pr-02", name: "Dr. Elena Ruiz, MD", short: "Dr. Ruiz", specialty: "Joint & Sports Medicine", npi: "1999999902" },
    Provider { id: "pr-03", name: "Dr. Samuel Okafor, MD", short: "Dr. Okafor", specialty: "Interventional Spine", npi: "1999999903" },
    Provider { id: "pr-04", name: "Daniel Park, PA-C", short: "D. Park, PA-C", specialty: "Joint & Sports Medicine", npi: "1999999904" },
];

pub struct LocationId {
    pub id: &'static str,
    pub name: &'static str,
}

pub const LOCATION_IDS: [LocationId; 4] = [
    LocationId { id: "loc-hv", name: "Hicksville" },
    LocationId { id: "loc-gc", name: "Garden City" },
    LocationId { id: "loc-hu", name: "Huntington" },
    LocationId { id: "loc-sm", name: "Smithtown" },
];

pub fn patient_name(p: &DemoPatient) -> String {
    format!("{} {}", p.given, p.family)
}

pub fn patient(id: &str) -> Option<&'static DemoPatient> {
    PATIENTS.iter().find(|p| p.id == id)
}

fn ymd(s: &str) -> (i32, i32, i32) {
    let mut it = s.split('-').map(|x| x.parse::<i32>().unwrap_or(0));
    (it.next().unwrap_or(0), it.next().unwrap_or(0), it.next().unwrap_or(0))
}

/// Age in whole years on 2026-11-16 (the demo's "today").
pub fn age(birth_date: &str) -> i32 {
    age_on(birth_date, "2026-11-16")
}

pub fn age_on(birth_date: &str, on: &str) -> i32 {
    let (by, bm, bd) = ymd(birth_date);
    let (oy, om, od) = ymd(on);
    let mut a = oy - by;
    let m = om - bm;
    if m < 0 || (m == 0 && od < bd) {
        a -= 1;
    }
    a
}

pub fn find_patient(query: &str) -> Vec<&'static DemoPatient> {
    let q = query.trim().to_lowercase();
    PATIENTS
        .iter()
        .filter(|p| {
            p.id == q
                || p.mrn.to_lowercase() == q
                || patient_name(p).to_lowercase().contains(&q)
                || p.family.to_lowercase().contains(&q)
                || p.given.to_lowercase().contains(&q)
        })
        .collect()
}
