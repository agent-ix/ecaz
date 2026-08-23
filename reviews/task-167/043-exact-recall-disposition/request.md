---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 exact-recall disposition matrix

Status: review-open; harness checkpoint and full production 10k/50k/100k
disposition matrix complete. Outside reviewer disposition is required; no
acceptance or closeout is claimed.

Please review checkpoint `0bce21c05`.

Packet 041 replaced pairwise ANN overlap with exact fp32 ground truth. Packet
042 corrected the incremental prune distance and demonstrated a material 10k
improvement, from `0.805382` to `0.945809` inserted-neighborhood distinct
recall, but its suite stopped on a hard `0.002` physical-vs-fresh threshold.

FR-083-AC-4 requires a bench A/B against a fresh rebuild but specifies no
numeric non-inferiority threshold. The `0.002` value came from a different
DistANN criterion, and the outside reviewer requested the exact-truth values
side by side for disposition. This checkpoint therefore preserves `0.002` as
an explicitly labeled `reference_band`, emits `within_reference_band`, and
assigns `disposition=outside_review`; it no longer converts a valid quality
measurement into a suite-process failure. Measurement-integrity failures—wrong
plans, missing rows, malformed truth, reloption mismatch, or insufficient
held-out queries—still fail the step.

The focused Task 167 CLI tests pass 9/9. The immutable full-matrix config is
[`artifacts/task167-disposition-suite.json`](artifacts/task167-disposition-suite.json).
All three canonical suite steps succeeded. The decision-grade extract is
[`artifacts/cited-results.log`](artifacts/cited-results.log), with structured
results in `artifacts/final-suite/results.jsonl`.

## Exact post-insert fp32 recall

The values are physical / fresh / delta. Exact truth uses per-query distinct
source fingerprints, the fresh indexes match reloptions, and 152/200 queries
are held out from inserted-neighborhood selection.

| Scale | Inserted neighborhood | Heldout |
| --- | --- | --- |
| 10k | `0.940600 / 0.954985 / -0.014385` | `0.974342 / 0.977632 / -0.003289` |
| 50k | `0.952257 / 0.940972 / +0.011285` | `0.853289 / 0.879276 / -0.025987` |
| 100k | `0.933160 / 0.936632 / -0.003472` | `0.802632 / 0.808553 / -0.005921` |

Only the positive 50k inserted-neighborhood result is within the one-sided
`0.002` reference band. These are measurements for outside disposition, not a
threshold-adjusted pass claim. The 10k result also varies from packet 042's
`0.945809` on the same extension runtime, so the packet does not hide
run-to-run variation.

## Other required evidence

- Ordinary physical distinct recall is `0.9990 / 0.9535 / 0.9280` at
  10k/50k/100k. Mean physical latency is `17.50 / 20.40 / 19.50 ms`.
- Physical generation bytes are `242958336 / 1243553792 / 2498248704`, with
  graph-side amplification `1.238533 / 1.335307 / 1.353787`.
- Distributed/single insert throughput ratios are
  `0.171724 / 0.534592 / 0.694012`; all runner integrity checks pass.
- The real same-fixture append-enabled/disabled throughput ratios are
  `0.975741 / 0.997529 / 0.993053`. They honestly report `pass=false`; the
  change does not demonstrate a throughput win in this matrix.
- Concurrency, natural-retry proof, saturated shared-target coverage, routed
  delete/vacuum, UPDATE replacement, rollback, owner placement, serving, and
  topology gates pass at every scale. The 100k concurrency wave observed 23
  natural 2PC retries.

## Requested disposition

Please confirm whether checkpoints 040–043 resolve findings 1–8 from packet
039, and decide whether the exact-truth matrix satisfies FR-083-AC-4 or calls
for a further product-quality follow-up. Packets 027–030 remain explicitly
superseded and are not acceptance evidence.
