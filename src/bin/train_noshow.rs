//! No-show risk model — training program (no dependencies beyond std + serde_json).
//!
//! Rust port of the original `ml/no_show_model.py`. Trains a logistic-regression classifier on a
//! SYNTHETIC appointment dataset and exports the learned coefficients to
//! `data/no-show-weights.json`, which the scheduling module embeds and scores live.
//!
//! To reproduce the Python results exactly, this implements CPython's `random.Random`
//! (MT19937 + init_by_array seeding, `random()`, `expovariate`, `normalvariate`,
//! `lognormvariate`). Same seed (7) → same synthetic data → same weights.
//!
//! In production this would train on de-identified appointment history exported from the
//! EMR / practice-management system through the central data pipeline, re-trained monthly.
//!
//! Run:  cargo run --release --bin train_noshow

use serde_json::json;
use std::time::Instant;

const SEED: u32 = 7;
const N_TRAIN: usize = 12_000;
const N_TEST: usize = 3_000;

const FEATURES: [&str; 9] = [
    "lead_time_days",     // days between booking and visit (scaled /30)
    "prior_no_shows",     // count in last 12 months
    "new_patient",        // 1 = first visit
    "reminder_confirmed", // 1 = patient confirmed via SMS/IVR
    "medicaid",           // 1 = Medicaid (proxy for transport/access barriers)
    "no_fault_or_wc",     // 1 = no-fault / workers' comp case
    "distance_miles",     // scaled /10
    "monday_or_friday",   // 1 = Mon/Fri slot
    "early_morning",      // 1 = slot before 9am
];

// "True" generating process used to simulate data (unknown to the learner).
const TRUE_BIAS: f64 = -2.15;
const TRUE_W: [f64; 9] = [0.55, 0.62, 0.48, -1.35, 0.58, -0.22, 0.30, 0.18, 0.21];

/// CPython's Mersenne Twister (`random.Random`).
struct PyRandom {
    mt: [u32; 624],
    index: usize,
}

