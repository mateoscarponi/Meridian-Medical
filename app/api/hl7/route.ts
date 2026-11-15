// HL7 v2 inbound endpoint: POST raw HL7 (text/plain) → { ack, fhir, parsed summary, timing }.
// Production equivalent: MLLP listener (interface engine such as Mirth/Rhapsody or a small Go service)
// → queue → this transformer → FHIR server / warehouse.
import { buildAck, hl7ToFhir, parseHl7 } from "@/lib/hl7";
import { audit } from "@/lib/audit";

export async function POST(req: Request) {
  const t0 = performance.now();
  const text = (await req.text()).slice(0, 100_000);
  const msg = parseHl7(text);
  if (msg.errors.length && !msg.segments.length) {
    return Response.json({ ok: false, errors: msg.errors }, { status: 400 });
  }
  const fhir = hl7ToFhir(msg);
  const ok = msg.errors.length === 0;
  const ack = buildAck(msg, ok, ok ? "" : msg.errors[0]);
  audit({ route: "/api/hl7", action: "ingest", resource: `${msg.type} ${msg.controlId}`, outcome: ok ? "allowed" : "error" });
  return Response.json({
    ok,
    messageType: msg.type,
    controlId: msg.controlId,
    segments: msg.segments.map((s) => s.id),
    warnings: msg.errors,
    ack,
    fhir,
    processingMs: Math.round((performance.now() - t0) * 100) / 100,
  });
}
