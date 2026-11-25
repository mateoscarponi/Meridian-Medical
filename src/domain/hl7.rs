//! HL7 v2.x parser, field dictionary, FHIR R4 mapper and ACK builder.
//! Handles custom encoding characters from MSH-1/MSH-2, repetitions (~), components (^)
//! and sub-components (&). Escape sequences (\F\, \S\, \T\, \R\, \E\) are decoded.

use super::util::{js_number, json_num, now_hl7_ts, slice};
use serde::Serialize;
use serde_json::{json, Map, Value};

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Hl7Field {
    /// HL7 field number (MSH-1 is the field separator itself)
    pub index: usize,
    pub name: String,
    pub raw: String,
    /// repetitions -> components
    pub components: Vec<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Hl7Segment {
    pub id: String,
    pub label: String,
    pub fields: Vec<Hl7Field>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Default)]
pub struct Hl7Message {
    pub segments: Vec<Hl7Segment>,
    /// e.g. ADT^A04
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(rename = "controlId")]
    pub control_id: String,
    pub version: String,
    #[serde(rename = "sendingApp")]
    pub sending_app: String,
    #[serde(rename = "sendingFacility")]
    pub sending_facility: String,
    pub errors: Vec<String>,
}

pub fn segment_label(id: &str) -> &'static str {
    match id {
        "MSH" => "Message Header",
        "EVN" => "Event Type",
        "PID" => "Patient Identification",
        "PD1" => "Patient Additional Demographic",
        "NK1" => "Next of Kin",
        "PV1" => "Patient Visit",
        "IN1" => "Insurance",
        "SCH" => "Scheduling Activity Information",
        "AIG" => "Appointment Information – General Resource",
        "AIL" => "Appointment Information – Location",
        "AIP" => "Appointment Information – Personnel",
        "ORC" => "Common Order",
        "OBR" => "Observation Request",
        "OBX" => "Observation / Result",
        "DG1" => "Diagnosis",
        "NTE" => "Notes and Comments",
        "MSA" => "Message Acknowledgment",
        _ => "Custom / Z-segment",
    }
}

fn field_name(seg: &str, n: usize) -> Option<&'static str> {
    Some(match (seg, n) {
        ("MSH", 1) => "Field Separator",
        ("MSH", 2) => "Encoding Characters",
        ("MSH", 3) => "Sending Application",
        ("MSH", 4) => "Sending Facility",
        ("MSH", 5) => "Receiving Application",
        ("MSH", 6) => "Receiving Facility",
        ("MSH", 7) => "Date/Time of Message",
        ("MSH", 9) => "Message Type",
        ("MSH", 10) => "Message Control ID",
        ("MSH", 11) => "Processing ID",
        ("MSH", 12) => "Version ID",
        ("EVN", 1) => "Event Type Code",
        ("EVN", 2) => "Recorded Date/Time",
        ("PID", 1) => "Set ID",
        ("PID", 3) => "Patient Identifier List",
        ("PID", 5) => "Patient Name",
        ("PID", 7) => "Date of Birth",
        ("PID", 8) => "Administrative Sex",
        ("PID", 11) => "Patient Address",
        ("PID", 13) => "Phone – Home",
        ("PID", 15) => "Primary Language",
        ("PID", 18) => "Patient Account Number",
        ("PID", 19) => "SSN",
        ("PV1", 1) => "Set ID",
        ("PV1", 2) => "Patient Class",
        ("PV1", 3) => "Assigned Location",
        ("PV1", 7) => "Attending Doctor",
        ("PV1", 8) => "Referring Doctor",
        ("PV1", 10) => "Hospital Service",
        ("PV1", 19) => "Visit Number",
        ("PV1", 44) => "Admit Date/Time",
        ("IN1", 1) => "Set ID",
        ("IN1", 2) => "Insurance Plan ID",
        ("IN1", 3) => "Insurance Company ID",
        ("IN1", 4) => "Insurance Company Name",
        ("IN1", 36) => "Policy Number",
        ("SCH", 1) => "Placer Appointment ID",
        ("SCH", 2) => "Filler Appointment ID",
        ("SCH", 6) => "Event Reason",
        ("SCH", 7) => "Appointment Reason",
        ("SCH", 8) => "Appointment Type",
        ("SCH", 9) => "Appointment Duration",
        ("SCH", 10) => "Duration Units",
        ("SCH", 11) => "Appointment Timing",
        ("SCH", 25) => "Filler Status Code",
        ("AIG", 1) => "Set ID",
        ("AIG", 3) => "Resource ID",
        ("AIG", 4) => "Resource Type",
        ("AIL", 1) => "Set ID",
        ("AIL", 3) => "Location Resource ID",
        ("AIP", 1) => "Set ID",
        ("AIP", 3) => "Personnel Resource ID",
        ("AIP", 4) => "Resource Role",
        ("ORC", 1) => "Order Control",
        ("ORC", 2) => "Placer Order Number",
        ("ORC", 3) => "Filler Order Number",
        ("ORC", 12) => "Ordering Provider",
        ("OBR", 1) => "Set ID",
        ("OBR", 2) => "Placer Order Number",
        ("OBR", 3) => "Filler Order Number",
        ("OBR", 4) => "Universal Service ID",
        ("OBR", 7) => "Observation Date/Time",
        ("OBR", 16) => "Ordering Provider",
        ("OBR", 22) => "Results Rpt/Status Chng",
        ("OBR", 25) => "Result Status",
        ("OBX", 1) => "Set ID",
        ("OBX", 2) => "Value Type",
        ("OBX", 3) => "Observation Identifier",
        ("OBX", 5) => "Observation Value",
        ("OBX", 6) => "Units",
        ("OBX", 8) => "Abnormal Flags",
        ("OBX", 11) => "Observation Result Status",
        ("OBX", 14) => "Date/Time of Observation",
        ("DG1", 1) => "Set ID",
        ("DG1", 2) => "Coding Method",
        ("DG1", 3) => "Diagnosis Code",
        ("DG1", 6) => "Diagnosis Type",
        ("NTE", 1) => "Set ID",
        ("NTE", 3) => "Comment",
        ("MSA", 1) => "Acknowledgment Code",
        ("MSA", 2) => "Message Control ID",
        ("MSA", 3) => "Text Message",
        _ => return None,
    })
}

