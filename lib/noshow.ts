// Scores appointments with the logistic-regression weights trained by ml/no_show_model.py.
import weights from "./models/no-show-weights.json";

export interface NoShowFeatures {
  lead_time_days: number;
  prior_no_shows: number;
  new_patient: boolean;
  reminder_confirmed: boolean;
  medicaid: boolean;
  no_fault_or_wc: boolean;
  distance_miles: number;
  monday_or_friday: boolean;
  early_morning: boolean;
}

export const FEATURE_LABELS: Record<keyof NoShowFeatures, string> = {
  lead_time_days: "Booking lead time",
  prior_no_shows: "Prior no-shows (12 mo)",
  new_patient: "New patient",
  reminder_confirmed: "Confirmed reminder",
  medicaid: "Medicaid coverage",
  no_fault_or_wc: "No-fault / workers' comp",
  distance_miles: "Travel distance",
  monday_or_friday: "Monday / Friday slot",
  early_morning: "Before 9 AM",
};

export const MODEL = weights as {
  model: string;
  version: string;
  trained_on: string;
  base_no_show_rate: number;
  test_auc: number;
  bias: number;
  features: (keyof NoShowFeatures)[];
  weights: Record<keyof NoShowFeatures, number>;
  scaling: { lead_time_days: number; distance_miles: number };
};

function vector(f: NoShowFeatures): Record<keyof NoShowFeatures, number> {
  return {
    lead_time_days: f.lead_time_days / MODEL.scaling.lead_time_days,
    prior_no_shows: f.prior_no_shows,
    new_patient: f.new_patient ? 1 : 0,
    reminder_confirmed: f.reminder_confirmed ? 1 : 0,
    medicaid: f.medicaid ? 1 : 0,
    no_fault_or_wc: f.no_fault_or_wc ? 1 : 0,
    distance_miles: f.distance_miles / MODEL.scaling.distance_miles,
    monday_or_friday: f.monday_or_friday ? 1 : 0,
    early_morning: f.early_morning ? 1 : 0,
  };
}

export function scoreNoShow(f: NoShowFeatures) {
  const x = vector(f);
  const contributions = MODEL.features.map((k) => ({ feature: k, label: FEATURE_LABELS[k], value: MODEL.weights[k] * x[k] }));
  const z = MODEL.bias + contributions.reduce((s, c) => s + c.value, 0);
  const p = 1 / (1 + Math.exp(-z));
  return { probability: p, contributions: contributions.sort((a, b) => Math.abs(b.value) - Math.abs(a.value)) };
}

export function riskBand(p: number): { label: string; tone: "good" | "warn" | "danger" } {
  if (p >= 0.3) return { label: "High", tone: "danger" };
  if (p >= 0.15) return { label: "Medium", tone: "warn" };
  return { label: "Low", tone: "good" };
}

export function intervention(p: number, f: NoShowFeatures) {
  if (p >= 0.3) {
    if (f.medicaid || f.distance_miles > 15) return "Call personally, offer Medicaid transportation (MAS) or a closer location, and backfill from the waitlist if unconfirmed by 24 h.";
    return "Personal call from front desk + double-confirm by SMS; allow a waitlist overbook in this slot.";
  }
  if (p >= 0.15) return f.reminder_confirmed ? "Standard 48 h + 2 h SMS reminders." : "Two-way SMS with one-tap confirm / reschedule at 72 h and 24 h.";
  return "Standard reminder cadence.";
}
