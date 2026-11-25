//! JSON / FHIR / HL7 API routes (Axum).
//!
//! - POST /api/ai        LLM endpoint for the Clinical Scribe and Patient Messages tools
//! - GET  /api/fhir/...  read-only FHIR R4 facade over the synthetic dataset
//! - POST /api/hl7       HL7 v2 inbound endpoint: raw HL7 -> { ack, fhir, parsed summary, timing }

use super::audit::audit;
use crate::domain::fhir::*;
use crate::domain::hl7::{build_ack, hl7_to_fhir, parse_hl7};
use crate::domain::patients::PATIENTS;
use crate::domain::phi::{find_phi, redact_with};
use crate::domain::util::{now_ms, round2, take_chars};
use axum::{
    body::Bytes,
    extract::{Path, Query},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::collections::HashMap;

pub fn routes<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/ai", post(ai))
        .route("/api/fhir/{*path}", get(fhir))
        .route("/api/hl7", post(hl7))
}

// ------------------------------------------------------------------ /api/ai

const SYSTEM_SCRIBE: &str = "You are a clinical documentation assistant for an orthopedic spine practice. Convert the visit transcript into a concise SOAP note for provider review.
Sections, each as a plain-text heading on its own line: SUBJECTIVE, OBJECTIVE, ASSESSMENT, PLAN.
In ASSESSMENT list likely ICD-10-CM codes with the code first (e.g. \"M51.16 – Intervertebral disc disorder with radiculopathy, lumbar region\").
In PLAN, list orders and note which ones typically need prior authorization.
Only document what the transcript supports; write \"not discussed\" otherwise. No markdown symbols. Under 350 words.";

const SYSTEM_REPLY: &str = "You draft replies from an orthopedic & spine practice front desk to patient messages, for staff to approve before sending.
Be warm, brief (under 90 words) and specific. Never give medical advice or diagnoses. If the message describes possible emergency symptoms (loss of bladder/bowel control, saddle numbness, rapidly worsening weakness, fever after surgery), tell the patient to call 911 or go to the nearest emergency room now and that the on-call provider is being notified.
Reply in the same language the patient wrote in. Sign as \"Meridian Orthopedics & Spine Care Team\". Plain text only.";

fn bad_request(msg: &str) -> Response {
    (StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))).into_response()
}

async fn ai(body: Bytes) -> Response {
    let Ok(body) = serde_json::from_slice::<Value>(&body) else { return bad_request("Invalid JSON body") };
    let task = body.get("task").and_then(Value::as_str).unwrap_or("");
    let input = take_chars(body.get("input").and_then(Value::as_str).unwrap_or(""), 8000);
    let system = match task {
        "scribe" => SYSTEM_SCRIBE,
        "reply" => SYSTEM_REPLY,
        _ => "",
    };
    if system.is_empty() || input.trim().is_empty() {
        return bad_request("task must be 'scribe' or 'reply' and input is required");
    }

    let key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
    if key.is_empty() {
        audit("/api/ai", task, None, "allowed", Some("local engine"));
        return Json(json!({ "engine": "local" })).into_response();
    }

    // Minimum necessary: strip direct identifiers before the text leaves our infrastructure,
    // even though the LLM vendor is covered by a BAA with zero data retention.
    let hits = find_phi(&input);
    let safe_input = redact_with(&input, &hits);
    audit("/api/ai", task, None, "allowed", Some(&format!("llm call, {} identifiers redacted", hits.len())));

    // Rust has no official Anthropic SDK, so this is the documented raw HTTP request.
    let req = json!({
        "model": "claude-opus-5-5",
        "max_tokens": 4000,
        "fallbacks": "default",
        "output_config": { "effort": "low" },
        "system": system,
        "messages": [{ "role": "user", "content": safe_input }],
    });
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(120)).build();
    let res = match client {
        Ok(c) => {
            c.post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", key)
                .header("anthropic-version", "2026-10-21")
                .header("anthropic-beta", "server-side-fallback-2026-07-01")
                .json(&req)
                .send()
                .await
        }
        Err(e) => Err(e),
    };
    let fail = |note: String| {
        audit("/api/ai", task, None, "error", Some(&note));
        Json(json!({ "engine": "local", "note": note })).into_response()
    };
    let res = match res {
        Ok(r) => r,
        Err(_) => return fail("LLM unavailable; used on-device generator.".into()),
    };
    let status = res.status();
    if !status.is_success() {
        return fail(match status.as_u16() {
            401 => "Invalid API key; used on-device generator.".into(),
            429 => "Rate limited; used on-device generator.".into(),
            s => format!("LLM error {s}; used on-device generator."),
        });
    }
    let Ok(msg) = res.json::<Value>().await else { return fail("LLM unavailable; used on-device generator.".into()) };
    if msg["stop_reason"] == "refusal" {
        return Json(json!({ "engine": "local", "note": "Model declined; used on-device generator." })).into_response();
    }
    let text = msg["content"]
        .as_array()
        .map(|blocks| blocks.iter().filter(|b| b["type"] == "text").filter_map(|b| b["text"].as_str()).collect::<Vec<_>>().join("\n"))
        .unwrap_or_default()
        .trim()
        .to_string();
    if text.is_empty() {
        return Json(json!({ "engine": "local", "note": "Empty response; used on-device generator." })).into_response();
    }
    Json(json!({ "engine": "claude", "model": msg["model"], "text": text, "phiRedacted": hits.len() })).into_response()
}

// ------------------------------------------------------------------ /api/fhir