fn valid_segment_id(id: &str) -> bool {
    let b = id.as_bytes();
    b.len() == 3 && b[0].is_ascii_uppercase() && b[1..].iter().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

pub fn parse_hl7(input: &str) -> Hl7Message {
    let mut errors: Vec<String> = Vec::new();
    let text = input.replace("\r\n", "\r").replace('\n', "\r");
    let text = text.trim();
    let lines: Vec<&str> = text.split('\r').map(str::trim).filter(|l| !l.is_empty()).collect();
    if lines.is_empty() || !lines[0].starts_with("MSH") {
        return Hl7Message { errors: vec!["Message must start with an MSH segment.".into()], ..Default::default() };
    }
    let head: Vec<char> = lines[0].chars().collect();
    let field_sep = head.get(3).copied().unwrap_or('|');
    let enc: Vec<char> = head.iter().skip(4).take(4).copied().collect();
    let comp_sep = enc.first().copied().unwrap_or('^');
    let rep_sep = enc.get(1).copied().unwrap_or('~');
    let esc = enc.get(2).copied().unwrap_or('\\');
    let sub_sep = enc.get(3).copied().unwrap_or('&');

    let unescape = |s: &str| -> String {
        s.replace(&format!("{esc}F{esc}"), &field_sep.to_string())
            .replace(&format!("{esc}S{esc}"), &comp_sep.to_string())
            .replace(&format!("{esc}R{esc}"), &rep_sep.to_string())
            .replace(&format!("{esc}T{esc}"), &sub_sep.to_string())
            .replace(&format!("{esc}E{esc}"), &esc.to_string())
    };

    let segments: Vec<Hl7Segment> = lines
        .iter()
        .enumerate()
        .map(|(li, line)| {
            let parts: Vec<&str> = line.split(field_sep).collect();
            let id = parts[0].to_string();
            if !valid_segment_id(&id) {
                errors.push(format!("Line {}: \"{}\" is not a valid segment ID.", li + 1, id));
            }
            let is_msh = id == "MSH";
            // For MSH, field 1 is the separator itself and field 2 the encoding characters.
            let fs = field_sep.to_string();
            let values: Vec<&str> = if is_msh {
                std::iter::once(fs.as_str()).chain(parts.iter().skip(1).copied()).collect()
            } else {
                parts.iter().skip(1).copied().collect()
            };
            let fields = values
                .iter()
                .enumerate()
                .filter(|(_, raw)| !raw.is_empty())
                .map(|(i, raw)| {
                    let index = i + 1;
                    let components = if is_msh && index <= 2 {
                        vec![vec![raw.to_string()]]
                    } else {
                        raw.split(rep_sep)
                            .map(|rep| rep.split(comp_sep).map(|c| unescape(&c.split(sub_sep).collect::<Vec<_>>().join(" & "))).collect())
                            .collect()
                    };
                    Hl7Field {
                        index,
                        name: field_name(&id, index).map(str::to_string).unwrap_or_else(|| format!("{id}-{index}")),
                        raw: raw.to_string(),
                        components,
                    }
                })
                .collect();
            Hl7Segment { label: segment_label(&id).to_string(), id, fields }
        })
        .collect();

    let msh = segments.first();
    let get = |n: usize| msh.and_then(|s| s.fields.iter().find(|f| f.index == n));
    let kind = get(9).map(|f| f.components[0].iter().take(2).cloned().collect::<Vec<_>>().join("^")).unwrap_or_default();
    if kind.is_empty() {
        errors.push("MSH-9 (Message Type) is missing.".into());
    }
    let control_id = get(10).map(|f| f.raw.clone()).unwrap_or_default();
    if control_id.is_empty() {
        errors.push("MSH-10 (Message Control ID) is missing.".into());
    }
    let version = get(12).map(|f| f.raw.clone()).unwrap_or_default();
    let sending_app = get(3).map(|f| f.components[0][0].clone()).unwrap_or_default();
    let sending_facility = get(4).map(|f| f.components[0][0].clone()).unwrap_or_default();

    Hl7Message { segments, kind, control_id, version, sending_app, sending_facility, errors }
}

fn seg<'a>(msg: &'a Hl7Message, id: &str) -> Option<&'a Hl7Segment> {
    msg.segments.iter().find(|s| s.id == id)
}
fn segs<'a>(msg: &'a Hl7Message, id: &'a str) -> impl Iterator<Item = &'a Hl7Segment> {
    msg.segments.iter().filter(move |s| s.id == id)
}
/// Component `c` (1-based) of repetition `rep` of field `field`; "" when absent.
pub fn comp(s: Option<&Hl7Segment>, field: usize, c: usize) -> String {
    s.and_then(|s| s.fields.iter().find(|f| f.index == field))
        .and_then(|f| f.components.first())
        .and_then(|r| r.get(c.wrapping_sub(1)))
        .cloned()
        .unwrap_or_default()
}

