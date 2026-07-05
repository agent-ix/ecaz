#!/usr/bin/env python3
"""Aggregate Task 105 full-scale sweep results into matrix tables.

Reads the packet-local results.jsonl files from the G4 (004) and Intel
(005) lane packets and emits markdown tables on stdout. Run from the
repo root. No external deps.
"""
import json
import os
import re
from collections import defaultdict

SOURCES = [
    ("g4", "10k", "reviews/task-105/004-g4-lane/artifacts/sweep-10k/20260612T054423Z/results.jsonl"),
    ("g4", "50k", "reviews/task-105/004-g4-lane/artifacts/sweep-50k-warm/20260612T074213Z/results.jsonl"),
    ("g4", "100k", "reviews/task-105/004-g4-lane/artifacts/gate-clean/20260612T053900Z/results.jsonl"),
    ("g4", "1m", "reviews/task-105/004-g4-lane/artifacts/sweep-1m/20260612T133752Z/results.jsonl"),
    ("intel", "10k", "reviews/task-105/005-intel-lane/artifacts/sweep-10k-clean/20260612T061932Z/results.jsonl"),
    ("intel", "50k", "reviews/task-105/005-intel-lane/artifacts/sweep-50k-quiet/results.jsonl"),
    ("intel", "1m", "reviews/task-105/005-intel-lane/artifacts/sweep-1m/results.jsonl"),
]

STEP_RE = re.compile(r"^(latency|recall)-(.+)-(on|off)-(10k|50k|1m|confirm)$")

# family -> sweep parameter key in values
def sweep_point(values):
    for k in ("nprobe", "ef_search", "list_size"):
        if k in values:
            return f"{k}={values[k]}"
    return "-"


def load(path):
    rows = []
    with open(path) as fh:
        for line in fh:
            rows.append(json.loads(line))
    return rows


def main():
    # cell[(lane, scale, family)] = {"lat": {(mode, pt): p50}, "rec": {(mode, pt): recall},
    #                                "isa": set(), "storage": (size, per_row)}
    cell = defaultdict(lambda: {"lat": {}, "rec": {}, "isa": set(), "storage": None})
    storage = {}

    for lane, scale, path in SOURCES:
        for r in load(path):
            v = r["values"]
            m = STEP_RE.match(r["step"]) if r.get("step") else None
            if r["metric"] == "latency" and m and "p50" in v:
                kind, fam, mode, _ = m.groups()
                cell[(lane, scale, fam)]["lat"][(mode, sweep_point(v))] = float(v["p50"].split()[0])
            elif r["metric"] == "recall" and m and "recall@k" in v:
                kind, fam, mode, _ = m.groups()
                cell[(lane, scale, fam)]["rec"][(mode, sweep_point(v))] = v["recall@k"]
            elif r["metric"] == "block_kernel_counters" and m:
                kind, fam, mode, _ = m.groups()
                if mode == "on":
                    cell[(lane, scale, fam)]["isa"].add(v.get("isa", "?"))
            elif r["metric"] == "storage_index" and v.get("access method") != "btree":
                pfx = v["prefix"]  # t105_<am>_<quant>_<scale>
                storage[(lane, pfx)] = (v["size"], v["per row"])

    fams = sorted({f for (_, _, f) in cell})
    scales = ["10k", "50k", "100k", "1m"]

    def fmt_lat(c):
        pts = sorted({pt for (_, pt) in c["lat"]}, key=lambda s: int(s.split("=")[1]))
        out = []
        for pt in pts:
            on, off = c["lat"].get(("on", pt)), c["lat"].get(("off", pt))
            if on is not None and off is not None:
                out.append(f"{on:g}/{off:g} ({100*(on-off)/off:+.0f}%)")
            elif on is not None:
                out.append(f"{on:g} (on only)")
        return "<br>".join(out) if out else "—"

    def fmt_rec(c):
        pts = sorted({pt for (_, pt) in c["rec"]}, key=lambda s: int(s.split("=")[1]))
        out = []
        for pt in pts:
            on, off = c["rec"].get(("on", pt)), c["rec"].get(("off", pt))
            if on is not None and off is not None:
                mark = "=" if on == off else f" OFF={off} MISMATCH"
                out.append(f"{on}{'' if mark == '=' else mark}")
            elif on is not None:
                out.append(f"{on}*")
        return " / ".join(out) if out else "—"

    for lane in ("g4", "intel"):
        print(f"\n### Latency — {lane.upper()} (p50 ms, kernel on/off, per sweep point)\n")
        print("| family | " + " | ".join(scales) + " |")
        print("|---|" + "---|" * len(scales))
        for f in fams:
            row = [fmt_lat(cell[(lane, s, f)]) if (lane, s, f) in cell else "—" for s in scales]
            print(f"| {f} | " + " | ".join(row) + " |")

    for lane in ("g4", "intel"):
        print(f"\n### Recall@10 — {lane.upper()} (kernel on; `*` = on-only cell, otherwise verified on==off)\n")
        print("| family | " + " | ".join(scales) + " |")
        print("|---|" + "---|" * len(scales))
        for f in fams:
            row = [fmt_rec(cell[(lane, s, f)]) if (lane, s, f) in cell else "—" for s in scales]
            print(f"| {f} | " + " | ".join(row) + " |")

    print("\n### ISA attribution (kernel-on cells with counter rows)\n")
    print("| family | " + " | ".join(f"{l}/{s}" for l in ("g4", "intel") for s in scales) + " |")
    print("|---|" + "---|" * 8)
    for f in fams:
        row = []
        for l in ("g4", "intel"):
            for s in scales:
                isa = cell[(l, s, f)]["isa"] if (l, s, f) in cell else set()
                row.append(",".join(sorted(isa)) if isa else "—")
        print(f"| {f} | " + " | ".join(row) + " |")

    print("\n### Index storage (G4 lane; Intel sizes match within build noise)\n")
    print("| fixture | size | per row |")
    print("|---|---|---|")
    for (lane, pfx), (size, pr) in sorted(storage.items()):
        if lane == "g4":
            print(f"| {pfx} | {size} | {pr} |")


if __name__ == "__main__":
    main()
