// Synthetic patients, coverages and appointments. No real people, no real PHI.
// Shared by the FHIR API, the scheduling model, the AI agent and the dashboards.

export type PayerType = "Commercial" | "Medicare" | "Medicare Advantage" | "Medicaid" | "Workers' Comp" | "No-Fault";

export interface DemoPatient {
  id: string;
  mrn: string;
  given: string;
  family: string;
  birthDate: string;
  gender: "male" | "female";
  phone: string;
  language: "en" | "es";
  city: string;
  payer: string;
  payerType: PayerType;
  memberId: string;
  condition: string;
  icd10: string;
}

export const PATIENTS: DemoPatient[] = [
  { id: "pt-1001", mrn: "MOS-204871", given: "Maria", family: "Gonzalez", birthDate: "1968-04-12", gender: "female", phone: "(516) 555-0191", language: "es", city: "Hempstead", payer: "Fidelis Care", payerType: "Medicaid", memberId: "FC88310442", condition: "Lumbar radiculopathy", icd10: "M54.16" },
  { id: "pt-1002", mrn: "MOS-198344", given: "James", family: "O'Connor", birthDate: "1975-09-30", gender: "male", phone: "(516) 555-0127", language: "en", city: "Levittown", payer: "Aetna PPO", payerType: "Commercial", memberId: "W219-55810", condition: "L4-L5 disc herniation", icd10: "M51.16" },
  { id: "pt-1003", mrn: "MOS-187205", given: "Linda", family: "Chen", birthDate: "1952-01-18", gender: "female", phone: "(718) 555-0164", language: "en", city: "Flushing", payer: "Humana Medicare Advantage", payerType: "Medicare Advantage", memberId: "H5619-0042", condition: "Lumbar stenosis with claudication", icd10: "M48.062" },
  { id: "pt-1004", mrn: "MOS-210933", given: "Robert", family: "Williams", birthDate: "1981-06-02", gender: "male", phone: "(631) 555-0118", language: "en", city: "Babylon", payer: "NYSIF (Workers' Comp)", payerType: "Workers' Comp", memberId: "WCB G3318042", condition: "Cervical radiculopathy, work injury", icd10: "M54.12" },
  { id: "pt-1005", mrn: "MOS-211476", given: "Aisha", family: "Rahman", birthDate: "1990-11-23", gender: "female", phone: "(718) 555-0150", language: "en", city: "Elmhurst", payer: "GEICO No-Fault", payerType: "No-Fault", memberId: "NF-77120934", condition: "Cervical strain after MVA", icd10: "S13.4XXA" },
  { id: "pt-1006", mrn: "MOS-176620", given: "Thomas", family: "Russo", birthDate: "1959-03-07", gender: "male", phone: "(516) 555-0133", language: "en", city: "East Meadow", payer: "UnitedHealthcare", payerType: "Commercial", memberId: "UHC 90418827", condition: "C5-C6 disc with radiculopathy", icd10: "M50.122" },
  { id: "pt-1007", mrn: "MOS-205518", given: "Carmen", family: "Diaz", birthDate: "1972-08-15", gender: "female", phone: "(516) 555-0172", language: "es", city: "Westbury", payer: "Empire BCBS", payerType: "Commercial", memberId: "YRP 451207733", condition: "Sciatica", icd10: "M54.31" },
  { id: "pt-1008", mrn: "MOS-169087", given: "Harold", family: "Becker", birthDate: "1946-12-01", gender: "male", phone: "(631) 555-0145", language: "en", city: "Lindenhurst", payer: "Medicare Part B", payerType: "Medicare", memberId: "1EG4-TE5-MK72", condition: "Lumbar spondylosis", icd10: "M47.816" },
  { id: "pt-1009", mrn: "MOS-213390", given: "Priya", family: "Natarajan", birthDate: "1986-05-27", gender: "female", phone: "(516) 555-0109", language: "en", city: "Garden City", payer: "Cigna Open Access", payerType: "Commercial", memberId: "U7730 18842", condition: "Adult idiopathic scoliosis", icd10: "M41.26" },
  { id: "pt-1010", mrn: "MOS-199812", given: "Kevin", family: "Murphy", birthDate: "1979-02-14", gender: "male", phone: "(631) 555-0186", language: "en", city: "Massapequa", payer: "Oxford (UHC)", payerType: "Commercial", memberId: "OXF 33019276", condition: "Neck pain", icd10: "M54.2" },
];

export const PROVIDERS = [
  { id: "pr-01", name: "Dr. Adrian Mateo, DO", short: "Dr. Mateo", specialty: "Spine Surgery", npi: "1999999901" },
  { id: "pr-02", name: "Dr. Elena Ruiz, MD", short: "Dr. Ruiz", specialty: "Orthopedics", npi: "1999999902" },
  { id: "pr-03", name: "Dr. Samuel Okafor, MD", short: "Dr. Okafor", specialty: "Pain Management", npi: "1999999903" },
  { id: "pr-04", name: "Daniel Park, PA-C", short: "D. Park, PA-C", specialty: "Orthopedics", npi: "1999999904" },
];

export const LOCATION_IDS = [
  { id: "loc-em", name: "East Meadow" },
  { id: "loc-wb", name: "Westbury" },
  { id: "loc-lh", name: "Lindenhurst" },
  { id: "loc-el", name: "Elmhurst" },
];

export function patientName(p: DemoPatient) {
  return `${p.given} ${p.family}`;
}

export function age(birthDate: string, on = new Date("2026-11-16")) {
  const b = new Date(birthDate);
  let a = on.getFullYear() - b.getFullYear();
  const m = on.getMonth() - b.getMonth();
  if (m < 0 || (m === 0 && on.getDate() < b.getDate())) a--;
  return a;
}

export function findPatient(query: string) {
  const q = query.trim().toLowerCase();
  return PATIENTS.filter(
    (p) =>
      p.id === q ||
      p.mrn.toLowerCase() === q ||
      `${p.given} ${p.family}`.toLowerCase().includes(q) ||
      p.family.toLowerCase().includes(q) ||
      p.given.toLowerCase().includes(q),
  );
}
