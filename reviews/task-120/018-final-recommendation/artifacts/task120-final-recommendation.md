# Task 120 Final Recommendation

This is the final Task 120 synthesis from the evidence currently committed on
the branch. It does not add new benchmark measurements, does not run AWS, and
does not make a SPIRE product/default claim.

The overall recommendation is **do not promote any Task 120 SPIRE coarse-rerank
location as a default**. Close the measurement program as no-promotion for the
tested local surfaces, and keep distributed near-data rerank as an iteration
candidate only if a future user explicitly approves the remaining AWS product
claim matrix.

## Per-Location Decision

| Location | Decision | Evidence | Rationale |
| --- | --- | --- | --- |
| Local leaf / block coarse-rerank | Shelve the tested `l2` per-leaf block cap; do not promote | Packet 008 | It cuts candidates but collapses recall at 50k/100k. Wider local rerank in packet 010 does not recover the gap. |
| Local candidate/rerank budgets | Diagnostic-only; do not promote wider exact rerank or candidate caps | Packet 010 | Wider rerank increases heap work without recall gain at 10k/50k/100k. |
| Topology route-set refinement | Iterate only; no default | Packet 011 | `nprobe=96` route overfetch plus rowcap25k is locally recall-positive, but 100k latency remains high and the AWS/product claim matrix is not complete. |
| Distributed near-data rerank | Iterate only; no product claim | Packets 015 and 017 | The path functions and local multi-node 10k/50k/100k evidence exists, but AWS 1M decision-grade distributed recall/latency was not completed or approved for rerun. |
| Durable summaries / sidecars / defaults | Do not introduce | Packet 016 | No measured location justifies a durable format/default, so only the conservative fallback invariant is recorded. |

## Requirement Audit

| Requirement | Evidence | Status |
| --- | --- | --- |
| Phase 1 stage containment and budget diagnostics | Packets 001-007, corrected by packet 009 | Satisfied for the measured local surfaces after the diagnostic rerun. The corrected Phase 1 result shows the flat local gap is route/leaf coarse selection, not a later rerank frontier. |
| Phase 2 local leaf coarse-rerank A/B at 10k/50k/100k | Packet 008 | Satisfied for the tested recursive RaBitQ `full` vs `l2` policy. Result is negative for `l2`. |
| Phase 3 candidate budget/rerank policy curves | Packet 010 | Satisfied. Wider exact rerank and candidate caps remain diagnostic-only. |
| Phase 4 route-set refinement | Packet 011 | Satisfied locally. Route overfetch + rowcap is promising as a hypothesis, not a default. |
| Phase 5 distributed near-data rerank shipping/merge behavior before any distributed claim | Packet 017, with packet 015 as partial AWS functional evidence | Satisfied for the local multi-node gate. Not satisfied for an AWS/product claim. |
| Phase 6 maintenance/staleness/fallback invariants | Packet 016 | Satisfied as a no-promotion invariant record. |
| Final promote/iterate/shelve recommendation | This packet | Satisfied by recommending no promotion, shelve local leaf/block cap, and iterate only on topology/distributed hypotheses. |

## Evidence Summary

Phase 1 corrected attribution:

- Packet 009 reran the diagnostic after fixing flat-index target-block
  attribution.
- At 100k, recall rose from `0.7695` at nprobe `8` to `0.9205` at nprobe `32`,
  while corrected containment showed the remaining misses were route/leaf coarse
  selection misses rather than a false block-pruning stage.

Phase 2 local leaf/block coarse-rerank:

- Packet 008 measured recursive RaBitQ `full` vs `l2` block pruning.
- At nprobe `32`, `l2` recall collapsed from `0.9725` to `0.5505` at 50k and
  from `0.9310` to `0.5060` at 100k.
- The tested per-leaf block cap is not recall-safe.

Phase 3 local budget/rerank policy:

- Packet 010 measured candidate caps and exact rerank widths at 10k/50k/100k.
- At nprobe `32`, 100k default and width-500 both reported recall `0.9310`;
  width-500 increased p50 from `25.396 ms` to `44.393 ms`.
- The remaining gap is not recovered by exact rerank over the routed frontier.

Phase 4 topology refinement:

- Packet 011 measured route overfetch and routed-row caps.
- At 100k, nprobe `96` improved recall to `0.9975`, but p50/p95 was
  `66.596/96.757 ms`.
- `rowcap25k` preserved the measured recall while cutting candidate/object
  volume locally, but this remains a hypothesis requiring product-scale
  distributed validation before any product claim.

Phase 5 distributed near-data rerank:

- Packet 015 proved the AWS distributed path can function at 1M with real remote
  leaf placement, but reviewer feedback correctly classified it as partial: the
  decision-grade distributed recall/latency steps remained pending.
- Packet 017 completed the required **local multi-node distributed gate** with
  one local coordinator and three local worker PostgreSQL instances on the same
  physical machine, distinct node identities `2`, `3`, and `4`, static remote
  placements, and `EcSpireDistributedScan`.
- Packet 017 local multi-node nprobe `96` default results:

| Scale | recall@10 | p50 | p95 | p99 | SPIRE index |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k | 0.9855 | 44.638 ms | 50.035 ms | 54.638 ms | 9.4 MiB |
| 50k | 0.9900 | 59.243 ms | 82.910 ms | 87.674 ms | 40.7 MiB |
| 100k | 0.9880 | 98.876 ms | 113.926 ms | 134.894 ms | 79.7 MiB |

Packet 017 also reported `status=ready`, `result_source=remote_heap_candidates`,
`local_pid_sum=0`, `dispatch_sum=600`, `timeout_sum=0`, and
`degraded_skip_sum=0` for every real-corpus suite row.

## Closeout Decision

Close Task 120 as a measurement program with **no promoted SPIRE default**:

- Do not promote local leaf/block pruning.
- Do not promote wider local exact rerank.
- Do not promote route overfetch + rowcap as a default.
- Do not claim distributed SPIRE product readiness.
- Keep the Phase 6 fallback invariant as the rule for any future task that
  introduces durable summaries, sidecars, or defaults.

AWS remains opt-in only. Under the current user instruction, no AWS benchmark
may run without explicit approval for the specific matrix. If a future product
claim is desired, the next task should request explicit AWS approval and run a
new 1M distributed matrix with:

- distributed recall@10/latency at nprobe `64/96` and past the `96` knee after
  rebuilding with a larger `top_graph_search_list_size`;
- rowcap25k on the distributed path;
- shipped rows/bytes plus merge/dedupe counters at decision-grade query count;
- a clear resolution or explicit scope-out of the full-row
  `requires_remote_heap_resolution` caveat.

Until that opt-in matrix exists, Task 120 should not be used for a product
default or product-readiness claim.
