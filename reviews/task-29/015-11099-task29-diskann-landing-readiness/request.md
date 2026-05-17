# Task 29 DiskANN Landing Readiness

## Request

Review the Task 29 local landing-readiness summary for the current DiskANN
branch.

Measured callback-smoke commit:
`b1cee686154114fc5e15665ad99f45f8e5a1feb7`

This packet keeps the fresh PG18 callback smoke log and the raw Task 29
baseline/latency logs it cites under `artifacts/`.

## Summary

Local Task 29 evidence now supports outside review / landing discussion.

- Correctness smoke: focused PG18 `test_ec_diskann_` callback coverage passed
  `19 passed; 0 failed` at `b1cee686`. The run covers build, ordered scan,
  insert/duplicate handling, planner gating, session list-size override, and
  vacuum repair paths.
- Recall blocker: packet `11096` shows the binary-sidecar prefilter closes the
  earlier grouped-PQ miss. Fresh local real-10k DiskANN recall@10 is `0.9965`
  at L=64, `0.9970` at L=200, and `0.9975` at L=800.
- Scan latency: packet `11098` shows the heap/early-stop scan changes preserve
  recall and reduce L=800 mean query time from the pre-optimization `247.34 ms`
  to `68.90 ms`. The L=800 latency pass measured p50/p95/p99
  `66.7/76.9/80.0 ms`.
- Build/load/storage: the fresh sidecar real-10k build completed in `503.10s`
  total: copy `4.27s`, encode `4.55s`, index build `492.13s`. Fresh DiskANN
  index size was `4.7 MiB` / `494.0 B` per row.
- Reference row: `ec_hnsw` on the same corpus measured recall@10 `0.9700`,
  p50/p95/p99 `33.1/39.4/49.1 ms`, and index size `13.0 MiB`.

## Recommendation

No remaining local Task 29 landing blocker was found.

The branch should move to outside review / landing discussion with this local
state:

- binary-sidecar prefilter is the right default `auto` recall path;
- heap plus early-stop persisted scan descent is the right latency checkpoint;
- focused PG18 DiskANN callback coverage passes on the current head;
- local real-10k benchmarking is sufficient for this lane's initial tuning
  decision.

The main remaining question is not correctness: it is whether the current
fresh build time is acceptable for the first landing slice. Product/AWS/RDS
benchmarking remains out of scope for Task 29's local lane unless reviewers ask
for it explicitly.

## Artifacts

See `artifacts/manifest.md`.
