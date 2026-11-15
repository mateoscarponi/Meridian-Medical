"""
No-show risk model — training script (standard library only).

Trains a logistic-regression classifier on a SYNTHETIC appointment dataset and
exports the learned coefficients to `lib/models/no-show-weights.json`, which the
Next.js scheduling module loads to score appointments in real time.

In production this would train on de-identified appointment history exported
from the EMR / practice-management system (e.g. eClinicalWorks or athenahealth
scheduling tables) through the central data pipeline, and be re-trained monthly.

Run:  python ml/no_show_model.py
"""

from __future__ import annotations

import json
import math
import random
from pathlib import Path

SEED = 7
N_TRAIN = 12_000
N_TEST = 3_000

FEATURES = [
    "lead_time_days",      # days between booking and visit (scaled /30)
    "prior_no_shows",      # count in last 12 months
    "new_patient",         # 1 = first visit
    "reminder_confirmed",  # 1 = patient confirmed via SMS/IVR
    "medicaid",            # 1 = Medicaid (proxy for transport/access barriers)
    "no_fault_or_wc",      # 1 = no-fault / workers' comp case
    "distance_miles",      # scaled /10
    "monday_or_friday",    # 1 = Mon/Fri slot
    "early_morning",       # 1 = slot before 9am
]

# "True" generating process used to simulate data (unknown to the learner).
TRUE_BIAS = -2.15
TRUE_W = [0.55, 0.62, 0.48, -1.35, 0.58, -0.22, 0.30, 0.18, 0.21]


def sigmoid(z: float) -> float:
    if z < -35:
        return 0.0
    if z > 35:
        return 1.0
    return 1.0 / (1.0 + math.exp(-z))


def synth_row(rng: random.Random) -> tuple[list[float], int]:
    lead = min(rng.expovariate(1 / 14), 90) / 30
    prior = min(int(rng.expovariate(1 / 0.6)), 6)
    new = 1 if rng.random() < 0.28 else 0
    confirmed = 1 if rng.random() < 0.62 else 0
    payer = rng.random()
    medicaid = 1 if payer < 0.12 else 0
    nf_wc = 1 if 0.12 <= payer < 0.37 else 0
    distance = min(rng.lognormvariate(1.6, 0.6), 45) / 10
    mon_fri = 1 if rng.random() < 0.4 else 0
    early = 1 if rng.random() < 0.18 else 0
    x = [lead, prior, new, confirmed, medicaid, nf_wc, distance, mon_fri, early]
    p = sigmoid(TRUE_BIAS + sum(w * v for w, v in zip(TRUE_W, x)))
    return x, 1 if rng.random() < p else 0


def train(X, y, lr=0.9, epochs=700, l2=1e-4):
    w = [0.0] * len(X[0])
    b = 0.0
    n = len(X)
    for _ in range(epochs):
        gw = [0.0] * len(w)
        gb = 0.0
        for xi, yi in zip(X, y):
            err = sigmoid(b + sum(a * c for a, c in zip(w, xi))) - yi
            gb += err
            for j, v in enumerate(xi):
                gw[j] += err * v
        b -= lr * gb / n
        w = [wj - lr * (gj / n + l2 * wj) for wj, gj in zip(w, gw)]
    return b, w


def auc(scores, labels):
    pairs = sorted(zip(scores, labels))
    pos = sum(labels)
    neg = len(labels) - pos
    rank_sum = 0.0
    for i, (_, lab) in enumerate(pairs, start=1):
        if lab == 1:
            rank_sum += i
    return (rank_sum - pos * (pos + 1) / 2) / (pos * neg)


def main() -> None:
    rng = random.Random(SEED)
    train_rows = [synth_row(rng) for _ in range(N_TRAIN)]
    test_rows = [synth_row(rng) for _ in range(N_TEST)]
    Xtr, ytr = zip(*train_rows)
    Xte, yte = zip(*test_rows)

    b, w = train(list(Xtr), list(ytr))
    scores = [sigmoid(b + sum(a * c for a, c in zip(w, x))) for x in Xte]
    test_auc = auc(scores, list(yte))
    base_rate = sum(ytr) / len(ytr)

    out = {
        "model": "logistic_regression",
        "version": "2026.11-synthetic",
        "trained_on": f"{N_TRAIN} synthetic appointments (seed {SEED})",
        "base_no_show_rate": round(base_rate, 4),
        "test_auc": round(test_auc, 4),
        "bias": round(b, 4),
        "features": FEATURES,
        "weights": {f: round(v, 4) for f, v in zip(FEATURES, w)},
        "scaling": {"lead_time_days": 30, "distance_miles": 10},
    }
    dest = Path(__file__).resolve().parent.parent / "lib" / "models" / "no-show-weights.json"
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(json.dumps(out, indent=2))
    print(json.dumps(out, indent=2))


if __name__ == "__main__":
    main()
