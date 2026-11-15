// HL7 v2.x parser, field dictionary, FHIR R4 mapper and ACK builder.
// Handles custom encoding characters from MSH-1/MSH-2, repetitions (~), components (^)
// and sub-components (&). Escape sequences (\F\, \S\, \T\, \R\, \E\) are decoded.

export interface Hl7Field {
  index: number; // HL7 field number (MSH-1 is the field separator itself)
  name: string;
  raw: string;
  components: string[][]; // repetitions -> components
}

export interface Hl7Segment {
  id: string;
  label: string;
  fields: Hl7Field[];
}

export interface Hl7Message {
  segments: Hl7Segment[];
  type: string; // e.g. ADT^A04
  controlId: string;
  version: string;
  sendingApp: string;
  sendingFacility: string;
  errors: string[];
}

const SEGMENT_LABELS: Record<string, string> = {
  MSH: "Message Header",
  EVN: "Event Type",
  PID: "Patient Identification",
  PD1: "Patient Additional Demographic",
  NK1: "Next of Kin",
  PV1: "Patient Visit",
  IN1: "Insurance",
  SCH: "Scheduling Activity Information",
  AIG: "Appointment Information – General Resource",
  AIL: "Appointment Information – Location",
  AIP: "Appointment Information – Personnel",
  ORC: "Common Order",
  OBR: "Observation Request",
  OBX: "Observation / Result",
  DG1: "Diagnosis",
  NTE: "Notes and Comments",
  MSA: "Message Acknowledgment",
};

const FIELD_NAMES: Record<string, Record<number, string>> = {
  MSH: { 1: "Field Separator", 2: "Encoding Characters", 3: "Sending Application", 4: "Sending Facility", 5: "Receiving Application", 6: "Receiving Facility", 7: "Date/Time of Message", 9: "Message Type", 10: "Message Control ID", 11: "Processing ID", 12: "Version ID" },
  EVN: { 1: "Event Type Code", 2: "Recorded Date/Time" },
  PID: { 1: "Set ID", 3: "Patient Identifier List", 5: "Patient Name", 7: "Date of Birth", 8: "Administrative Sex", 11: "Patient Address", 13: "Phone – Home", 15: "Primary Language", 18: "Patient Account Number", 19: "SSN" },
  PV1: { 1: "Set ID", 2: "Patient Class", 3: "Assigned Location", 7: "Attending Doctor", 8: "Referring Doctor", 10: "Hospital Service", 19: "Visit Number", 44: "Admit Date/Time" },
  IN1: { 1: "Set ID", 2: "Insurance Plan ID", 3: "Insurance Company ID", 4: "Insurance Company Name", 36: "Policy Number" },
  SCH: { 1: "Placer Appointment ID", 2: "Filler Appointment ID", 6: "Event Reason", 7: "Appointment Reason", 8: "Appointment Type", 9: "Appointment Duration", 10: "Duration Units", 11: "Appointment Timing", 25: "Filler Status Code" },
  AIG: { 1: "Set ID", 3: "Resource ID", 4: "Resource Type" },
  AIL: { 1: "Set ID", 3: "Location Resource ID" },
  AIP: { 1: "Set ID", 3: "Personnel Resource ID", 4: "Resource Role" },
  ORC: { 1: "Order Control", 2: "Placer Order Number", 3: "Filler Order Number", 12: "Ordering Provider" },
  OBR: { 1: "Set ID", 2: "Placer Order Number", 3: "Filler Order Number", 4: "Universal Service ID", 7: "Observation Date/Time", 16: "Ordering Provider", 22: "Results Rpt/Status Chng", 25: "Result Status" },
  OBX: { 1: "Set ID", 2: "Value Type", 3: "Observation Identifier", 5: "Observation Value", 6: "Units", 8: "Abnormal Flags", 11: "Observation Result Status", 14: "Date/Time of Observation" },
  DG1: { 1: "Set ID", 2: "Coding Method", 3: "Diagnosis Code", 6: "Diagnosis Type" },
  NTE: { 1: "Set ID", 3: "Comment" },
  MSA: { 1: "Acknowledgment Code", 2: "Message Control ID", 3: "Text Message" },
};

