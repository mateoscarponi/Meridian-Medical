//! Minimal FHIR R4 (US Core-flavored) builders used by the mock FHIR server and the UI.

use super::patients::{DemoPatient, LOCATION_IDS, PATIENTS, PROVIDERS};
use super::util::now_iso;
use serde_json::{json, Value};

pub const FHIR_BASE: &str = "/api/fhir";

pub fn to_fhir_patient(p: &DemoPatient) -> Value {
    json!({
        "resourceType": "Patient",
        "id": p.id,
        "meta": { "profile": ["http://hl7.org/fhir/us/core/StructureDefinition/us-core-patient"] },
        "identifier": [{
            "use": "usual",
            "type": { "coding": [{ "system": "http://terminology.hl7.org/CodeSystem/v2-0203", "code": "MR" }] },
            "system": "urn:oid:2.16.840.1.113883.3.0000.1",
            "value": p.mrn,
        }],
        "name": [{ "use": "official", "family": p.family, "given": [p.given] }],
        "telecom": [{ "system": "phone", "value": p.phone, "use": "mobile" }],
        "gender": p.gender,
        "birthDate": p.birth_date,
        "address": [{ "city": p.city, "state": "NY", "country": "US" }],
        "communication": [{
            "language": { "coding": [{ "system": "urn:ietf:bcp:47", "code": if p.language == "es" { "es" } else { "en" } }] },
            "preferred": true,
        }],
    })
}

pub fn to_fhir_coverage(p: &DemoPatient) -> Value {
    json!({
        "resourceType": "Coverage",
        "id": format!("cov-{}", p.id),
        "status": "active",
        "type": { "text": p.payer_type.as_str() },
        "subscriberId": p.member_id,
        "beneficiary": { "reference": format!("Patient/{}", p.id) },
        "payor": [{ "display": p.payer }],
        "period": { "start": "2026-01-01", "end": "2026-12-31" },
    })
}

/// Same day as the Smart Scheduling board (Tue, Nov 17): patient, time, provider index, location index, visit.
const SCHEDULE: [(&str, &str, usize, usize, &str); 10] = [
    ("pt-1001", "08:15", 2, 1, "Follow-up"),
    ("pt-1008", "08:45", 3, 2, "Follow-up"),
    ("pt-1009", "09:30", 0, 2, "New patient consult"),
    ("pt-1005", "10:00", 1, 3, "Follow-up"),
    ("pt-1007", "10:45", 3, 1, "New patient consult"),
    ("pt-1010", "11:30", 3, 1, "Follow-up"),
    ("pt-1004", "13:00", 2, 1, "Procedure (ESI)"),
    ("pt-1006", "14:15", 0, 0, "Pre-op"),
    ("pt-1002", "15:00", 2, 1, "Follow-up"),
    ("pt-1003", "15:45", 0, 3, "Follow-up"),
];

pub fn demo_appointments(date: &str) -> Vec<Value> {
    SCHEDULE
        .iter()
        .enumerate()
        .map(|(i, (pid, time, prov_idx, loc_idx, visit))| {
            let p = PATIENTS.iter().find(|x| x.id == *pid).expect("scheduled patient exists");
            let prov = &PROVIDERS[*prov_idx];
            let loc = &LOCATION_IDS[*loc_idx];
            json!({
                "resourceType": "Appointment",
                "id": format!("appt-{}", 3000 + i),
                "status": "booked",
                "serviceType": [{ "text": visit }],
                "reasonCode": [{ "coding": [{ "system": "http://hl7.org/fhir/sid/icd-10-cm", "code": p.icd10, "display": p.condition }] }],
                "start": format!("{date}T{time}:00-05:00"),
                "minutesDuration": if visit.starts_with("New") { 40 } else { 20 },
                "participant": [
                    { "actor": { "reference": format!("Patient/{}", p.id), "display": format!("{} {}", p.given, p.family) }, "status": "accepted" },
                    { "actor": { "reference": format!("Practitioner/{}", prov.id), "display": prov.name }, "status": "accepted" },
                    { "actor": { "reference": format!("Location/{}", loc.id), "display": loc.name }, "status": "accepted" },
                ],
            })
        })
        .collect()
}

