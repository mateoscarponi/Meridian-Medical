//! Structured audit events for server routes. Records who/what/when/outcome, never the payload,
//! so logs themselves never become a store of PHI. In production these go to an append-only,
//! access-controlled log sink (e.g. CloudWatch / Cloud Logging with retention + object lock).

use serde_json::json;

pub fn audit(route: &str, action: &str, resource: Option<&str>, outcome: &str, detail: Option<&str>) {
    let mut ev = json!({
        "type": "audit",
        "ts": crate::domain::util::now_iso(),
        "actor": "demo-user",
        "route": route,
        "action": action,
    });
    let m = ev.as_object_mut().expect("object");
    if let Some(r) = resource {
        m.insert("resource".into(), r.into());
    }
    m.insert("outcome".into(), outcome.into());
    if let Some(d) = detail {
        m.insert("detail".into(), d.into());
    }
    println!("{ev}");
}
