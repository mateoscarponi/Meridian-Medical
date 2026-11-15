// Structured audit events for server routes. Records who/what/when/outcome, never the payload,
// so logs themselves never become a store of PHI. In production these go to an append-only,
// access-controlled log sink (e.g. CloudWatch / Cloud Logging with retention + object lock).
export function audit(event: { route: string; action: string; resource?: string; outcome: "allowed" | "denied" | "error"; detail?: string }) {
  console.info(JSON.stringify({ type: "audit", ts: new Date().toISOString(), actor: "demo-user", ...event }));
}
