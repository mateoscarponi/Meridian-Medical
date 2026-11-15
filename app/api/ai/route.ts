// LLM endpoint for the Clinical Scribe and Patient Messages tools.
// - If ANTHROPIC_API_KEY is configured, generates with Claude (server-side only; key never reaches the browser).
// - Otherwise returns { engine: "local" } and the client uses its deterministic on-device generator,
//   so the demo always works. In production this runs only under a BAA, on de-identified or
//   minimum-necessary text, with zero data retention configured.
import Anthropic from "@anthropic-ai/sdk";
import { findPhi, redact } from "@/lib/phi";
import { audit } from "@/lib/audit";

export const runtime = "nodejs";

const SYSTEM: Record<string, string> = {
  scribe: `You are a clinical documentation assistant for an orthopedic spine practice. Convert the visit transcript into a concise SOAP note for provider review.
Sections, each as a plain-text heading on its own line: SUBJECTIVE, OBJECTIVE, ASSESSMENT, PLAN.
In ASSESSMENT list likely ICD-10-CM codes with the code first (e.g. "M51.16 – Intervertebral disc disorder with radiculopathy, lumbar region").
In PLAN, list orders and note which ones typically need prior authorization.
Only document what the transcript supports; write "not discussed" otherwise. No markdown symbols. Under 350 words.`,
  reply: `You draft replies from an orthopedic & spine practice front desk to patient messages, for staff to approve before sending.
Be warm, brief (under 90 words) and specific. Never give medical advice or diagnoses. If the message describes possible emergency symptoms (loss of bladder/bowel control, saddle numbness, rapidly worsening weakness, fever after surgery), tell the patient to call 911 or go to the nearest emergency room now and that the on-call provider is being notified.
Reply in the same language the patient wrote in. Sign as "Meridian Orthopedics & Spine Care Team". Plain text only.`,
};

export async function POST(req: Request) {
  let body: { task?: string; input?: string };
  try {
    body = await req.json();
  } catch {
    return Response.json({ error: "Invalid JSON body" }, { status: 400 });
  }
  const task = body.task ?? "";
  const input = (body.input ?? "").slice(0, 8000);
  if (!SYSTEM[task] || !input.trim()) return Response.json({ error: "task must be 'scribe' or 'reply' and input is required" }, { status: 400 });

  if (!process.env.ANTHROPIC_API_KEY) {
    audit({ route: "/api/ai", action: task, outcome: "allowed", detail: "local engine" });
    return Response.json({ engine: "local" });
  }

  // Minimum necessary: strip direct identifiers before the text leaves our infrastructure,
  // even though the LLM vendor is covered by a BAA with zero data retention.
  const hits = findPhi(input);
  const safeInput = redact(input, hits);
  audit({ route: "/api/ai", action: task, outcome: "allowed", detail: `llm call, ${hits.length} identifiers redacted` });

  const client = new Anthropic();
  try {
    const response = await client.beta.messages.create({
      model: "claude-opus-5-5",
      max_tokens: 4000,
      betas: ["server-side-fallback-2026-07-01"],
      fallbacks: "default",
      output_config: { effort: "low" },
      system: SYSTEM[task],
      messages: [{ role: "user", content: safeInput }],
    });
    if (response.stop_reason === "refusal") return Response.json({ engine: "local", note: "Model declined; used on-device generator." });
    const text = response.content
      .filter((b): b is Extract<typeof b, { type: "text" }> => b.type === "text")
      .map((b) => b.text)
      .join("\n")
      .trim();
    if (!text) return Response.json({ engine: "local", note: "Empty response; used on-device generator." });
    return Response.json({ engine: "claude", model: response.model, text, phiRedacted: hits.length });
  } catch (error) {
    let note = "LLM unavailable; used on-device generator.";
    if (error instanceof Anthropic.AuthenticationError) note = "Invalid API key; used on-device generator.";
    else if (error instanceof Anthropic.RateLimitError) note = "Rate limited; used on-device generator.";
    else if (error instanceof Anthropic.APIError) note = `LLM error ${error.status}; used on-device generator.`;
    audit({ route: "/api/ai", action: task, outcome: "error", detail: note });
    return Response.json({ engine: "local", note });
  }
}
