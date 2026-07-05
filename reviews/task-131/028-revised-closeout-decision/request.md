# Task 131 Packet 028: Revised Closeout Decision

## Summary

This packet replaces the rejected packet 024 closeout with the additional
evidence requested by reviewers. It does not mark the task closed; `Status:`
should flip only after an outside reviewer accepts this closeout.

The revised conclusion is: shelve Task 131's streaming global-threshold pruning
algorithm for the current SPIRE distributed surface, keep candidate-to-heap
streaming as structural infrastructure, and file the duplicate-result defect as
its own correctness task.

## Decisions

| Surface | Decision | Rationale |
| --- | --- | --- |
| Global merge before heap | Shelve / do not promote | It reliably cuts remote heap rows from 6000 to 2000 in three-worker reads and proved compact candidate scores can form a correct frontier, but latency stayed flat in the measured matrix and it does not target scan/scoring. Known gaps remain: no payload-bytes-avoided lane and no 100k `n1024/b2` cell. |
| Candidate-to-heap streaming | Keep / iterate as infrastructure | Packet 015 removed the candidate-phase barrier in the default path. Packet 021 proved overlap under skew. Packet 027's gate-off arms give normal-fixture scale numbers on the streaming path: 10k `n128/b4` recall `0.9985`, p50/p95/p99 `609.243/686.941/894.404 ms`; 50k `n128/b4` recall `1.0000`, p50/p95/p99 `2645.864/3287.777/3816.541 ms`. |
| Streaming global threshold feedback | Shelve / do not continue | Packet 025 proved summaries can make sound bounds available at 10k, but packet 027's full-query 10k/50k A/B rejects the initial-threshold early-stop path: returned IDs are byte-identical between arms, actual scan `leaf_block_skipped_sum=0` on every remote node, and latency is flat/regressed. The diagnostic ceiling also collapses at 50k: 18,931 skipped rows out of 189,322,205 available (`0.010%`). |
| New bound metadata | Do not promote from Task 131 | Leaf-block summaries add measurable remote storage and materialization cost, and the only measured threshold use has no practical scan-time payoff. If summaries are pursued later, they need a separate metadata/version/maintenance task with insert/delete/vacuum/split/stale-summary invariants. |
| Duplicate distributed result IDs | File separately / do not fix here | Packet 027 identity artifacts show duplicate corpus IDs inside top-10 results in both A/B arms. This does not alter Task 131's inter-arm no-op conclusion, but it qualifies recall language and is filed as `plan/tasks/137-spire-distributed-result-deduplication.md`. |

## Stop-Condition Readout

1. Phase 3 tested with bound source enabled: satisfied by packet 025 and packet
   027. Packet 025 rebuilt 10k `n128/b4` with `ec_spire.leaf_block_rows=64` and
   showed bounds available. Packet 027 ran 10k and 50k `n128/b4` with summaries
   enabled in `pgoptions` and `load_session_gucs`.
2. Phase 3 decision backed by measured evidence: satisfied for the shelve
   decision. Packet 027 shows matched inter-arm returned IDs, matched current
   recall metrics, zero actual scan blocks skipped, and no matched-recall
   latency win. The 50k diagnostic skip ceiling is only `0.010%` of rows.
3. Phase 2 scale result stated: satisfied for 10k/50k `n128/b4` by packet 027
   gate-off normal-fixture arms. A 100k normal-fixture Phase 2 latency cell is
   explicitly scoped out because Phase 3 is shelved and no promotion claim is
   being made.
4. Phase 1 gaps acknowledged: satisfied. Payload-bytes-avoided was not measured
   in a payload lane, and 100k `n1024/b2` did not run.
5. Disk cleared: satisfied. After packet 027, generated run artifacts were
   cleaned and the workspace reported `124G` free (`88%` used).
6. Status flip only after reviewer acceptance: not done here. This packet asks
   for review and leaves the task in progress until accepted.

## Evidence Readout

### Phase 0

Phase 0 instrumentation is retained as useful. Packets 010 and 011 added the
scan-time and production-read profile fields Task 131 needed: selected/scanned
PID counts, candidate row counts, local kth scores, score timing, and sound-bound
availability/missing counters.

### Phase 1

Global merge-before-heap is a measured dead end for the target latency problem:

- `10k n128/b4`: recall `0.9985`, query p50/p95/p99
  `591.330/675.278/885.398 ms` -> `595.324/712.235/892.992 ms`; heap rows
  `6000` -> `2000`.
- `10k n1024/b2`: recall `0.9975`, query p50/p95/p99
  `535.275/649.270/712.482 ms` -> `534.449/627.665/699.608 ms`; heap rows
  `6000` -> `2000`.