fn fhir_json(body: &Value, status: StatusCode) -> Response {
    let text = serde_json::to_string_pretty(body).unwrap_or_default();
    (
        status,
        [
            (header::CONTENT_TYPE, "application/fhir+json; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store"),
            (header::HeaderName::from_static("x-demo-data"), "synthetic"),
        ],
        text,
    )
        .into_response()
}

fn ok(v: Value) -> Response {
    fhir_json(&v, StatusCode::OK)
}
fn not_found(msg: &str, code: &str) -> Response {
    fhir_json(&operation_outcome(msg, code), StatusCode::NOT_FOUND)
}

fn ref_of<'a>(r: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut cur = r;
    for p in path {
        cur = cur.get(*p)?;
    }
    cur.as_str()
}

async fn fhir(Path(path): Path<String>, Query(params): Query<HashMap<String, String>>) -> Response {
    let mut parts = path.split('/').filter(|s| !s.is_empty());
    let kind = parts.next().unwrap_or("").to_string();
    let id = parts.next().map(str::to_string);
    let q = |k: &str| params.get(k).map(|v| v.trim().to_lowercase());
    let patient_param = || q("patient").map(|p| p.strip_prefix("patient/").map(str::to_string).unwrap_or(p));
    audit("/api/fhir", if id.is_some() { "read" } else { "search" }, Some(&kind), "allowed", None);

    match kind.as_str() {
        "metadata" => ok(capability_statement()),
        "Patient" => {
            if let Some(id) = id {
                return match PATIENTS.iter().find(|p| p.id == id) {
                    Some(p) => ok(to_fhir_patient(p)),
                    None => not_found(&format!("Patient/{id} not found"), "not-found"),
                };
            }
            let (name, ident, dob) = (q("name"), q("identifier"), q("birthdate"));
            let list: Vec<Value> = PATIENTS
                .iter()
                .filter(|p| name.as_ref().is_none_or(|n| n.is_empty() || format!("{} {}", p.given, p.family).to_lowercase().contains(n.as_str())))
                .filter(|p| ident.as_ref().is_none_or(|i| i.is_empty() || p.mrn.to_lowercase() == *i || p.member_id.to_lowercase() == *i))
                .filter(|p| dob.as_ref().is_none_or(|d| d.is_empty() || p.birth_date == d))
                .map(to_fhir_patient)
                .collect();
            ok(bundle(list, "searchset"))
        }
        "Coverage" => {
            let pat = patient_param().filter(|p| !p.is_empty());
            let list: Vec<Value> = PATIENTS.iter().filter(|p| pat.as_ref().is_none_or(|x| p.id == x)).map(to_fhir_coverage).collect();
            if let Some(id) = id {
                return match list.into_iter().find(|c| c["id"] == id.as_str()) {
                    Some(c) => ok(c),
                    None => not_found(&format!("Coverage/{id} not found"), "not-found"),
                };
            }
            ok(bundle(list, "searchset"))
        }
        "Appointment" => {
            let date = params.get("date").cloned().unwrap_or_else(|| "2026-11-17".into());
            let date = date.strip_prefix("eq").or_else(|| date.strip_prefix("ge")).or_else(|| date.strip_prefix("le")).unwrap_or(&date);
            let mut list = demo_appointments(&take_chars(date, 10));
            if let Some(pat) = patient_param().filter(|p| !p.is_empty()) {
                let want = format!("patient/{pat}");
                list.retain(|a| {
                    a["participant"].as_array().is_some_and(|ps| ps.iter().any(|p| ref_of(p, &["actor", "reference"]).map(str::to_lowercase).as_deref() == Some(want.as_str())))
                });
            }
            if let Some(id) = id {
                return match list.into_iter().find(|a| a["id"] == id.as_str()) {
                    Some(a) => ok(a),
                    None => not_found(&format!("Appointment/{id} not found"), "not-found"),
                };
            }
            ok(bundle(list, "searchset"))
        }
        "ServiceRequest" => {
            let mut list = demo_service_requests();
            if let Some(pat) = patient_param().filter(|p| !p.is_empty()) {
                let want = format!("patient/{pat}");
                list.retain(|s| ref_of(s, &["subject", "reference"]).map(str::to_lowercase).as_deref() == Some(want.as_str()));
            }
            ok(bundle(list, "searchset"))
        }
        "Practitioner" => ok(bundle(practitioners(), "searchset")),
        _ => not_found(&format!("Resource type '{kind}' is not supported by this server. See /api/fhir/metadata."), "not-supported"),
    }
}

// ------------------------------------------------------------------ /api/hl7

async fn hl7(body: Bytes) -> Response {
    let t0 = now_ms();
    let text = String::from_utf8_lossy(&body);
    let text = take_chars(&text, 100_000);
    let msg = parse_hl7(&text);
    if !msg.errors.is_empty() && msg.segments.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "ok": false, "errors": msg.errors }))).into_response();
    }
    let fhir = hl7_to_fhir(&msg);
    let ok = msg.errors.is_empty();
    let ack = build_ack(&msg, ok, if ok { "" } else { &msg.errors[0] });
    audit("/api/hl7", "ingest", Some(&format!("{} {}", msg.kind, msg.control_id)), if ok { "allowed" } else { "error" }, None);
    Json(json!({
        "ok": ok,
        "messageType": msg.kind,
        "controlId": msg.control_id,
        "segments": msg.segments.iter().map(|s| s.id.clone()).collect::<Vec<_>>(),
        "warnings": msg.errors,
        "ack": ack,
        "fhir": fhir,
        "processingMs": round2(now_ms() - t0),
    }))
    .into_response()
}