export function parseHl7(input: string): Hl7Message {
  const errors: string[] = [];
  const text = input.replace(/\r\n|\n/g, "\r").trim();
  const lines = text.split("\r").map((l) => l.trim()).filter(Boolean);
  if (!lines.length || !lines[0].startsWith("MSH")) {
    return { segments: [], type: "", controlId: "", version: "", sendingApp: "", sendingFacility: "", errors: ["Message must start with an MSH segment."] };
  }
  const fieldSep = lines[0][3] ?? "|";
  const enc = lines[0].slice(4, 8);
  const compSep = enc[0] ?? "^";
  const repSep = enc[1] ?? "~";
  const escChar = enc[2] ?? "\\";
  const subSep = enc[3] ?? "&";

  const unescape = (s: string) =>
    s
      .split(`${escChar}F${escChar}`).join(fieldSep)
      .split(`${escChar}S${escChar}`).join(compSep)
      .split(`${escChar}R${escChar}`).join(repSep)
      .split(`${escChar}T${escChar}`).join(subSep)
      .split(`${escChar}E${escChar}`).join(escChar);

  const segments: Hl7Segment[] = lines.map((line, li) => {
    const parts = line.split(fieldSep);
    const id = parts[0];
    if (!/^[A-Z][A-Z0-9]{2}$/.test(id)) errors.push(`Line ${li + 1}: "${id}" is not a valid segment ID.`);
    const fields: Hl7Field[] = [];
    const isMsh = id === "MSH";
    // For MSH, field 1 is the separator itself and field 2 the encoding characters.
    const values = isMsh ? [fieldSep, ...parts.slice(1)] : parts.slice(1);
    values.forEach((raw, i) => {
      const index = i + 1;
      if (raw === "") return;
      const components =
        isMsh && index <= 2
          ? [[raw]]
          : raw.split(repSep).map((rep) => rep.split(compSep).map((c) => unescape(c.split(subSep).join(" & "))));
      fields.push({ index, name: FIELD_NAMES[id]?.[index] ?? `${id}-${index}`, raw, components });
    });
    return { id, label: SEGMENT_LABELS[id] ?? "Custom / Z-segment", fields };
  });

  const msh = segments[0];
  const get = (seg: Hl7Segment | undefined, n: number) => seg?.fields.find((f) => f.index === n);
  const typeField = get(msh, 9);
  const type = typeField ? typeField.components[0].slice(0, 2).join("^") : "";
  if (!type) errors.push("MSH-9 (Message Type) is missing.");
  const controlId = get(msh, 10)?.raw ?? "";
  if (!controlId) errors.push("MSH-10 (Message Control ID) is missing.");

  return {
    segments,
    type,
    controlId,
    version: get(msh, 12)?.raw ?? "",
    sendingApp: get(msh, 3)?.components[0][0] ?? "",
    sendingFacility: get(msh, 4)?.components[0][0] ?? "",
    errors,
  };
}

function seg(msg: Hl7Message, id: string) {
  return msg.segments.find((s) => s.id === id);
}
function segs(msg: Hl7Message, id: string) {
  return msg.segments.filter((s) => s.id === id);
}
function comp(s: { fields: Hl7Field[] } | undefined, field: number, c = 1, rep = 0) {
  return s?.fields.find((f) => f.index === field)?.components[rep]?.[c - 1] ?? "";
}

function hl7Date(ts: string) {
  if (!ts || ts.length < 8) return undefined;
  const d = `${ts.slice(0, 4)}-${ts.slice(4, 6)}-${ts.slice(6, 8)}`;
  if (ts.length >= 12) return `${d}T${ts.slice(8, 10)}:${ts.slice(10, 12)}:00-05:00`;
  return d;
}

type Json = Record<string, unknown>;

