#!/usr/bin/env python3
"""Inject the Task 106 gap-2 (HNSW x grouped-PQ) flush-width histogram steps
into the AWS targeted lane configs.

Gap 2 needs the flush-width histogram (ADR-077 9.2) that decides whether a
grouped-PQ traversal block kernel is justified. The new measure-first probe
(commit 3e5a837af) records that histogram on the batch-on arm; this adds the
HNSW grouped-PQ (pq_fastscan) load + recall + latency cells so a bench run
captures it. The histogram (candidate width per traversal boundary) is
host-independent, so it is identical on both lanes; both are kept for config
symmetry and a cross-lane confirmation. Latency on/off here is a noise pair
(scoring is unchanged by the probe) — the deliverable is the width histogram,
not a batch speedup A/B.

Idempotent: existing groupedpq steps are dropped before re-appending.
Run from repo root: python3 reviews/task-106/004-aws-targeted-bench/gen_groupedpq_gap2_steps.py
"""
import json
import os

PACKET = "reviews/task-106/004-aws-targeted-bench"
STAGED = "/var/lib/pgsql/18/datasets/staged-task106-targeted"

# scale token -> staged corpus stem (matches the IVF/SPIRE load cells)
SCALES = [
    ("10k", "ec_real_10k"),
    ("50k", "ec_real_50k"),
    ("100k", "ec_real_100k"),
    ("1m", "ec_real_ann_benchmarks_anchor"),
]

LANES = ["aws-intel", "aws-graviton"]
SWEEP = [40, 80, 120]


def groupedpq_steps(lane, artifact_dir):
    lane_tag = lane
    lane_slug = lane.replace("aws-", "")
    steps = []
    for scale, stem in SCALES:
        prefix = f"t106_aws_{lane_slug}_{scale}_hnsw_groupedpq"
        common = {
            "prefix": prefix,
            "profile": "ec_hnsw",
        }
        # load
        steps.append({
            "kind": "load",
            "name": f"load-{scale}-hnsw-groupedpq",
            "tags": ["task106", "aws-targeted", lane_tag, f"real{scale}",
                     "hnsw", "hnsw-pqfastscan-groupedpq", "gap2-flush-width"],
            **common,
            "bits": 4,
            "dim": 1536,
            "corpus_file": f"{STAGED}/{stem}_corpus.tsv",
            "queries_file": f"{STAGED}/{stem}_queries.tsv",
            "manifest_file": f"{STAGED}/{stem}_manifest.json",
            "allow_manifest_mismatch": True,
            "reloptions": [],
            "storage_format": "pq_fastscan",
            "m": [16],
            "ef_construction": 128,
            "log_file": f"{artifact_dir}/load-{scale}-hnsw-groupedpq.log",
        })
        for state in ("on", "off"):
            opt_tag = f"option=batch-{state}"
            ab_tag = "candidate_batch" if state == "on" else "scalar-ab"
            base_tags = ["task106", "aws-targeted", lane_tag, f"real{scale}",
                         "hnsw", "hnsw-pqfastscan-groupedpq",
                         "storage_format=pq_fastscan", "traversal_score_mode=pq",
                         "gap2-flush-width", ab_tag, opt_tag]
            gucs = None if state == "on" else ["ec_hnsw.candidate_batch_scoring=off"]
            # recall (parity check; host-independent)
            rc = {
                "kind": "recall",
                "name": f"recall-{scale}-hnsw-groupedpq-batch-{state}",
                "tags": base_tags + ["recall"],
                **common,
                "k": 10,
                "sweep": SWEEP,
                "queries_limit": 200,
                "truth_cache_dir": f"{artifact_dir}/truth-cache",
                "log_output": f"{artifact_dir}/recall-{scale}-hnsw-groupedpq-batch-{state}.log",
            }
            if gucs:
                rc["session_gucs"] = gucs
            steps.append(rc)
            # latency (carries the candidate-batch counters -> flush-width histogram on the on arm)
            lt = {
                "kind": "latency",
                "name": f"latency-{scale}-hnsw-groupedpq-batch-{state}",
                "tags": base_tags + ["latency"],
                **common,
                "k": 10,
                "sweep": SWEEP,
                "queries_limit": 200,
                "iterations": 200,
                "concurrency": 1,
                "cache_state": f"task106_{scale}_hnsw-groupedpq_batch-{state}",
                "task87_candidate_batch_counters": True,
                "truth_cache_dir": f"{artifact_dir}/truth-cache",
                "log_output": f"{artifact_dir}/latency-{scale}-hnsw-groupedpq-batch-{state}.log",
            }
            if gucs:
                lt["session_gucs"] = gucs
            steps.append(lt)
    return steps


def main():
    for lane in LANES:
        path = os.path.join(PACKET, f"task106-{lane}-targeted.json")
        cfg = json.load(open(path))
        artifact_dir = cfg["artifact_dir"]
        kept = [s for s in cfg["steps"]
                if "gap2-flush-width" not in s.get("tags", [])]
        cfg["steps"] = kept + groupedpq_steps(lane, artifact_dir)
        with open(path, "w") as fh:
            json.dump(cfg, fh, indent=2)
            fh.write("\n")
        print(f"{path}: {len(cfg['steps'])} steps "
              f"(+{len(cfg['steps']) - len(kept)} groupedpq)")


if __name__ == "__main__":
    main()
