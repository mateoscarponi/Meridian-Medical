//! Public-facing practice content for the demo site.
//! A fictional orthopedic & spine practice. Names, founding year, tagline, office towns, phone and
//! provider bios are all invented so the demo can never be mistaken for a real office.

pub struct Practice {
    pub name: &'static str,
    pub short: &'static str,
    pub tagline: &'static str,
    pub phone: &'static str,
    pub since: u32,
    pub region: &'static str,
}

pub const PRACTICE: Practice = Practice {
    name: "Meridian Orthopedics & Spine Care",
    short: "Meridian Orthopedics & Spine",
    tagline: "Precise care. A clear path back to motion.",
    phone: "(516) 555-0148",
    since: 2011,
    region: "Nassau & Suffolk",
};

pub struct Specialty {
    pub slug: &'static str,
    pub title: &'static str,
    pub blurb: &'static str,
    pub items: [&'static str; 4],
}

pub const SPECIALTIES: [Specialty; 4] = [
    Specialty {
        slug: "spine-surgery",
        title: "Spine & Neck Surgery",
        blurb: "Motion-preserving and minimally invasive surgery for the neck and back, planned only after conservative care has been given a fair chance.",
        items: ["Minimally invasive decompression", "ACDF & cervical disc replacement", "Adult scoliosis care", "Revision spine surgery"],
    },
    Specialty {
        slug: "pain-management",
        title: "Interventional Spine",
        blurb: "Image-guided procedures in our own suite that calm inflamed nerves and joints, so most patients recover without an operation.",
        items: ["Epidural steroid injections", "Facet & medial branch blocks", "Radiofrequency ablation", "Medication management"],
    },
    Specialty {
        slug: "orthopedics",
        title: "Joint & Sports Medicine",
        blurb: "Shoulder, knee, hip and hand care for athletes, workers and weekend gardeners, plus walk-in injury visits without the ER wait.",
        items: ["Arthritis & joint pain", "Sports & work injuries", "Fracture care", "Walk-in injury clinic"],
    },
    Specialty {
        slug: "physical-therapy",
        title: "Rehab & Movement",
        blurb: "In-house physical therapy at Meridian Motion PT, so your surgeon, spine physician and therapist all work from one plan.",
        items: ["Post-surgical rehab", "Spine stabilization", "Manual therapy", "Return-to-work programs"],
    },
];

pub struct TitleText {
    pub title: &'static str,
    pub text: &'static str,
}

pub const CONDITIONS: [TitleText; 8] = [
    TitleText { title: "Neck Pain", text: "Cervical strain, disc disease and pinched nerves that radiate into the shoulders and arms." },
    TitleText { title: "Lower Back Pain", text: "Mechanical, disc-related and arthritic back pain, treated conservatively first." },
    TitleText { title: "Herniated Disc", text: "When disc material presses on a nerve root, causing pain, numbness or weakness." },
    TitleText { title: "Spinal Stenosis", text: "Narrowing of the spinal canal that causes leg pain when walking or standing." },
    TitleText { title: "Sciatica", text: "Shooting pain down the leg from irritation of the lumbar nerve roots." },
    TitleText { title: "Adult Scoliosis", text: "Curvature of the spine in adulthood, often with stenosis and imbalance." },
    TitleText { title: "ACDF", text: "Anterior cervical discectomy and fusion for cervical compression that fails conservative care." },
    TitleText { title: "Minimally Invasive Surgery", text: "Smaller incisions, less muscle disruption and faster return to daily life." },
];

pub struct CareStep {
    pub step: &'static str,
    pub title: &'static str,
    pub text: &'static str,
}

pub const CARE_PATH: [CareStep; 4] = [
    CareStep { step: "01", title: "Evaluate", text: "A thorough exam, imaging review and a clear diagnosis, often on the first visit." },
    CareStep { step: "02", title: "Conservative care", text: "Physical therapy, activity changes and medication: the first line for most patients." },
    CareStep { step: "03", title: "Interventional", text: "Image-guided injections and ablation in our own procedure suite when pain persists." },
    CareStep { step: "04", title: "Surgery, if needed", text: "Minimally invasive and reconstructive options, tailored with you and your family." },
];

pub struct TeamMember {
    pub name: &'static str,
    pub role: &'static str,
    pub detail: &'static str,
    pub initials: &'static str,
}

pub const TEAM: [TeamMember; 4] = [
    TeamMember {
        name: "Dr. Adrian Hale, MD",
        role: "Spine Surgeon · Medical Director",
        detail: "Minimally invasive and motion-preserving neck and back surgery. Leads the practice's quality program.",
        initials: "AH",
    },
    TeamMember {
        name: "Dr. Elena Ruiz, MD",
        role: "Joint & Sports Medicine Surgeon",
        detail: "Shoulder and knee arthroscopy, joint preservation and work-related injuries.",
        initials: "ER",
    },
    TeamMember {
        name: "Dr. Samuel Okafor, MD",
        role: "Physiatrist · Interventional Spine",
        detail: "Physical medicine & rehabilitation: image-guided injections, nerve ablation and non-surgical recovery plans.",
        initials: "SO",
    },
    TeamMember {
        name: "Daniel Park, PA-C",
        role: "Physician Assistant · Walk-in Clinic",
        detail: "Same-week new-patient visits, post-operative checks and walk-in injury care.",
        initials: "DP",
    },
];

pub struct Location {
    pub town: &'static str,
    pub kind: &'static str,
    pub note: &'static str,
    pub county: &'static str,
}

pub const LOCATIONS: [Location; 4] = [
    Location { town: "Hicksville", kind: "Walk-in Injury Clinic", note: "No appointment needed for sprains, fractures & acute back pain.", county: "Nassau" },
    Location { town: "Garden City", kind: "Spine Center & Procedure Suite", note: "Image-guided procedure suite and surgical consultations.", county: "Nassau" },
    Location { town: "Huntington", kind: "Spine & Joint Office", note: "Consultations, follow-ups and imaging review.", county: "Suffolk" },
    Location { town: "Smithtown", kind: "Rehab & Movement Center", note: "Home of Meridian Motion PT and return-to-work programs.", county: "Suffolk" },
];

pub struct Insurance {
    pub name: &'static str,
    pub note: &'static str,
}

pub const INSURANCE: [Insurance; 5] = [
    Insurance { name: "Medicare", note: "Accepted" },
    Insurance { name: "Workers' Compensation", note: "NYS & federal programs" },
    Insurance { name: "No-Fault", note: "Motor vehicle accident plans" },
    Insurance { name: "Medicaid", note: "Established patients & hospital follow-ups" },
    Insurance { name: "Commercial plans", note: "Call billing to confirm" },
];

pub const VISIT_CHECKLIST: [&str; 6] = [
    "Photo ID and insurance card",
    "Imaging and reports: X-rays, MRI CDs, CT scans, EMG",
    "Past operative and treatment reports",
    "Current medication list",
    "Completed intake & HIPAA forms (English or Spanish)",
    "PCP referral, if your plan requires one",
];
