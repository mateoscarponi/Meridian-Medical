// Public-facing practice content for the demo site.
// Clinical scope mirrors a real Long Island orthopedic & spine practice; names, phone and
// street details are fictional so the demo can never be mistaken for a real office.

export const PRACTICE = {
  name: "Mateo Orthopedic & Spine Care",
  short: "Mateo Orthopedic & Spine",
  tagline: "Our Patients. Our Purpose. Our Passion.",
  phone: "(516) 555-0148",
  since: 2018,
  region: "Long Island & Queens",
};

export const SPECIALTIES = [
  {
    slug: "spine-surgery",
    title: "Spine Surgery",
    blurb:
      "Fellowship-trained care for complex spine and deformity, from minimally invasive decompression to cervical fusion and disc replacement.",
    items: ["Minimally invasive spine surgery", "ACDF & cervical disc replacement", "Adult scoliosis & deformity", "Complex reconstruction"],
  },
  {
    slug: "pain-management",
    title: "Pain Management",
    blurb:
      "An on-site interventional suite for image-guided relief, so most patients improve without ever needing an operation.",
    items: ["Epidural steroid injections", "Facet & medial branch blocks", "Radiofrequency ablation", "Medication management"],
  },
  {
    slug: "orthopedics",
    title: "Orthopedics",
    blurb:
      "Diagnosis and treatment of joint, tendon and trauma-related conditions, including urgent orthopedic visits without the ER wait.",
    items: ["Osteoarthritis & joint pain", "Sports & work injuries", "Fracture care", "Orthopedic urgent care"],
  },
  {
    slug: "physical-therapy",
    title: "Physical Therapy",
    blurb:
      "Go Rehab-style in-house physical therapy that keeps your surgeon, pain physician and therapist working from one plan.",
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
    name: "Dr. Adrian Mateo, DO",
    role: "Orthopedic Spine Surgeon · Director",
    detail: "Fellowship-trained in complex spine & deformity. Faculty, orthopedic residency program.",
    initials: "AM",
  },
  {
    name: "Dr. Elena Ruiz, MD",
    role: "Board-Certified Orthopedic Surgeon",
    detail: "Joint preservation, trauma and work-related orthopedic injuries.",
    initials: "ER",
  },
  {
    name: "Dr. Samuel Okafor, MD",
    role: "Interventional Pain Management",
    detail: "Image-guided spine injections, radiofrequency ablation and non-surgical care.",
    initials: "SO",
  },
  {
    name: "Daniel Park, PA-C",
    role: "Orthopedics",
    detail: "Same-week new-patient visits, post-operative care and urgent orthopedic visits.",
    initials: "DP",
  },
];

export const LOCATIONS = [
  { town: "East Meadow", kind: "Orthopedic Urgent Care", note: "Walk-ins welcome for sprains, fractures & acute back pain.", county: "Nassau" },
  { town: "Westbury", kind: "Spine Center & Procedure Suite", note: "Interventional pain suite and surgical consultations.", county: "Nassau" },
  { town: "Lindenhurst", kind: "Spine & Orthopedic Office", note: "Consultations, follow-ups and physical therapy.", county: "Suffolk" },
  { town: "Elmhurst", kind: "Queens Office", note: "Spine surgery consultations on Queens Boulevard.", county: "Queens" },
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
