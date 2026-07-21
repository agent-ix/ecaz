---
task: 191
packet: 004-closeout
role: coder
status: review_requested
head_sha: 0cd579da4e70d5656160f1d4d3501f1d22087568
date: 2026-07-20
decision: complete
---

# Review request: Task 191 closeout

## Outcome

Task 191 is complete. Fixed deterministic global-ranked payload windows of 10
are the production physical `ec_distann` scan path. The release A/B preserves
recall and semantics while materially improving mean and tail latency at every
required scale. No persistent format changes, index rebuild, production tuning
knob, or absolute latency gate were introduced.

The prior extension binary remains the rollback: reinstalling it restores eager
scans against the same indexes because Task 191 changed no durable bytes.

## Final requirement audit

| Requirement | Disposition |
| --- | --- |
| FR-079/FR-081/NFR-019/test contract and rollback | Complete in packet 001 |
| Fixed production window 10; no production reloption/GUC | Complete |
| Feature-only eager A/B override | Complete; explicit-zero forwarding regression covered |
| Genuine external TOAST projection/qual | Pass |
| Stable-prefix reuse and no duplicate remote fetch | Pass; duplicate counter zero in semantics and all full-scale arms |
| Later-window owner failure aborts | Pass |
| Non-overlapping merge/associate stages | Pass at semantic and all full-scale gates |
| Clean pre-output suite runner descriptor | Pass in packets 002–004 |
| 10k/50k/100k recall/latency/storage A/B | Pass; packet 003 PROMOTE |
| Normal production build isolation | Pass; no GUC in `.so` or SQL, three-owner serving smoke passes |
| Retained baseline and Task 187 handoff | Complete in task/roadmap updates and below |

## Retained result

| Scale | Recall | Eager → production mean | Mean gain | Eager → production p95 | p95 gain |
| --- | ---: | --- | ---: | --- | ---: |
| 10k | 0.9990 | 34.00 → 21.70 ms | 36.2% | 39.70 → 25.10 ms | 36.8% |
| 50k | 0.9685 | 36.90 → 22.70 ms | 38.5% | 44.20 → 26.20 ms | 40.7% |
| 100k | 0.9625 | 39.00 → 23.70 ms | 39.2% | 49.20 → 27.20 ms | 44.7% |

Payload bytes and remote candidates fall 72.2%, 74.5%, and 75.3% at
10k/50k/100k with identical storage/construction and zero duplicate requests.

## Task 187 handoff

Task 187 is now unblocked. Freeze this retained production baseline before its
attribution work: `training_landmarks_exact`, cap 4,096, 32 seeds, BW4/H100,
graph degree 32, RaBitQ neighbor scoring, and lazy10 payload windows. At 100k,
traversal remains 7.849 ms of the 23.70 ms wall mean (33.1%), so Task 187's
conditional skip does not apply. The task file records the generation, head,
seed, and query digests required to reproduce the baseline.

## Feedback disposition

The outside review accepted packets 001–004 and identified one P2 precision
finding: the no-qual payload bound omitted tombstone or snapshot-invisible
ranked slots. Commit `0cd579da4` corrects NFR-019, ADR-085, the test matrix,
and packet 001 to state the no-skip bound and the `t`-skip bound
`min(D, W × ceil((k+t)/W))`. This is specification-only; the accepted
benchmark and semantic evidence remain valid and require no rerun.
The review requests remain available for outside review; this packet does not
self-close them.

Evidence and exact commands are indexed by
[`artifacts/manifest.md`](artifacts/manifest.md).