export function hl7ToFhir(msg: Hl7Message): Json {
  const entries: Json[] = [];
  const pid = seg(msg, "PID");
  const patientId = comp(pid, 3) ? `pt-${comp(pid, 3).replace(/[^A-Za-z0-9]/g, "").toLowerCase()}` : "pt-unknown";

  if (pid) {
    const sex = comp(pid, 8);
    entries.push({
      resourceType: "Patient",
      id: patientId,
      identifier: [{ type: { coding: [{ code: comp(pid, 3, 5) || "MR" }] }, value: comp(pid, 3), assigner: comp(pid, 3, 4) ? { display: comp(pid, 3, 4) } : undefined }],
      name: [{ family: comp(pid, 5, 1), given: [comp(pid, 5, 2), comp(pid, 5, 3)].filter(Boolean) }],
      birthDate: hl7Date(comp(pid, 7)),
      gender: sex === "F" ? "female" : sex === "M" ? "male" : sex ? "other" : "unknown",
      address: comp(pid, 11) ? [{ line: [comp(pid, 11, 1)].filter(Boolean), city: comp(pid, 11, 3), state: comp(pid, 11, 4), postalCode: comp(pid, 11, 5) }] : undefined,
      telecom: comp(pid, 13) ? [{ system: "phone", value: comp(pid, 13) }] : undefined,
      communication: comp(pid, 15) ? [{ language: { text: comp(pid, 15, 2) || comp(pid, 15) } }] : undefined,
    });
  }

  for (const in1 of segs(msg, "IN1")) {
    entries.push({
      resourceType: "Coverage",
      id: `cov-${comp(in1, 1) || 1}`,
      status: "active",
      subscriberId: comp(in1, 36) || undefined,
      beneficiary: { reference: `Patient/${patientId}` },
      payor: [{ display: comp(in1, 4) || comp(in1, 3) }],
    });
  }

  const pv1 = seg(msg, "PV1");
  if (pv1 && msg.type.startsWith("ADT")) {
    const cls = comp(pv1, 2);
    entries.push({
      resourceType: "Encounter",
      id: `enc-${comp(pv1, 19) || msg.controlId}`,
      status: msg.type === "ADT^A03" ? "finished" : "arrived",
      class: { system: "http://terminology.hl7.org/CodeSystem/v3-ActCode", code: cls === "I" ? "IMP" : cls === "E" ? "EMER" : "AMB" },
      subject: { reference: `Patient/${patientId}` },
      participant: comp(pv1, 7) ? [{ individual: { display: [comp(pv1, 7, 3), comp(pv1, 7, 2)].filter(Boolean).join(" ") || comp(pv1, 7) } }] : undefined,
      location: comp(pv1, 3) ? [{ location: { display: [comp(pv1, 3, 1), comp(pv1, 3, 4)].filter(Boolean).join(" · ") } }] : undefined,
      period: { start: hl7Date(comp(pv1, 44)) },
    });
  }

  const sch = seg(msg, "SCH");
  if (sch) {
    const aip = seg(msg, "AIP");
    const ail = seg(msg, "AIL");
    const status = comp(sch, 25) || "Booked";
    entries.push({
      resourceType: "Appointment",
      id: `appt-${comp(sch, 1) || msg.controlId}`,
      status: status.toLowerCase().includes("cancel") ? "cancelled" : status.toLowerCase().includes("no") ? "noshow" : "booked",
      appointmentType: comp(sch, 8) ? { text: comp(sch, 8, 2) || comp(sch, 8) } : undefined,
      reasonCode: comp(sch, 7) ? [{ text: comp(sch, 7, 2) || comp(sch, 7) }] : undefined,
      start: hl7Date(comp(sch, 11, 4)),
      minutesDuration: Number(comp(sch, 9)) || undefined,
      participant: [
        { actor: { reference: `Patient/${patientId}` }, status: "accepted" },
        aip ? { actor: { display: [comp(aip, 3, 3), comp(aip, 3, 2)].filter(Boolean).join(" ") || comp(aip, 3) }, status: "accepted" } : null,
        ail ? { actor: { display: comp(ail, 3, 1) || comp(ail, 3) }, status: "accepted" } : null,
      ].filter(Boolean),
    });
  }

  const obr = seg(msg, "OBR");
  if (obr) {
    const obxs = segs(msg, "OBX");
    const obsIds: string[] = [];
    obxs.forEach((obx, i) => {
      const id = `obs-${msg.controlId}-${i + 1}`;
      obsIds.push(id);
      const valueType = comp(obx, 2);
      const value = comp(obx, 5);
      entries.push({
        resourceType: "Observation",
        id,
        status: comp(obx, 11) === "F" ? "final" : "preliminary",
        code: { coding: [{ code: comp(obx, 3, 1), display: comp(obx, 3, 2), system: comp(obx, 3, 3) === "LN" ? "http://loinc.org" : undefined }] },
        subject: { reference: `Patient/${patientId}` },
        ...(valueType === "NM" ? { valueQuantity: { value: Number(value), unit: comp(obx, 6) } } : { valueString: value }),
        interpretation: comp(obx, 8) ? [{ coding: [{ code: comp(obx, 8) }] }] : undefined,
      });
    });
    entries.push({
      resourceType: "DiagnosticReport",
      id: `dr-${comp(obr, 3) || msg.controlId}`,
      status: comp(obr, 25) === "F" ? "final" : "preliminary",
      code: { coding: [{ code: comp(obr, 4, 1), display: comp(obr, 4, 2) }] },
      subject: { reference: `Patient/${patientId}` },
      effectiveDateTime: hl7Date(comp(obr, 7)),
      result: obsIds.map((id) => ({ reference: `Observation/${id}` })),
    });
  }

  for (const dg1 of segs(msg, "DG1")) {
    entries.push({
      resourceType: "Condition",
      id: `cond-${comp(dg1, 3, 1).replace(/\W/g, "")}`,
      code: { coding: [{ system: "http://hl7.org/fhir/sid/icd-10-cm", code: comp(dg1, 3, 1), display: comp(dg1, 3, 2) }] },
      subject: { reference: `Patient/${patientId}` },
    });
  }

  const clean = (o: unknown): unknown => {
    if (Array.isArray(o)) return o.map(clean).filter((v) => v !== undefined);
    if (o && typeof o === "object") {
      const out: Json = {};
      for (const [k, v] of Object.entries(o as Json)) {
        const c = clean(v);
        if (c === undefined || c === "" || (Array.isArray(c) && c.length === 0)) continue;
        if (c && typeof c === "object" && !Array.isArray(c) && Object.keys(c as Json).length === 0) continue;
        out[k] = c;
      }
      return out;
    }
    return o;
  };

  return clean({
    resourceType: "Bundle",
    type: "transaction",
    meta: { source: `${msg.sendingApp}@${msg.sendingFacility}`, tag: [{ code: msg.type }] },
    entry: entries.map((r) => ({
      fullUrl: `urn:uuid:${r.id}`,
      resource: r,
      request: { method: "PUT", url: `${r.resourceType}/${r.id}` },
    })),
  }) as Json;
}

