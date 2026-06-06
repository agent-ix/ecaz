# Task 84 Closeout Manifest: No Bounded Recovery Policy

- Task: `plan/tasks/84-spire-1m-recall-recovery-without-candidate-inflation.md`
- Packet: `reviews/task-84/006-closeout-no-bounded-recovery/`
- Branch: `task-84-spire-recall-recovery`
- Head SHA at packet creation: `3f3fdd9d9`
- Packet type: closeout / no accepted recovery policy

## Evidence Sources

This closeout cites packet-local Task 84 evidence:

- `reviews/task-84/001-enriched-block-context-diagnostic/`
  - AWS 1M/q500 retained baseline.
  - `recall@10=0.9832`, `candidate_sum=9,213,846`.
  - Miss split: `3` routing misses, `81` selected-leaf block-pruning or
    candidate-cap misses.
  - Enriched selected-leaf context in
    `artifacts/selected-leaf-miss-enriched-context.tsv`.
- `reviews/task-84/002-route-prior-calibration-sweep/`
  - AWS 1M/q500 route-prior sweep.
  - Weights `0.02`, `0.05`, `0.10`, and `0.20` all remained at
    `recall@10=0.9832` with unchanged miss split `4916/3/81`.
- `reviews/task-84/005-multi-index-knn-frontdoor-fix/`
  - AWS 1M/q500 k=3 summary representative result after the multi-index KNN
    front-door fix.
  - `global1152`: `recall@10=0.9832`, `candidate_sum=9,213,742`.
  - `global1280`: `recall@10=0.9846`, `candidate_sum=10,237,430`.
- Task 83 controls from
  `reviews/task-83/002-global-cap-recovery-sweep/`
  - `global1280`: `recall@10=0.9846`, `candidate_sum=10,237,554`.
  - `global1536`: `recall@10=0.9876`, `candidate_sum=12,284,852`.
  - `global1664`: `recall@10=0.9892`, `candidate_sum=13,308,518`.

## Closeout Analysis Artifacts

- `idealized-rescue-coverage.tsv`
  - Computes the best possible selected-leaf recall if every missed truth row
    satisfying a rank or score-margin predicate were rescued.
  - This is an oracle upper bound, not an implementation result.
- `per-query-rank-rescue-upper-bound.tsv`
  - Computes query-level rank-window upper bounds and the minimum candidate
    rows added if the rescue only triggered on the missed queries.
  - A real score-margin or ambiguity trigger would add candidates on additional
    non-missing queries.
- `analysis-commands.md`
  - Records the exact `awk` commands used to derive the closeout tables.
- `cloud-status-final-paused-closeout.log`
  - Command: `target/debug/ecaz cloud status --profile 1m --database postgres --log-file reviews/task-84/006-closeout-no-bounded-recovery/artifacts/cloud-status-final-paused-closeout.log`
  - Note: the status command printed the status to stdout; the packet artifact
    captures that output.
  - Result: `state: paused`, `db: 10.42.1.131 (i-06ace3e95ab942623)`,
    running cost `~$0.00/hr`, retained storage `~$8.00/mo`.

## Key Results

Idealized score-margin rescue is too small:

- margin `<=0.001`: recovers at most `3` truth rows, recall `0.9838`.
- margin `<=0.0025`: recovers at most `6` truth rows, recall `0.9844`.
- margin `<=0.005`: recovers at most `11` truth rows, recall `0.9854`.
- margin `<=0.01`: recovers at most `26` truth rows, recall `0.9884`.

Rank-window rescue is not product-safe as a bounded policy:

- `+128` blocks can recover at most `7` selected-leaf truth rows, matching the
  Task 83 `global1280` recall `0.9846` only under an oracle trigger.
- `+512` blocks can recover at most `30` truth rows and reach `0.9892`, which
  matches Task 83 `global1664` recall only under an oracle trigger.
- The per-query oracle `+2048` window still covers only `47` of the `81`
  selected-leaf truth misses while requiring at least `1,310,720` additional
  candidate rows on the missed queries alone.

## Decision

No Task 84 recovery policy lands.

The strongest tested scoring policies did not improve the retained AWS 1M/q500
recall point:

- route-prior calibration preserved candidates but recovered zero selected-leaf
  misses;
- k=3 summary representatives preserved candidates but recovered zero
  selected-leaf misses at `global1152`.

The remaining selective-rescue direction is not justified by the enriched
diagnostic:

- a narrow score-margin rescue has too little recall ceiling;
- a wider rank-window rescue needs hundreds to thousands of extra blocks and
  becomes an oracle-shaped version of the rejected blanket cap sweep;
- a real trigger would spend budget on non-missing queries, weakening the
  candidate-control argument further.

The next concrete recommendation is Task 85: treat SPIRE as a product-scale
Pareto program after this no-policy closeout, comparing any future structural
candidate-selection work against IVF/HNSW/DiskANN rather than continuing
single-knob Task 84 rescue slices.

AWS `1m` was paused at closeout.