fn hl7_date(ts: &str) -> Value {
    if ts.chars().count() < 8 {
        return Value::Null;
    }
    let d = format!("{}-{}-{}", slice(ts, 0, 4), slice(ts, 4, 6), slice(ts, 6, 8));
    if ts.chars().count() >= 12 {
        return Value::String(format!("{d}T{}:{}:00-05:00", slice(ts, 8, 10), slice(ts, 10, 12)));
    }
    Value::String(d)
}

fn or(a: String, b: String) -> String {
    if a.is_empty() {
        b
    } else {
        a
    }
}
fn s_or_null(s: String) -> Value {
    if s.is_empty() {
        Value::Null
    } else {
        Value::String(s)
    }
}
fn join_nonempty(parts: &[String], sep: &str) -> String {
    parts.iter().filter(|p| !p.is_empty()).cloned().collect::<Vec<_>>().join(sep)
}

/// Removes null (undefined), "", [] and {} recursively, like the original `clean`.
pub fn clean(v: Value) -> Value {
    match v {
        Value::Array(a) => Value::Array(a.into_iter().map(clean).filter(|v| !v.is_null()).collect()),
        Value::Object(o) => {
            let mut out = Map::new();
            for (k, v) in o {
                let c = clean(v);
                let empty = match &c {
                    Value::Null => true,
                    Value::String(s) => s.is_empty(),
                    Value::Array(a) => a.is_empty(),
                    Value::Object(m) => m.is_empty(),
                    _ => false,
                };
                if !empty {
                    out.insert(k, c);
                }
            }
            Value::Object(out)
        }
        other => other,
    }
}