- `50k n128/b4`: recall `1.0000`, query p50/p95/p99
  `2582.977/2953.783/3514.706 ms` -> `2593.483/2981.787/3492.307 ms`; heap
  rows `6000` -> `2000`.
- `50k n1024/b2`: recall `0.9980`, query p50/p95/p99
  `663.809/795.704/904.363 ms` -> `663.340/718.746/859.830 ms`; heap rows
  `6000` -> `2000`.
- `100k n128/b4`: recall `1.0000`, query p50/p95/p99
  `5366.063/6413.783/6711.519 ms` -> `5357.062/6199.141/6616.327 ms`; heap
  rows `6000` -> `2000`.

Known gaps are not hidden: the payload-bytes-avoided evidence was not captured
in a payload lane, and `100k n1024/b2` was not run.

### Phase 2

Candidate-to-heap streaming remains useful infrastructure. Packet 021's skewed
fixture showed `heap_started_before_all_candidates_done=1`,
`fast_heap_before_slowest_heap=1`, and `heap_start_minus_candidate_done_p50=-252
ms` with recall `1.0000`.

Packet 027's gate-off production-read arms provide normal-fixture scale timing
for the current streaming path:

- 10k `n128/b4`, summaries on, full 200-query set: current recall metric
  `0.9985`, p50/p95/p99 `609.243/686.941/894.404 ms`.
- 50k `n128/b4`, summaries on, full 1000-query set: current recall metric
  `1.0000`, p50/p95/p99 `2645.864/3287.777/3816.541 ms`.

These numbers are not a product-win claim; they are the scale readout requested
for the structural path.

### Phase 3

The reviewer-requested summaries-enabled path was tested and rejected:

- Packet 025, 10k summaries-on diagnostic: bounds available
  (`sound_bound_missing_sum=0`), threshold rows skipped
  `40489 / 754126 = 5.37%`, remote index storage delta about `+1.7` to
  `+1.9 MiB` per remote node, and materialization time increased from about
  `2.5-3.0s` to `13.2-16.6s` per remote node.
- Packet 027, 10k A/B: off/on returned-ID JSONL is byte-identical; current
  recall metric matched at `0.9985`; latency p50/p95 is
  `609.243/686.941 ms` off vs `613.294/728.343 ms` on; actual scan profile
  `leaf_block_skipped_sum=0` for every node.
- Packet 027, 50k A/B: off/on returned-ID JSONL is byte-identical; current
  recall metric matched at `1.0000`; latency p50/p95 is
  `2645.864/3287.777 ms` off vs `2659.226/3191.039 ms` on; actual scan profile
  `leaf_block_skipped_sum=0` for every node.
- Packet 027, 50k diagnostic threshold ceiling: rows skipped
  `4021 + 8980 + 5930 = 18931` out of `189322205` available, or `0.010%`.
  Blocks skipped `7018 / 3006326 = 0.23%`. The ceiling collapses from about
  `5.4%` at 10k to `0.01%` at 50k.

The threshold endpoint remains gated/default-off and should not be promoted.
Packet 026's version-skew behavior is acceptable only for a gated experiment:
gate-on against a remote without `ec_spire_remote_search_with_initial_threshold`
fails the candidate query through strict/degraded handling rather than silently
falling back. The requested threshold-carrying fault-path unit test was not
added because the A/B result shelves the path; add it before any future
promotion or renewed experiment.

### Phase 4

Task 131 should not promote new bound metadata. Leaf-block summaries are a real
bound source, but packet 027 shows they do not support a useful threshold-prune
protocol at 50k. Any future summary or bound metadata work belongs in a separate
task with a format/version plan and maintenance/fallback invariants for insert,
delete, vacuum, split, movement, remote version skew, and stale summaries.

### Duplicate-ID Caveat

Packet 027's identity harness found a separate distributed result-quality defect:

- 10k threshold-off: 183/200 queries contain duplicate IDs in top-10; worst case
  has only 4 distinct IDs.
- 50k threshold-off: 1000/1000 queries contain duplicate IDs in top-10; worst
  case has only 4 distinct IDs.

The defect is shared by both A/B arms, so it does not change Task 131's
threshold-gate no-op conclusion. It does mean all recall language in this
closeout is "current duplicate-tolerant recall metric" until Task 137 fixes the
distributed result dedupe and metric behavior.

## Review Ask

Please review this as the revised Phase 5 closeout. If accepted, Task 131 can be
closed with the decisions above. If rejected, the next work should be whatever
specific closeout gap remains; do not reopen the retired heap-side path.
