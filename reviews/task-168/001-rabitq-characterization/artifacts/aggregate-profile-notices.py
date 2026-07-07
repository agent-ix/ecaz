#!/usr/bin/env python3
"""Aggregate ec_diskann_scan_profile NOTICE lines from packet logs.

Usage: python3 aggregate-profile-notices.py profile-notices-*.log
Prints per-file mean/p50/p95 for every profile key plus the flush-width
histogram totals and the frontier share of total_us.
"""
import re
import sys
import statistics

KEYS = [
    "setup_us", "entry_resolution_us", "graph_read_decode_us",
    "prefilter_score_us", "frontier_us", "frontier_candidate_heap_us",
    "frontier_visited_set_us", "frontier_neighbor_iter_us",
    "frontier_retained_insert_us", "heap_prefetch_us", "exact_rerank_us",
    "result_expand_us", "total_us", "graph_read_count", "prefilter_count",
    "frontier_candidate_heap_ops", "frontier_visited_set_ops",
    "frontier_neighbor_slots", "frontier_retained_inserts",
    "flush_width_zero", "flush_width_1_7", "flush_width_8_15",
    "flush_width_16_31", "flush_width_ge32", "rerank_count", "result_count",
]
PAIR = re.compile(r"([a-zA-Z0-9_]+)=([0-9.]+)")

for path in sys.argv[1:]:
    values = {}
    rows = 0
    with open(path) as fh:
        for line in fh:
            if "ec_diskann_scan_profile" not in line:
                continue
            rows += 1
            for key, val in PAIR.findall(line):
                values.setdefault(key, []).append(float(val))
    print(f"FILE {path} rows {rows}")
    for key in KEYS:
        series = sorted(values.get(key, []))
        if not series:
            continue
        mean = statistics.fmean(series)
        p50 = series[int(0.50 * (len(series) - 1))]
        p95 = series[int(0.95 * (len(series) - 1))]
        print(f"{key} mean={mean:.2f} p50={p50:.2f} p95={p95:.2f} "
              f"min={series[0]:.2f} max={series[-1]:.2f}")
    total = values.get("total_us")
    frontier = values.get("frontier_us")
    if total and frontier:
        share = sum(frontier) / sum(total) * 100.0
        print(f"frontier_share_of_total_pct={share:.1f}")
    widths = [values.get(k, []) for k in (
        "flush_width_zero", "flush_width_1_7", "flush_width_8_15",
        "flush_width_16_31", "flush_width_ge32")]
    if all(widths):
        sums = [sum(w) for w in widths]
        flushes = sum(sums)
        pct = [s / flushes * 100.0 for s in sums]
        print("flush_width_pct zero={:.1f} w1_7={:.1f} w8_15={:.1f} "
              "w16_31={:.1f} ge32={:.1f}".format(*pct))
    print()
