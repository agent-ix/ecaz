# Task 222 packet 003 artifact manifest

- Head SHA: `c9f79be4a756031b3f8301960fc0f57b77ae60d1`
- Task bucket / packet: `reviews/task-222/003-isolated-100k/`
- Timestamp: `2026-08-23T22:09:35-07:00`
- Lane / fixture: three-owner physical PG18, `ec_real_100k`, 200 held-out
  queries, top-k 10, 50 warm timed iterations plus 10 warmups
- Storage / rerank: RaBitQ physical generation, persisted head 4096,
  head-search/head-seed 32/32, beam width 4, hop limit 100, production lazy-10
- Isolation: one immutable generation and query surface across both arms; only
  `payload_projection` differs (`false` control, `true` candidate)
- Runner: `/home/peter/.cargo-target/release/ecaz bench suite`
- Command: `ecaz bench suite run --config reviews/task-222/003-isolated-100k/artifacts/task222-payload-projection-100k.json --log-file reviews/task-222/003-isolated-100k/artifacts/suite-final.log`
- Run directory: `/home/peter/.ecaz/clusters/task222-payload-projection-100k`
  (external to the repository; retained temporarily for packet 004's attested
  100k reuse and removed after the matrix)
- Suite result: one completed, zero failed/skipped/missing/stale; duration
  1,385,945 ms
- Decision: ADVANCE; see `decision.md`

## Durable artifacts

- `task222-payload-projection-100k.json`: checked-in SuiteConfig.
- `suite-final.log` and `suite-status.log`: successful run/status summaries.
- `run/suite-manifest.json`: command, config hash, source SHA, NFR-021
  registrations, step status, and timing.
- `run/results.jsonl`: normalized suite evidence (1,812 rows).
- `run/100k/distann-multinode-summary.log`: compact fixture, provenance,
  A/B, storage, stage-counter, and correctness lines cited by the decision.
- `run/100k/physical-{all-columns-control,projected-candidate}-{recall,latency}.log`:
  direct recall/latency and payload/stage-counter values.
- `run/100k/physical-{all-columns-control,projected-candidate}-predictions.json`:
  byte-identical result identities; common SHA-256 is recorded in
  `decision.md`.
- `run/100k/physical-head-membership.json`: suite-required immutable head
  membership evidence.

No corpus TSV, truth cache, cluster directory, PostgreSQL operational log,
poll snapshot, or intermediate failed-run exhaust is committed.
