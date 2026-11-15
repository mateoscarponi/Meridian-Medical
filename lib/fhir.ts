// Minimal FHIR R4 (US Core-flavored) builders used by the mock FHIR server and the UI.
import { PATIENTS, PROVIDERS, LOCATION_IDS, type DemoPatient } from "./data/patients";

export const FHIR_BASE = "/api/fhir";

type Json = Record<string, unknown>;

export function toFhirPatient(p: DemoPatient): Json {
  return {
    resourceType: "Patient",
    id: p.id,
    meta: { profile: ["http://hl7.org/fhir/us/core/StructureDefinition/us-core-patient"] },
    identifier: [
      {
        use: "usual",
        type: { coding: [{ system: "http://terminology.hl7.org/CodeSystem/v2-0203", code: "MR" }] },
        system: "urn:oid:2.16.840.1.113883.3.0000.1",
        value: p.mrn,
      },
    ],
    name: [{ use: "official", family: p.family, given: [p.given] }],
    telecom: [{ system: "phone", value: p.phone, use: "mobile" }],
    gender: p.gender,
    birthDate: p.birthDate,
    address: [{ city: p.city, state: "NY", country: "US" }],
    communication: [
      {
        language: { coding: [{ system: "urn:ietf:bcp:47", code: p.language === "es" ? "es" : "en" }] },
        preferred: true,
      },
    ],
  };
}

export function toFhirCoverage(p: DemoPatient): Json {
  return {
    resourceType: "Coverage",
    id: `cov-${p.id}`,
    status: "active",
    type: { text: p.payerType },
    subscriberId: p.memberId,
    beneficiary: { reference: `Patient/${p.id}` },
    payor: [{ display: p.payer }],
    period: { start: "2026-01-01", end: "2026-12-31" },
  };
}

// Same day as the Smart Scheduling board (Tue, Nov 17): patient, time, provider index, location index, visit.
const SCHEDULE: [string, string, number, number, string][] = [
  ["pt-1001", "08:15", 2, 1, "Follow-up"],
  ["pt-1008", "08:45", 3, 2, "Follow-up"],
  ["pt-1009", "09:30", 0, 2, "New patient consult"],
  ["pt-1005", "10:00", 1, 3, "Follow-up"],
  ["pt-1007", "10:45", 3, 1, "New patient consult"],
  ["pt-1010", "11:30", 3, 1, "Follow-up"],
  ["pt-1004", "13:00", 2, 1, "Procedure (ESI)"],
  ["pt-1006", "14:15", 0, 0, "Pre-op"],
  ["pt-1002", "15:00", 2, 1, "Follow-up"],
  ["pt-1003", "15:45", 0, 3, "Follow-up"],
];

export function demoAppointments(date = "2026-11-17"): Json[] {
  return SCHEDULE.map(([pid, time, provIdx, locIdx, visit], i) => {
    const p = PATIENTS.find((x) => x.id === pid)!;
    const prov = PROVIDERS[provIdx];
    const loc = LOCATION_IDS[locIdx];
    const start = `${date}T${time}:00-05:00`;
    return {
      resourceType: "Appointment",
      id: `appt-${3000 + i}`,
      status: "booked",
      serviceType: [{ text: visit }],
      reasonCode: [{ coding: [{ system: "http://hl7.org/fhir/sid/icd-10-cm", code: p.icd10, display: p.condition }] }],
      start,
      minutesDuration: visit.startsWith("New") ? 40 : 20,
      participant: [
        { actor: { reference: `Patient/${p.id}`, display: `${p.given} ${p.family}` }, status: "accepted" },
        { actor: { reference: `Practitioner/${prov.id}`, display: prov.name }, status: "accepted" },
        { actor: { reference: `Location/${loc.id}`, display: loc.name }, status: "accepted" },
      ],
    };
  });
}

export function demoServiceRequests(): Json[] {
  const orders = [
    { p: "pt-1002", code: "72148", display: "MRI lumbar spine without contrast" },
    { p: "pt-1003", code: "64483", display: "Transforaminal epidural injection, lumbar/sacral, single level" },
    { p: "pt-1006", code: "22551", display: "Anterior cervical discectomy & fusion, single interspace" },
    { p: "pt-1004", code: "62321", display: "Interlaminar epidural injection, cervical/thoracic, with imaging" },
  ];
  return orders.map((o, i) => ({
    resourceType: "ServiceRequest",
    id: `sr-${5000 + i}`,
    status: "active",
    intent: "order",
    code: { coding: [{ system: "http://www.ama-assn.org/go/cpt", code: o.code, display: o.display }] },
    subject: { reference: `Patient/${o.p}` },
    requester: { reference: `Practitioner/${PROVIDERS[i % 3].id}` },
    authoredOn: "2026-11-12",
  }));
}

export function bundle(resources: Json[], type: "searchset" | "transaction" | "collection" = "searchset"): Json {
  return {
    resourceType: "Bundle",
    type,
    total: resources.length,
    timestamp: new Date().toISOString(),
    entry: resources.map((r) => ({ fullUrl: `${FHIR_BASE}/${r.resourceType}/${r.id}`, resource: r })),
  };
}

export function capabilityStatement(): Json {
  const search = (names: string[]) => names.map((n) => ({ name: n, type: n === "date" ? "date" : "string" }));
  return {
    resourceType: "CapabilityStatement",
    status: "active",
    date: "2026-11-15",
    kind: "instance",
    fhirVersion: "4.0.1",
    format: ["application/fhir+json"],
    implementation: { description: "Meridian Orthopedics & Spine — demo FHIR facade (synthetic data)" },
    rest: [
      {
        mode: "server",
        security: {
          service: [{ coding: [{ system: "http://terminology.hl7.org/CodeSystem/restful-security-service", code: "SMART-on-FHIR" }] }],
          description: "Production: SMART Backend Services (OAuth2 client-credentials, JWT assertion). Demo: open, read-only.",
        },
        resource: [
          { type: "Patient", interaction: [{ code: "read" }, { code: "search-type" }], searchParam: search(["name", "identifier", "birthdate"]) },
          { type: "Coverage", interaction: [{ code: "read" }, { code: "search-type" }], searchParam: search(["patient"]) },
          { type: "Appointment", interaction: [{ code: "read" }, { code: "search-type" }], searchParam: search(["date", "patient"]) },
          { type: "ServiceRequest", interaction: [{ code: "read" }, { code: "search-type" }], searchParam: search(["patient"]) },
          { type: "Practitioner", interaction: [{ code: "read" }, { code: "search-type" }] },
        ],
      },
    ],
  };
}

export function practitioners(): Json[] {
  return PROVIDERS.map((p) => ({
    resourceType: "Practitioner",
    id: p.id,
    identifier: [{ system: "http://hl7.org/fhir/sid/us-npi", value: p.npi }],
    name: [{ text: p.name }],
    qualification: [{ code: { text: p.specialty } }],
  }));
}

export function operationOutcome(diagnostics: string, code = "not-found"): Json {
  return { resourceType: "OperationOutcome", issue: [{ severity: "error", code, diagnostics }] };
}
