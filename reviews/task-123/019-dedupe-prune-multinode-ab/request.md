# Task 123 Review Request: Dedupe Prune Multi-Instance A/B

Please review packet 019 as the multi-instance follow-up for the packet 018
dedupe prune threshold fix.

The packet measures the fixed core algorithm on the representative 100k corpus
under local PG18 multi-instance production-read workloads:

- n128 / boundary replicas 4 / nprobe 96 / 200 queries.
- n1024 / boundary replicas 2 / nprobe 64 / 200 queries.
- Projection variants `id,source` and `id`.
- `ec_spire.pre_materialization_prune=on/off`.
- Default routing cap and `max_routed_candidate_rows=25000`.

Artifacts are under
`reviews/task-123/019-dedupe-prune-multinode-ab/artifacts/`.
The packet source of truth is
`artifacts/manifest.md`.

Important recovery note: the top-level suite completed n128 and built the
n1024 fixture, then failed during n1024 remote materialization because
operator cleanup removed assignment TSVs too early. I restarted the existing
n1024 PG data dirs, re-exported assignments from the coordinator, re-ran remote
materialization and registration, then ran the generated nested
`ecaz bench suite` for n1024. The recovered artifacts are packet-local and the
TSV scratch files were deleted before this request.

Key results:

- n1024 prune-on/off is flat:
  - source default: 777.223 ms on vs 780.018 ms off, recall 1.0000.
  - source rowcap25k: 777.627 ms on vs 777.575 ms off, recall 1.0000.
  - id default: 731.133 ms on vs 733.572 ms off, recall 1.0000.
  - id rowcap25k: 730.132 ms on vs 730.743 ms off, recall 1.0000.
- n128 prune-on/off is also flat, with p50 roughly 5.1s to 5.2s across variants.
- Per-worker heap payload bytes demonstrate the communications dimension:
  - `id,source`: 24,632,000 bytes per worker per 200-query variant.
  - `id`: 16,000 bytes per worker per 200-query variant.
- Remote heap resolution is healthy in these runs:
  `remote_heap_ready_dispatch_sum=600`,
  `remote_heap_failed_dispatch_sum=0`, no degraded remote skips.

Requested review focus:

1. Confirm the packet is acceptable multi-instance evidence for the corrected
   core algorithm.
2. Confirm the recovery path is sufficiently documented and the nested n1024
   suite artifacts are acceptable.
3. Confirm the conclusion: the dedupe prune threshold fix is correctness-needed
   but not a demonstrated latency win in this representative multi-instance
   matrix.
