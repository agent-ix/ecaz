# Task 131 Packet 024: Phase 5 Closeout Decision

## Summary

This packet closes the Task 131 investigation with separate decisions for the
four surfaces named in Phase 5. The short answer is that the full streaming
global top-k pruning algorithm does not currently beat the Task 123 baseline at
matched recall by enough to justify more protocol complexity on the default
production-read surface.

The only result that transfers directly to streaming top-k is small: compact
remote candidate scores are trustworthy enough for a coordinator global
frontier. That is useful for threshold reasoning, but it did not require the
explicit-subset heap endpoint or the wider heap-side latency matrix, and those
pieces should not be promoted as streaming-top-k scaffolding.

## Decisions

| Surface | Decision | Rationale |
| --- | --- | --- |
| Global merge before heap | Shelve / do not promote as a product path | It reduces remote heap rows from `6000` to `2000` in measured three-worker reads and proves the compact-score frontier is correct, but query latency is flat in the representative matrix and the shape does not address the scan/scoring bottleneck. |
| Candidate-to-heap streaming | Iterate / keep as structural production behavior | Packet 015 removes the candidate-phase barrier in the default path and packet 021 proves fast workers can start heap receive while a slow worker is still producing candidates. Correctness and cleanup guards are covered, but this is an enabling change rather than a measured 10k/50k/100k product win. |
| Streaming global threshold feedback | Shelve for the current default surface | Packets 020 and 022 show the diagnostic protocol can derive the threshold, but the current no-summary production-read indexes expose no sound bound to compare it against. A recall-safe early-stop rule would have zero safe skip opportunities. |
| New bound metadata | Iterate only as a separate metadata-gated experiment | Packet 023 identifies leaf block summaries as the near-term sound bound source, but representative indexes do not build them. Durable metadata needs its own A/B, storage accounting, format/version plan, and maintenance/fallback invariants before it belongs in the default read path. |

## Evidence Readout

Phase 0 is satisfied by the scan-time instrumentation in packets 010 and 011.
Those packets added selected/scanned PID counts, candidate row counts, local kth
scores, score timing, and sound-bound availability/missing counters to the
production-read profile surface.

Phase 1 rejects global merge-before-heap as the Task 131 win. The strongest
measured signal is heap work reduction, not query-latency reduction:

- `10k n128/b4`: recall `0.9985`, query p50/p95/p99 `591.330/675.278/885.398 ms` -> `595.324/712.235/892.992 ms`; heap rows `6000` -> `2000`.
- `10k n1024/b2`: recall `0.9975`, query p50/p95/p99 `535.275/649.270/712.482 ms` -> `534.449/627.665/699.608 ms`; heap rows `6000` -> `2000`.
- `50k n128/b4`: recall `1.0000`, query p50/p95/p99 `2582.977/2953.783/3514.706 ms` -> `2593.483/2981.787/3492.307 ms`; heap rows `6000` -> `2000`.
- `50k n1024/b2`: recall `0.9980`, query p50/p95/p99 `663.809/795.704/904.363 ms` -> `663.340/718.746/859.830 ms`; heap rows `6000` -> `2000`.
- `100k n128/b4`: recall `1.0000`, query p50/p95/p99 `5366.063/6413.783/6711.519 ms` -> `5357.062/6199.141/6616.327 ms`; heap rows `6000` -> `2000`.

Phase 2 is accepted as structural progress. Packet 021's skewed fixture shows
`heap_started_before_all_candidates_done=1`, `fast_heap_before_slowest_heap=1`,
and `heap_start_minus_candidate_done_p50=-252 ms`, with recall `1.0000`.

Phase 3 is rejected for the current no-summary surface. Packet 022 completed the
real-scale 10k/50k cells for both representative shapes, and every completed
cell reported `sound_bound_available_sum=0` with zero threshold block/row skips:

- `10k n128/b4`: recall `1.0000`, p50 `613.826 ms`, p95 `673.385 ms`.
- `10k n1024/b2`: recall `0.9750`, p50 `528.689 ms`, p95 `672.335 ms`.
- `50k n128/b4`: recall `1.0000`, p50 `2703.202 ms`, p95 `3371.223 ms`.
- `50k n1024/b2`: recall `1.0000`, p50 `644.494 ms`, p95 `833.718 ms`.

The packet 022 suite failed during `100k n128/b4` setup because the workspace
filesystem filled, and `100k n1024/b2` was not reached. This packet does not
make a promotion claim that depends on those missing cells. The completed cells,
plus packet 023's code inspection, are enough for the narrower rejection: the
current default production-read indexes have no sound bound surface for Phase 3
early stop.

## Closeout Recommendation

Do not continue implementing coordinator-to-worker threshold plumbing on this
branch unless the task is explicitly re-scoped to a metadata-backed experiment.
For Task 131 as written, the measured outcome is:

- heap-side global merge is a dead end for the target latency problem;
- candidate-to-heap streaming is useful infrastructure and can remain;
- streaming threshold feedback is shelved on the current surface;
- bound metadata needs a separate measured design path before it can justify
additional production protocol complexity.

## Review Ask

Please review this as the Phase 5 closeout packet. If accepted, Task 131 should
be considered closed with the decisions above rather than expanded into another
round of heap-side or diagnostic threshold profiling.
