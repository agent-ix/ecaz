---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 quality-gate repeat preregistration

Status: measurement preregistered; results pending. No acceptance or Task 167
closeout is claimed.

Packet 043 feedback requested 3–5 same-runtime 10k repeats, a repeated 50k
heldout observation, and a variance-derived automated quality gate. This packet
fixes that experiment before any new result is observed:

- five isolated production 10k fixtures;
- one isolated production 50k fixture to remeasure the prior heldout
  `physical_minus_fresh=-0.025987` observation;
- real staged corpora, PG18, three physical owners, graph degree 32,
  beam width 4, heap 32, hop rounds 100, exact fp32 truth, 200 heldout queries,
  and a separate 48-query inserted-neighborhood population;
- one index per table and a distinct external run directory and port range for
  every repeat;
- ordinary recall retained so the new pre-insert scorer calibration must pass;
  duplicate single-index query and concurrency drills are skipped because this
  packet measures exact-recall variance, not those already-complete gates.

## Preregistered gate derivation

For each population (`inserted_neighborhood`, `heldout`) independently:

1. Let `d_i = fresh_distinct_recall_i - physical_distinct_recall_i` for the
   five 10k repeats.
2. Compute the sample standard deviation `s_physical` of the five physical-arm
   values. The fresh arm must remain deterministic to six displayed decimals;
   otherwise calibration is invalid and no band will be set.
3. Set `band = ceil_0.001(max(0, mean(d_i)) + 2 * s_physical)`.
4. Require every calibration repeat to fall within its derived band. Encode
   the two bands as hard, population-specific suite gates; a future deficit
   above its band fails the step.

The 50k rerun is evaluated against the preregistered heldout band. If it remains
outside, Task 167 stays open for product diagnosis; the threshold will not be
widened from the 50k result. After encoding the gate, a final suite confirmation
must demonstrate the hard failure surface before closeout.

Configuration and initial provenance are in
[`artifacts/manifest.md`](artifacts/manifest.md).
