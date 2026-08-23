---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 quality-gate repeat preregistration

Status: measurement complete; product quality gate failed at 50k. Review is
requested for the calibration result and the resulting open-task disposition.
No Task 167 closeout is claimed.

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

## Results

All six isolated suite steps succeeded. The same-state calibration eliminated
the earlier instrument ambiguity: ordinary and exact-scorer distinct recall
matched exactly in every repeat (`0.999000` at 10k and `0.954500` at 50k).

The five 10k repeats produced these preregistered bands:

- inserted-neighborhood: physical sample standard deviation `0.000000`, mean
  deficit `0.014385`, derived band `0.015`;
- heldout: physical sample standard deviation `0.000223607`, mean deficit
  `0.005600`, derived band `0.007`.

The fresh arm was deterministic to six decimals and every 10k baseline repeat
fell inside its derived band. The repeated 50k heldout result did not:
physical `0.843000`, fresh `0.869250`, deficit `0.026250`, exceeding the fixed
`0.007` band by `0.019250`. This closely reproduces packet 043's `0.025987`
deficit with all 200 heldout queries and pinned search GUCs. The threshold was
not widened.

The append-when-room timing ratios remained noisy and showed no consistent
gain across these repeats, so the shortcut remains disabled by default.

## Disposition

Task 167 remains open. The 50k product-quality degradation is real rather than
a scorer or query-subset artifact and must be diagnosed. One confound is now
material: the measured physical graph contains both the shipped robust-prune
insert arm and the subsequently enabled, rejected append-when-room diagnostic
arm. The next slice must place the exact quality measurement immediately after
the shipped/default arm, before the diagnostic candidate mutates the graph.

The compact cited values and derivation are in
[`artifacts/cited-results.log`](artifacts/cited-results.log); the raw structured
source is `artifacts/calibration-suite/results.jsonl`.