impl PyRandom {
    fn new(seed: u32) -> Self {
        let mut r = PyRandom { mt: [0; 624], index: 624 };
        r.init_by_array(&[seed]);
        r
    }
    fn init_genrand(&mut self, s: u32) {
        self.mt[0] = s;
        for i in 1..624 {
            self.mt[i] = 1812433253u32.wrapping_mul(self.mt[i - 1] ^ (self.mt[i - 1] >> 30)).wrapping_add(i as u32);
        }
        self.index = 624;
    }
    fn init_by_array(&mut self, key: &[u32]) {
        self.init_genrand(19650218);
        let n = 624;
        let (mut i, mut j) = (1usize, 0usize);
        for _ in 0..n.max(key.len()) {
            self.mt[i] = (self.mt[i] ^ (self.mt[i - 1] ^ (self.mt[i - 1] >> 30)).wrapping_mul(1664525)).wrapping_add(key[j]).wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= n {
                self.mt[0] = self.mt[n - 1];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
        }
        for _ in 0..n - 1 {
            self.mt[i] = (self.mt[i] ^ (self.mt[i - 1] ^ (self.mt[i - 1] >> 30)).wrapping_mul(1566083941)).wrapping_sub(i as u32);
            i += 1;
            if i >= n {
                self.mt[0] = self.mt[n - 1];
                i = 1;
            }
        }
        self.mt[0] = 0x8000_0000;
    }
    fn next_u32(&mut self) -> u32 {
        if self.index >= 624 {
            for k in 0..624 {
                let y = (self.mt[k] & 0x8000_0000) | (self.mt[(k + 1) % 624] & 0x7fff_ffff);
                let mut v = self.mt[(k + 397) % 624] ^ (y >> 1);
                if y & 1 != 0 {
                    v ^= 0x9908_b0df;
                }
                self.mt[k] = v;
            }
            self.index = 0;
        }
        let mut y = self.mt[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^= y >> 18;
        y
    }
    /// `random()`: 53-bit float in [0, 1).
    fn random(&mut self) -> f64 {
        let a = (self.next_u32() >> 5) as f64;
        let b = (self.next_u32() >> 6) as f64;
        (a * 67108864.0 + b) * (1.0 / 9007199254740992.0)
    }
    fn expovariate(&mut self, lambd: f64) -> f64 {
        -(1.0 - self.random()).ln() / lambd
    }
    fn normalvariate(&mut self, mu: f64, sigma: f64) -> f64 {
        let nv_magicconst = 4.0 * (-0.5f64).exp() / 2.0f64.sqrt();
        loop {
            let u1 = self.random();
            let u2 = 1.0 - self.random();
            let z = nv_magicconst * (u1 - 0.5) / u2;
            let zz = z * z / 4.0;
            if zz <= -u2.ln() {
                return mu + z * sigma;
            }
        }
    }
    fn lognormvariate(&mut self, mu: f64, sigma: f64) -> f64 {
        self.normalvariate(mu, sigma).exp()
    }
}

fn sigmoid(z: f64) -> f64 {
    if z < -35.0 {
        return 0.0;
    }
    if z > 35.0 {
        return 1.0;
    }
    1.0 / (1.0 + (-z).exp())
}

fn dot(w: &[f64; 9], x: &[f64; 9]) -> f64 {
    w.iter().zip(x).fold(0.0, |s, (a, b)| s + a * b)
}

fn synth_row(rng: &mut PyRandom) -> ([f64; 9], f64) {
    let lead = rng.expovariate(1.0 / 14.0).min(90.0) / 30.0;
    let prior = (rng.expovariate(1.0 / 0.6) as i64).min(6) as f64;
    let new = if rng.random() < 0.28 { 1.0 } else { 0.0 };
    let confirmed = if rng.random() < 0.62 { 1.0 } else { 0.0 };
    let payer = rng.random();
    let medicaid = if payer < 0.12 { 1.0 } else { 0.0 };
    let nf_wc = if (0.12..0.37).contains(&payer) { 1.0 } else { 0.0 };
    let distance = rng.lognormvariate(1.6, 0.6).min(45.0) / 10.0;
    let mon_fri = if rng.random() < 0.4 { 1.0 } else { 0.0 };
    let early = if rng.random() < 0.18 { 1.0 } else { 0.0 };
    let x = [lead, prior, new, confirmed, medicaid, nf_wc, distance, mon_fri, early];
    let p = sigmoid(TRUE_BIAS + dot(&TRUE_W, &x));
    (x, if rng.random() < p { 1.0 } else { 0.0 })
}

fn train(x: &[[f64; 9]], y: &[f64], lr: f64, epochs: usize, l2: f64) -> (f64, [f64; 9]) {
    let mut w = [0.0f64; 9];
    let mut b = 0.0f64;
    let n = x.len() as f64;
    for _ in 0..epochs {
        let mut gw = [0.0f64; 9];
        let mut gb = 0.0f64;
        for (xi, yi) in x.iter().zip(y) {
            let err = sigmoid(b + dot(&w, xi)) - yi;
            gb += err;
            for (g, v) in gw.iter_mut().zip(xi) {
                *g += err * v;
            }
        }
        b -= lr * gb / n;
        for (wj, gj) in w.iter_mut().zip(gw) {
            *wj -= lr * (gj / n + l2 * *wj);
        }
    }
    (b, w)
}

fn auc(scores: &[f64], labels: &[f64]) -> f64 {
    let mut pairs: Vec<(f64, f64)> = scores.iter().copied().zip(labels.iter().copied()).collect();
    pairs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let pos: f64 = labels.iter().sum();
    let neg = labels.len() as f64 - pos;
    let rank_sum: f64 = pairs.iter().enumerate().filter(|(_, (_, l))| *l == 1.0).map(|(i, _)| (i + 1) as f64).sum();
    (rank_sum - pos * (pos + 1.0) / 2.0) / (pos * neg)
}

/// Python's `round(x, 4)` (correctly rounded).
fn round4(x: f64) -> f64 {
    format!("{x:.4}").parse().unwrap_or(x)
}

fn main() {
    let t0 = Instant::now();
    let mut rng = PyRandom::new(SEED);
    let train_rows: Vec<_> = (0..N_TRAIN).map(|_| synth_row(&mut rng)).collect();
    let test_rows: Vec<_> = (0..N_TEST).map(|_| synth_row(&mut rng)).collect();
    let (xtr, ytr): (Vec<[f64; 9]>, Vec<f64>) = train_rows.into_iter().unzip();
    let (xte, yte): (Vec<[f64; 9]>, Vec<f64>) = test_rows.into_iter().unzip();

    let (b, w) = train(&xtr, &ytr, 0.9, 700, 1e-4);
    let scores: Vec<f64> = xte.iter().map(|x| sigmoid(b + dot(&w, x))).collect();
    let test_auc = auc(&scores, &yte);
    let base_rate = ytr.iter().sum::<f64>() / ytr.len() as f64;

    let mut weights = serde_json::Map::new();
    for (f, v) in FEATURES.iter().zip(w) {
        weights.insert(f.to_string(), json!(round4(v)));
    }
    let out = json!({
        "model": "logistic_regression",
        "version": "2026.11-synthetic",
        "trained_on": format!("{N_TRAIN} synthetic appointments (seed {SEED})"),
        "base_no_show_rate": round4(base_rate),
        "test_auc": round4(test_auc),
        "bias": round4(b),
        "features": FEATURES,
        "weights": weights,
        "scaling": { "lead_time_days": 30, "distance_miles": 10 },
    });
    let text = serde_json::to_string_pretty(&out).expect("json");
    let dest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data").join("no-show-weights.json");
    std::fs::write(&dest, &text).expect("write weights");
    println!("{text}");
    eprintln!("trained in {:.2} s -> {}", t0.elapsed().as_secs_f64(), dest.display());
}

#[cfg(test)]
mod tests {
    use super::PyRandom;
    #[test]
    fn matches_cpython_random() {
        // python -c "import random; r=random.Random(7); print(r.random())"
        let mut r = PyRandom::new(7);
        assert_eq!(r.random(), 0.32383276483316237);
    }
}