export function buildAck(msg: Hl7Message, ok = true, text = "") {
  const now = new Date();
  const ts = `${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, "0")}${String(now.getDate()).padStart(2, "0")}${String(now.getHours()).padStart(2, "0")}${String(now.getMinutes()).padStart(2, "0")}${String(now.getSeconds()).padStart(2, "0")}`;
  const trigger = msg.type.split("^")[1] ?? "";
  return [
    `MSH|^~\\&|MOS_INTEGRATION|MATEO_ORTHO|${msg.sendingApp}|${msg.sendingFacility}|${ts}||ACK^${trigger}^ACK|ACK${msg.controlId}|P|${msg.version || "2.5.1"}`,
    `MSA|${ok ? "AA" : "AE"}|${msg.controlId}${text ? `|${text}` : ""}`,
  ].join("\r");
}

export const SAMPLE_MESSAGES: { id: string; label: string; description: string; text: string }[] = [
  {
    id: "adt",
    label: "ADT^A04 — Register patient",
    description: "Front desk registers a new patient in the EMR; insurance and visit data travel together.",
    text: [
      "MSH|^~\\&|ECW|MOS_WESTBURY|MOS_INTEGRATION|MATEO_ORTHO|20261116081502||ADT^A04^ADT_A01|MSG00041871|P|2.5.1",
      "EVN|A04|20261116081500",
      "PID|1||MOS-204871^^^MOS^MR||Gonzalez^Maria^L||19680412|F|||142 Fulton Ave^^Hempstead^NY^11550||(516)555-0191||spa^Spanish|||ACCT-88120",
      "PV1|1|O|WB^^^Westbury||||1999999901^Mateo^Adrian^^^DO|1888888801^Patel^Rina^^^MD|ORT||||||||||V-3381102|||||||||||||||||||||||||20261116081500",
      "IN1|1|FIDELIS-MCD|FC01|Fidelis Care|||||||||||||||||||||||||||||||||FC88310442",
      "DG1|1|I10|M54.16^Radiculopathy, lumbar region^I10||20261116|A",
    ].join("\n"),
  },
  {
    id: "siu",
    label: "SIU^S12 — New appointment",
    description: "Scheduling system books a spine consult; the platform turns it into a FHIR Appointment and triggers reminders.",
    text: [
      "MSH|^~\\&|ATHENA_SCHED|MOS_LINDENHURST|MOS_INTEGRATION|MATEO_ORTHO|20261116093011||SIU^S12|SCH77120|P|2.5.1",
      "SCH|A-55120|A-55120|||||SPINECONS^Spine surgery consultation|NEW^New patient|40|min|^^40^20261124101500^20261124105500||||||||||||||Booked",
      "PID|1||MOS-187205^^^MOS^MR||Chen^Linda||19520118|F|||88 Main St^^Flushing^NY^11354||(718)555-0164",
      "AIP|1||1999999901^Mateo^Adrian^^^DO|SURGEON",
      "AIL|1||LH^Lindenhurst Office",
    ].join("\n"),
  },
  {
    id: "oru",
    label: "ORU^R01 — Imaging result",
    description: "Outside imaging center sends an MRI report; findings are attached to the chart and to the pending prior authorization.",
    text: [
      "MSH|^~\\&|RIS|NASSAU_IMAGING|MOS_INTEGRATION|MATEO_ORTHO|20261115171244||ORU^R01|RAD99012|P|2.5.1",
      "PID|1||MOS-198344^^^MOS^MR||O'Connor^James||19750930|M",
      "OBR|1|ORD-7781|RAD-552019|72148^MRI lumbar spine w/o contrast^CPT|||20261115153000|||||||||1999999901^Mateo^Adrian||||||20261115171000|||F",
      "OBX|1|TX|IMPRESSION^Impression||Right paracentral disc extrusion at L4-L5 contacting the descending right L5 nerve root.||||||F",
      "OBX|2|TX|FINDINGS^Findings||Moderate right lateral recess stenosis at L4-L5. No fracture. Conus terminates normally.||||||F",
      "OBX|3|NM|72514-3^Pain severity 0-10^LN||7|{score}|||||F",
    ].join("\n"),
  },
];
