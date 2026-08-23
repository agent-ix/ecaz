---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 failed-step result extraction

Status: review requested for code checkpoints `6d205bdbb`, `7b20d18fa`, and
`7e3d3d714`. No Task 167 closeout is claimed.

Packet 047 correctly stopped when its preregistered quality gate failed, but
the suite runner originally omitted the failed DistANN step from
`results.jsonl`. The three checkpoints make failed DistANN logs eligible for
report extraction, add typed Task 167 calibration/quality/insert metrics, and
recognize the hard-gate error form where the metric follows the error message
without the ordinary log prefix.

The report-only path was exercised against packet 047's original child log and
produced a structured `physical_benchmark_post_insert_exact_recall` row with
the exact failed values and `diagnostic_candidate_mutation_excluded=true`.
That regenerated result is committed in packet 047; the benchmark was not
rerun.

The two focused regression tests pass. A broader suite-module diagnostic ran
91 tests: 86 passed and five unrelated existing expansion expectations failed
(seed-variant serialization and the physical-head-membership expected-artifact
list). This packet does not claim that broader module run is green and does not
expand Task 167 into those unrelated failures.

Validation and provenance are in
[`artifacts/manifest.md`](artifacts/manifest.md).
