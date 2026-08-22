---
agent: codex
role: coder
model: GPT-5
date: 2026-08-21
seq: 1
---

# Task 167 recovery harness correction

Status: review-open; not a runtime closeout request.

Please review code checkpoint `c231d1332` for the Task 167 append A/B sample
accounting defect discovered during recovery.

The caller declared, reported, and used a 5-trial × 32-row workload when
checking insert-work counters, but `measure_task167_insert_arm` independently
executed only 3 trials × 16 rows. A current-head physical run would therefore
perform 48 candidate inserts and then require counters for 160, aborting before
it could produce decision-grade evidence.

The checkpoint defines the preregistered 5 × 32 workload once, passes those
dimensions into every measured arm, and uses the same constants for output and
counter expectations. A focused unit regression pins the resulting 160-row
sample.

Validation is summarized in [`artifacts/validation.log`](artifacts/validation.log)
with provenance in [`artifacts/manifest.md`](artifacts/manifest.md). No PG18
runtime or benchmark result is claimed here. The next packet will carry the
production-feature synthetic/10k smoke and, only after those gates pass, the
fresh 10k/50k/100k closeout matrix.

