"""Compares target/parity-ts.json (original TypeScript) with target/parity-rust.json (Rust port)."""
import json, sys
a = json.load(open("target/parity-ts.json", encoding="utf8"))
b = json.load(open("target/parity-rust.json", encoding="utf8"))
def norm(x):
    if isinstance(x, float) and x.is_integer(): return int(x)
    if isinstance(x, dict): return {k: norm(v) for k, v in x.items()}
    if isinstance(x, list): return [norm(v) for v in x]
    return x
a, b = norm(a), norm(b)
bad = 0
for k in a:
    if a[k] != b.get(k):
        bad += 1
        print("DIFF", k)
        if isinstance(a[k], dict) and isinstance(b.get(k), dict):
            for kk in a[k]:
                if a[k][kk] != b[k].get(kk):
                    print("  field", kk, "\n   ts:", json.dumps(a[k][kk], ensure_ascii=False)[:800], "\n   rs:", json.dumps(b[k].get(kk), ensure_ascii=False)[:800])
        else:
            print("   ts:", json.dumps(a[k], ensure_ascii=False)[:1500], "\n   rs:", json.dumps(b.get(k), ensure_ascii=False)[:1500])
print(f"cases compared: {len(a)} · mismatches: {bad}")
sys.exit(1 if bad else 0)
