// Read-only FHIR R4 facade over the synthetic dataset.
// GET /api/fhir/metadata
// GET /api/fhir/Patient?name=&identifier=&birthdate=   |  /api/fhir/Patient/{id}
// GET /api/fhir/Coverage?patient=                      |  /api/fhir/Appointment?patient=&date=
// GET /api/fhir/ServiceRequest?patient=                |  /api/fhir/Practitioner
import { PATIENTS } from "@/lib/data/patients";
import { audit } from "@/lib/audit";
import { bundle, capabilityStatement, demoAppointments, demoServiceRequests, operationOutcome, practitioners, toFhirCoverage, toFhirPatient } from "@/lib/fhir";

const HEADERS = { "Content-Type": "application/fhir+json; charset=utf-8", "Cache-Control": "no-store", "X-Demo-Data": "synthetic" };
const json = (body: unknown, status = 200) => new Response(JSON.stringify(body, null, 2), { status, headers: HEADERS });

type Res = Record<string, unknown>;
const refOf = (r: Res, path: string[]): string | undefined => {
  let cur: unknown = r;
  for (const p of path) cur = (cur as Record<string, unknown> | undefined)?.[p];
  return typeof cur === "string" ? cur : undefined;
};

export async function GET(req: Request, ctx: { params: Promise<{ path: string[] }> }) {
  const { path } = await ctx.params;
  const url = new URL(req.url);
  const q = (k: string) => url.searchParams.get(k)?.toLowerCase().trim();
  const [type, id] = path;
  audit({ route: "/api/fhir", action: id ? "read" : "search", resource: type, outcome: "allowed" });

  if (type === "metadata") return json(capabilityStatement());

  switch (type) {
    case "Patient": {
      let list = PATIENTS;
      if (id) {
        const p = list.find((x) => x.id === id);
        return p ? json(toFhirPatient(p)) : json(operationOutcome(`Patient/${id} not found`), 404);
      }
      const name = q("name");
      const ident = q("identifier");
      const dob = q("birthdate");
      if (name) list = list.filter((p) => `${p.given} ${p.family}`.toLowerCase().includes(name));
      if (ident) list = list.filter((p) => p.mrn.toLowerCase() === ident || p.memberId.toLowerCase() === ident);
      if (dob) list = list.filter((p) => p.birthDate === dob);
      return json(bundle(list.map(toFhirPatient)));
    }
    case "Coverage": {
      const pat = q("patient")?.replace(/^patient\//, "");
      const list = PATIENTS.filter((p) => !pat || p.id === pat).map(toFhirCoverage);
      if (id) {
        const c = list.find((x) => x.id === id);
        return c ? json(c) : json(operationOutcome(`Coverage/${id} not found`), 404);
      }
      return json(bundle(list));
    }
    case "Appointment": {
      const date = url.searchParams.get("date") ?? "2026-11-17";
      let list: Res[] = demoAppointments(date.replace(/^(eq|ge|le)/, "").slice(0, 10));
      const pat = q("patient")?.replace(/^patient\//, "");
      if (pat) list = list.filter((a) => (a.participant as Res[]).some((p) => refOf(p, ["actor", "reference"])?.toLowerCase() === `patient/${pat}`));
      if (id) {
        const a = list.find((x) => x.id === id);
        return a ? json(a) : json(operationOutcome(`Appointment/${id} not found`), 404);
      }
      return json(bundle(list));
    }
    case "ServiceRequest": {
      let list: Res[] = demoServiceRequests();
      const pat = q("patient")?.replace(/^patient\//, "");
      if (pat) list = list.filter((s) => refOf(s, ["subject", "reference"])?.toLowerCase() === `patient/${pat}`);
      return json(bundle(list));
    }
    case "Practitioner":
      return json(bundle(practitioners()));
    default:
      return json(operationOutcome(`Resource type '${type}' is not supported by this server. See /api/fhir/metadata.`, "not-supported"), 404);
  }
}