pub fn hl7_to_fhir(msg: &Hl7Message) -> Value {
    let mut entries: Vec<Value> = Vec::new();
    let pid = seg(msg, "PID");
    let pid3 = comp(pid, 3, 1);
    let patient_id = if pid3.is_empty() {
        "pt-unknown".to_string()
    } else {
        format!("pt-{}", pid3.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_lowercase())
    };
    let pref = format!("Patient/{patient_id}");

    if pid.is_some() {
        let sex = comp(pid, 8, 1);
        let gender = match sex.as_str() {
            "F" => "female",
            "M" => "male",
            "" => "unknown",
            _ => "other",
        };
        let addr = comp(pid, 11, 1);
        entries.push(json!({
            "resourceType": "Patient",
            "id": patient_id,
            "identifier": [{
                "type": { "coding": [{ "code": or(comp(pid, 3, 5), "MR".into()) }] },
                "value": pid3,
                "assigner": if comp(pid, 3, 4).is_empty() { Value::Null } else { json!({ "display": comp(pid, 3, 4) }) },
            }],
            "name": [{ "family": comp(pid, 5, 1), "given": ([comp(pid, 5, 2), comp(pid, 5, 3)].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>()) }],
            "birthDate": hl7_date(&comp(pid, 7, 1)),
            "gender": gender,
            "address": if addr.is_empty() { Value::Null } else { json!([{
                "line": ([addr.clone()].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>()),
                "city": comp(pid, 11, 3), "state": comp(pid, 11, 4), "postalCode": comp(pid, 11, 5),
            }]) },
            "telecom": if comp(pid, 13, 1).is_empty() { Value::Null } else { json!([{ "system": "phone", "value": comp(pid, 13, 1) }]) },
            "communication": if comp(pid, 15, 1).is_empty() { Value::Null } else { json!([{ "language": { "text": or(comp(pid, 15, 2), comp(pid, 15, 1)) } }]) },
        }));
    }

    for in1 in segs(msg, "IN1") {
        let in1 = Some(in1);
        entries.push(json!({
            "resourceType": "Coverage",
            "id": format!("cov-{}", or(comp(in1, 1, 1), "1".into())),
            "status": "active",
            "subscriberId": s_or_null(comp(in1, 36, 1)),
            "beneficiary": { "reference": pref },
            "payor": [{ "display": or(comp(in1, 4, 1), comp(in1, 3, 1)) }],
        }));
    }

    let pv1 = seg(msg, "PV1");
    if pv1.is_some() && msg.kind.starts_with("ADT") {
        let cls = comp(pv1, 2, 1);
        let attending = join_nonempty(&[comp(pv1, 7, 3), comp(pv1, 7, 2)], " ");
        entries.push(json!({
            "resourceType": "Encounter",
            "id": format!("enc-{}", or(comp(pv1, 19, 1), msg.control_id.clone())),
            "status": if msg.kind == "ADT^A03" { "finished" } else { "arrived" },
            "class": { "system": "http://terminology.hl7.org/CodeSystem/v3-ActCode", "code": match cls.as_str() { "I" => "IMP", "E" => "EMER", _ => "AMB" } },
            "subject": { "reference": pref },
            "participant": if comp(pv1, 7, 1).is_empty() { Value::Null } else { json!([{ "individual": { "display": or(attending, comp(pv1, 7, 1)) } }]) },
            "location": if comp(pv1, 3, 1).is_empty() { Value::Null } else { json!([{ "location": { "display": join_nonempty(&[comp(pv1, 3, 1), comp(pv1, 3, 4)], " · ") } }]) },
            "period": { "start": hl7_date(&comp(pv1, 44, 1)) },
        }));
    }

    if let Some(sch) = seg(msg, "SCH") {
        let sch = Some(sch);
        let aip = seg(msg, "AIP");
        let ail = seg(msg, "AIL");
        let status = or(comp(sch, 25, 1), "Booked".into()).to_lowercase();
        let mut participant = vec![json!({ "actor": { "reference": pref }, "status": "accepted" })];
        if aip.is_some() {
            participant.push(json!({ "actor": { "display": or(join_nonempty(&[comp(aip, 3, 3), comp(aip, 3, 2)], " "), comp(aip, 3, 1)) }, "status": "accepted" }));
        }
        if ail.is_some() {
            participant.push(json!({ "actor": { "display": or(comp(ail, 3, 1), comp(ail, 3, 1)) }, "status": "accepted" }));
        }
        let minutes = js_number(&comp(sch, 9, 1)).filter(|n| *n != 0.0).map(json_num).unwrap_or(Value::Null);
        entries.push(json!({
            "resourceType": "Appointment",
            "id": format!("appt-{}", or(comp(sch, 1, 1), msg.control_id.clone())),
            "status": if status.contains("cancel") { "cancelled" } else if status.contains("no") { "noshow" } else { "booked" },
            "appointmentType": if comp(sch, 8, 1).is_empty() { Value::Null } else { json!({ "text": or(comp(sch, 8, 2), comp(sch, 8, 1)) }) },
            "reasonCode": if comp(sch, 7, 1).is_empty() { Value::Null } else { json!([{ "text": or(comp(sch, 7, 2), comp(sch, 7, 1)) }]) },
            "start": hl7_date(&comp(sch, 11, 4)),
            "minutesDuration": minutes,
            "participant": participant,
        }));
    }

    if let Some(obr) = seg(msg, "OBR") {
        let obr = Some(obr);
        let mut obs_ids: Vec<String> = Vec::new();
        for (i, obx) in segs(msg, "OBX").enumerate() {
            let obx = Some(obx);
            let id = format!("obs-{}-{}", msg.control_id, i + 1);
            obs_ids.push(id.clone());
            let value_type = comp(obx, 2, 1);
            let value = comp(obx, 5, 1);
            let mut o = json!({
                "resourceType": "Observation",
                "id": id,
                "status": if comp(obx, 11, 1) == "F" { "final" } else { "preliminary" },
                "code": { "coding": [{ "code": comp(obx, 3, 1), "display": comp(obx, 3, 2), "system": if comp(obx, 3, 3) == "LN" { Value::from("http://loinc.org") } else { Value::Null } }] },
                "subject": { "reference": pref },
            });
            let map = o.as_object_mut().expect("object");
            if value_type == "NM" {
                let v = js_number(&value).map(json_num).unwrap_or(Value::Null);
                map.insert("valueQuantity".into(), json!({ "value": v, "unit": comp(obx, 6, 1) }));
            } else {
                map.insert("valueString".into(), Value::String(value));
            }
            map.insert(
                "interpretation".into(),
                if comp(obx, 8, 1).is_empty() { Value::Null } else { json!([{ "coding": [{ "code": comp(obx, 8, 1) }] }]) },
            );
            entries.push(o);
        }
        entries.push(json!({
            "resourceType": "DiagnosticReport",
            "id": format!("dr-{}", or(comp(obr, 3, 1), msg.control_id.clone())),
            "status": if comp(obr, 25, 1) == "F" { "final" } else { "preliminary" },
            "code": { "coding": [{ "code": comp(obr, 4, 1), "display": comp(obr, 4, 2) }] },
            "subject": { "reference": pref },
            "effectiveDateTime": hl7_date(&comp(obr, 7, 1)),
            "result": obs_ids.iter().map(|id| json!({ "reference": format!("Observation/{id}") })).collect::<Vec<_>>(),
        }));
    }

    for dg1 in segs(msg, "DG1") {
        let dg1 = Some(dg1);
        let code = comp(dg1, 3, 1);
        entries.push(json!({
            "resourceType": "Condition",
            "id": format!("cond-{}", code.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_').collect::<String>()),
            "code": { "coding": [{ "system": "http://hl7.org/fhir/sid/icd-10-cm", "code": code, "display": comp(dg1, 3, 2) }] },
            "subject": { "reference": pref },
        }));
    }

    let entry: Vec<Value> = entries
        .into_iter()
        .map(|r| {
            let rt = r["resourceType"].as_str().unwrap_or("").to_string();
            let id = r["id"].as_str().unwrap_or("").to_string();
            json!({ "fullUrl": format!("urn:uuid:{id}"), "resource": r, "request": { "method": "PUT", "url": format!("{rt}/{id}") } })
        })
        .collect();

    clean(json!({
        "resourceType": "Bundle",
        "type": "transaction",
        "meta": { "source": format!("{}@{}", msg.sending_app, msg.sending_facility), "tag": [{ "code": msg.kind }] },
        "entry": entry,
    }))
}

pub fn build_ack(msg: &Hl7Message, ok: bool, text: &str) -> String {
    let ts = now_hl7_ts();
    let trigger = msg.kind.split('^').nth(1).unwrap_or("");
    let version = if msg.version.is_empty() { "2.5.1" } else { msg.version.as_str() };
    let msa_text = if text.is_empty() { String::new() } else { format!("|{text}") };
    [
        format!(
            "MSH|^~\\&|MOS_INTEGRATION|MERIDIAN_ORTHO|{}|{}|{ts}||ACK^{trigger}^ACK|ACK{}|P|{version}",
            msg.sending_app, msg.sending_facility, msg.control_id
        ),
        format!("MSA|{}|{}{msa_text}", if ok { "AA" } else { "AE" }, msg.control_id),
    ]
    .join("\r")
}

pub struct SampleMessage {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub text: &'static str,
}

pub const SAMPLE_MESSAGES: [SampleMessage; 3] = [
    SampleMessage {
        id: "adt",
        label: "ADT^A04 — Register patient",
        description: "Front desk registers a new patient in the EMR; insurance and visit data travel together.",
        text: "MSH|^~\\&|ECW|MOS_GARDENCITY|MOS_INTEGRATION|MERIDIAN_ORTHO|20261116081502||ADT^A04^ADT_A01|MSG00041871|P|2.5.1\nEVN|A04|20261116081500\nPID|1||MOS-204871^^^MOS^MR||Gonzalez^Maria^L||19680412|F|||142 Fulton Ave^^Hempstead^NY^11550||(516)555-0191||spa^Spanish|||ACCT-88120\nPV1|1|O|GC^^^Garden City||||1999999901^Hale^Adrian^^^MD|1888888801^Patel^Rina^^^MD|ORT||||||||||V-3381102|||||||||||||||||||||||||20261116081500\nIN1|1|FIDELIS-MCD|FC01|Fidelis Care|||||||||||||||||||||||||||||||||FC88310442\nDG1|1|I10|M54.16^Radiculopathy, lumbar region^I10||20261116|A",
    },
    SampleMessage {
        id: "siu",
        label: "SIU^S12 — New appointment",
        description: "Scheduling system books a spine consult; the platform turns it into a FHIR Appointment and triggers reminders.",
        text: "MSH|^~\\&|ATHENA_SCHED|MOS_HUNTINGTON|MOS_INTEGRATION|MERIDIAN_ORTHO|20261116093011||SIU^S12|SCH77120|P|2.5.1\nSCH|A-55120|A-55120|||||SPINECONS^Spine surgery consultation|NEW^New patient|40|min|^^40^20261124101500^20261124105500||||||||||||||Booked\nPID|1||MOS-187205^^^MOS^MR||Chen^Linda||19520118|F|||88 Main St^^Flushing^NY^11354||(718)555-0164\nAIP|1||1999999901^Hale^Adrian^^^MD|SURGEON\nAIL|1||HU^Huntington Office",
    },
    SampleMessage {
        id: "oru",
        label: "ORU^R01 — Imaging result",
        description: "Outside imaging center sends an MRI report; findings are attached to the chart and to the pending prior authorization.",
        text: "MSH|^~\\&|RIS|NASSAU_IMAGING|MOS_INTEGRATION|MERIDIAN_ORTHO|20261115171244||ORU^R01|RAD99012|P|2.5.1\nPID|1||MOS-198344^^^MOS^MR||O'Connor^James||19750930|M\nOBR|1|ORD-7781|RAD-552019|72148^MRI lumbar spine w/o contrast^CPT|||20261115153000|||||||||1999999901^Hale^Adrian||||||20261115171000|||F\nOBX|1|TX|IMPRESSION^Impression||Right paracentral disc extrusion at L4-L5 contacting the descending right L5 nerve root.||||||F\nOBX|2|TX|FINDINGS^Findings||Moderate right lateral recess stenosis at L4-L5. No fracture. Conus terminates normally.||||||F\nOBX|3|NM|72514-3^Pain severity 0-10^LN||7|{score}|||||F",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_and_maps_samples() {
        for s in SAMPLE_MESSAGES.iter() {
            let m = parse_hl7(s.text);
            assert!(m.errors.is_empty(), "{:?}", m.errors);
            let b = hl7_to_fhir(&m);
            assert_eq!(b["resourceType"], "Bundle");
            assert!(!b["entry"].as_array().unwrap().is_empty());
        }
        let siu = hl7_to_fhir(&parse_hl7(SAMPLE_MESSAGES[1].text));
        let appt = &siu["entry"][1]["resource"];
        assert_eq!(appt["minutesDuration"], 40);
        assert_eq!(appt["start"], "2026-11-24T10:15:00-05:00");
        let oru = hl7_to_fhir(&parse_hl7(SAMPLE_MESSAGES[2].text));
        assert_eq!(oru["entry"][3]["resource"]["valueQuantity"]["value"], 7);
    }
}