pub fn demo_service_requests() -> Vec<Value> {
    let orders = [
        ("pt-1002", "72148", "MRI lumbar spine without contrast"),
        ("pt-1003", "64483", "Transforaminal epidural injection, lumbar/sacral, single level"),
        ("pt-1006", "22551", "Anterior cervical discectomy & fusion, single interspace"),
        ("pt-1004", "62321", "Interlaminar epidural injection, cervical/thoracic, with imaging"),
    ];
    orders
        .iter()
        .enumerate()
        .map(|(i, (p, code, display))| {
            json!({
                "resourceType": "ServiceRequest",
                "id": format!("sr-{}", 5000 + i),
                "status": "active",
                "intent": "order",
                "code": { "coding": [{ "system": "http://www.ama-assn.org/go/cpt", "code": code, "display": display }] },
                "subject": { "reference": format!("Patient/{p}") },
                "requester": { "reference": format!("Practitioner/{}", PROVIDERS[i % 3].id) },
                "authoredOn": "2026-11-12",
            })
        })
        .collect()
}

pub fn bundle(resources: Vec<Value>, kind: &str) -> Value {
    let total = resources.len();
    let entry: Vec<Value> = resources
        .into_iter()
        .map(|r| {
            let url = format!("{FHIR_BASE}/{}/{}", r["resourceType"].as_str().unwrap_or(""), r["id"].as_str().unwrap_or(""));
            json!({ "fullUrl": url, "resource": r })
        })
        .collect();
    json!({ "resourceType": "Bundle", "type": kind, "total": total, "timestamp": now_iso(), "entry": entry })
}

pub fn capability_statement() -> Value {
    let search = |names: &[&str]| -> Vec<Value> { names.iter().map(|n| json!({ "name": n, "type": if *n == "date" { "date" } else { "string" } })).collect() };
    json!({
        "resourceType": "CapabilityStatement",
        "status": "active",
        "date": "2026-11-15",
        "kind": "instance",
        "fhirVersion": "4.0.1",
        "format": ["application/fhir+json"],
        "implementation": { "description": "Meridian Orthopedics & Spine — demo FHIR facade (synthetic data)" },
        "rest": [{
            "mode": "server",
            "security": {
                "service": [{ "coding": [{ "system": "http://terminology.hl7.org/CodeSystem/restful-security-service", "code": "SMART-on-FHIR" }] }],
                "description": "Production: SMART Backend Services (OAuth2 client-credentials, JWT assertion). Demo: open, read-only.",
            },
            "resource": [
                { "type": "Patient", "interaction": [{ "code": "read" }, { "code": "search-type" }], "searchParam": search(&["name", "identifier", "birthdate"]) },
                { "type": "Coverage", "interaction": [{ "code": "read" }, { "code": "search-type" }], "searchParam": search(&["patient"]) },
                { "type": "Appointment", "interaction": [{ "code": "read" }, { "code": "search-type" }], "searchParam": search(&["date", "patient"]) },
                { "type": "ServiceRequest", "interaction": [{ "code": "read" }, { "code": "search-type" }], "searchParam": search(&["patient"]) },
                { "type": "Practitioner", "interaction": [{ "code": "read" }, { "code": "search-type" }] },
            ],
        }],
    })
}

pub fn practitioners() -> Vec<Value> {
    PROVIDERS
        .iter()
        .map(|p| {
            json!({
                "resourceType": "Practitioner",
                "id": p.id,
                "identifier": [{ "system": "http://hl7.org/fhir/sid/us-npi", "value": p.npi }],
                "name": [{ "text": p.name }],
                "qualification": [{ "code": { "text": p.specialty } }],
            })
        })
        .collect()
}

pub fn operation_outcome(diagnostics: &str, code: &str) -> Value {
    json!({ "resourceType": "OperationOutcome", "issue": [{ "severity": "error", "code": code, "diagnostics": diagnostics }] })
}
