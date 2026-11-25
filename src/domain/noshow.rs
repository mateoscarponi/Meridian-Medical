//! Scores appointments with the logistic-regression weights trained by ml/no_show_model.py.
//! The weights JSON is embedded at compile time and parsed once.

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NoShowFeatures {
    pub lead_time_days: f64,
    pub prior_no_shows: f64,
    pub new_patient: bool,
    pub reminder_confirmed: bool,
    pub medicaid: bool,
    pub no_fault_or_wc: bool,
    pub distance_miles: f64,
    pub monday_or_friday: bool,
    pub early_morning: bool,
}

pub fn feature_label(k: &str) -> &'static str {
    match k {
        "lead_time_days" => "Booking lead time",
        "prior_no_shows" => "Prior no-shows (12 mo)",
        "new_patient" => "New patient",
        "reminder_confirmed" => "Confirmed reminder",
        "medicaid" => "Medicaid coverage",
        "no_fault_or_wc" => "No-fault / workers' comp",
        "distance_miles" => "Travel distance",
        "monday_or_friday" => "Monday / Friday slot",
        "early_morning" => "Before 9 AM",
        _ => "",
    }
}

#[derive(Deserialize, Debug)]
pub struct Scaling {
    pub lead_time_days: f64,
    pub distance_miles: f64,
}

#[derive(Deserialize, Debug)]
pub struct Model {
    pub model: String,
    pub version: String,
    pub trained_on: String,
    pub base_no_show_rate: f64,
    pub test_auc: f64,
    pub bias: f64,
    pub features: Vec<String>,
    pub weights: HashMap<String, f64>,
    pub scaling: Scaling,
}

pub static MODEL: LazyLock<Model> =
    LazyLock::new(|| serde_json::from_str(include_str!("../../data/no-show-weights.json")).expect("valid model JSON"));

fn value(f: &NoShowFeatures, k: &str) -> f64 {
    let b = |x: bool| if x { 1.0 } else { 0.0 };
    match k {
        "lead_time_days" => f.lead_time_days / MODEL.scaling.lead_time_days,
        "prior_no_shows" => f.prior_no_shows,
        "new_patient" => b(f.new_patient),
        "reminder_confirmed" => b(f.reminder_confirmed),
        "medicaid" => b(f.medicaid),
        "no_fault_or_wc" => b(f.no_fault_or_wc),
        "distance_miles" => f.distance_miles / MODEL.scaling.distance_miles,
        "monday_or_friday" => b(f.monday_or_friday),
        "early_morning" => b(f.early_morning),
        _ => 0.0,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Contribution {
    pub feature: String,
    pub label: &'static str,
    pub value: f64,
}

pub struct Score {
    pub probability: f64,
    pub contributions: Vec<Contribution>,
}

pub fn score_no_show(f: &NoShowFeatures) -> Score {
    let mut contributions: Vec<Contribution> = MODEL
        .features
        .iter()
        .map(|k| Contribution { feature: k.clone(), label: feature_label(k), value: MODEL.weights.get(k).copied().unwrap_or(0.0) * value(f, k) })
        .collect();
    let z = MODEL.bias + contributions.iter().map(|c| c.value).sum::<f64>();
    let p = 1.0 / (1.0 + (-z).exp());
    // Stable sort by |value| descending, same as the original Array.prototype.sort.
    contributions.sort_by(|a, b| b.value.abs().partial_cmp(&a.value.abs()).unwrap_or(std::cmp::Ordering::Equal));
    Score { probability: p, contributions }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RiskTone {
    Good,
    Warn,
    Danger,
}

pub fn risk_band(p: f64) -> (&'static str, RiskTone) {
    if p >= 0.3 {
        ("High", RiskTone::Danger)
    } else if p >= 0.15 {
        ("Medium", RiskTone::Warn)
    } else {
        ("Low", RiskTone::Good)
    }
}

pub fn intervention(p: f64, f: &NoShowFeatures) -> &'static str {
    if p >= 0.3 {
        if f.medicaid || f.distance_miles > 15.0 {
            return "Call personally, offer Medicaid transportation (MAS) or a closer location, and backfill from the waitlist if unconfirmed by 24 h.";
        }
        return "Personal call from front desk + double-confirm by SMS; allow a waitlist overbook in this slot.";
    }
    if p >= 0.15 {
        return if f.reminder_confirmed { "Standard 48 h + 2 h SMS reminders." } else { "Two-way SMS with one-tap confirm / reschedule at 72 h and 24 h." };
    }
    "Standard reminder cadence."
}
