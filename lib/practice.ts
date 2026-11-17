// Public-facing practice content for the demo site.
// A fictional orthopedic & spine practice. Names, founding year, tagline, office towns, phone and
// provider bios are all invented so the demo can never be mistaken for a real office.

export const PRACTICE = {
  name: "Meridian Orthopedics & Spine Care",
  short: "Meridian Orthopedics & Spine",
  tagline: "Precise care. A clear path back to motion.",
  phone: "(516) 555-0148",
  since: 2011,
  region: "Nassau & Suffolk",
};

export const SPECIALTIES = [
  {
    slug: "spine-surgery",
    title: "Spine & Neck Surgery",
    blurb:
      "Motion-preserving and minimally invasive surgery for the neck and back, planned only after conservative care has been given a fair chance.",
    items: ["Minimally invasive decompression", "ACDF & cervical disc replacement", "Adult scoliosis care", "Revision spine surgery"],
  },
  {
    slug: "pain-management",
    title: "Interventional Spine",
    blurb:
      "Image-guided procedures in our own suite that calm inflamed nerves and joints, so most patients recover without an operation.",
    items: ["Epidural steroid injections", "Facet & medial branch blocks", "Radiofrequency ablation", "Medication management"],
  },
  {
    slug: "orthopedics",
    title: "Joint & Sports Medicine",
    blurb:
      "Shoulder, knee, hip and hand care for athletes, workers and weekend gardeners, plus walk-in injury visits without the ER wait.",
    items: ["Arthritis & joint pain", "Sports & work injuries", "Fracture care", "Walk-in injury clinic"],
  },
  {
    slug: "physical-therapy",
    title: "Rehab & Movement",
    blurb:
      "In-house physical therapy at Meridian Motion PT, so your surgeon, spine physician and therapist all work from one plan.",
    items: ["Post-surgical rehab", "Spine stabilization", "Manual therapy", "Return-to-work programs"],
  },
] as const;

export const CONDITIONS = [
  { title: "Neck Pain", text: "Cervical strain, disc disease and pinched nerves that radiate into the shoulders and arms." },
  { title: "Lower Back Pain", text: "Mechanical, disc-related and arthritic back pain, treated conservatively first." },
  { title: "Herniated Disc", text: "When disc material presses on a nerve root, causing pain, numbness or weakness." },
  { title: "Spinal Stenosis", text: "Narrowing of the spinal canal that causes leg pain when walking or standing." },
  { title: "Sciatica", text: "Shooting pain down the leg from irritation of the lumbar nerve roots." },
  { title: "Adult Scoliosis", text: "Curvature of the spine in adulthood, often with stenosis and imbalance." },
  { title: "ACDF", text: "Anterior cervical discectomy and fusion for cervical compression that fails conservative care." },
  { title: "Minimally Invasive Surgery", text: "Smaller incisions, less muscle disruption and faster return to daily life." },
];

export const CARE_PATH = [
  { step: "01", title: "Evaluate", text: "A thorough exam, imaging review and a clear diagnosis, often on the first visit." },
  { step: "02", title: "Conservative care", text: "Physical therapy, activity changes and medication: the first line for most patients." },
  { step: "03", title: "Interventional", text: "Image-guided injections and ablation in our own procedure suite when pain persists." },
  { step: "04", title: "Surgery, if needed", text: "Minimally invasive and reconstructive options, tailored with you and your family." },
];

export const TEAM = [
  {
    name: "Dr. Adrian Hale, MD",
    role: "Spine Surgeon · Medical Director",
    detail: "Minimally invasive and motion-preserving neck and back surgery. Leads the practice's quality program.",
    initials: "AH",
  },
  {
    name: "Dr. Elena Ruiz, MD",
    role: "Joint & Sports Medicine Surgeon",
    detail: "Shoulder and knee arthroscopy, joint preservation and work-related injuries.",
    initials: "ER",
  },
  {
    name: "Dr. Samuel Okafor, MD",
    role: "Physiatrist · Interventional Spine",
    detail: "Physical medicine & rehabilitation: image-guided injections, nerve ablation and non-surgical recovery plans.",
    initials: "SO",
  },
  {
    name: "Daniel Park, PA-C",
    role: "Physician Assistant · Walk-in Clinic",
    detail: "Same-week new-patient visits, post-operative checks and walk-in injury care.",
    initials: "DP",
  },
];

export const LOCATIONS = [
  { town: "Hicksville", kind: "Walk-in Injury Clinic", note: "No appointment needed for sprains, fractures & acute back pain.", county: "Nassau" },
  { town: "Garden City", kind: "Spine Center & Procedure Suite", note: "Image-guided procedure suite and surgical consultations.", county: "Nassau" },
  { town: "Huntington", kind: "Spine & Joint Office", note: "Consultations, follow-ups and imaging review.", county: "Suffolk" },
  { town: "Smithtown", kind: "Rehab & Movement Center", note: "Home of Meridian Motion PT and return-to-work programs.", county: "Suffolk" },
];

export const INSURANCE = [
  { name: "Medicare", note: "Accepted" },
  { name: "Workers' Compensation", note: "NYS & federal programs" },
  { name: "No-Fault", note: "Motor vehicle accident plans" },
  { name: "Medicaid", note: "Established patients & hospital follow-ups" },
  { name: "Commercial plans", note: "Call billing to confirm" },
];

export const VISIT_CHECKLIST = [
  "Photo ID and insurance card",
  "Imaging and reports: X-rays, MRI CDs, CT scans, EMG",
  "Past operative and treatment reports",
  "Current medication list",
  "Completed intake & HIPAA forms (English or Spanish)",
  "PCP referral, if your plan requires one",
];
