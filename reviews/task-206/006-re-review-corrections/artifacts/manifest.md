# Task 206 re-review correction manifest

- code checkpoint: `366a7973d` (includes the persisted-marker regression test)
- task bucket: `reviews/task-206/006-re-review-corrections/`
- lane: PG18 attribution-feature physical diagnostic
- matrix: three-owner physical distann, `build_shards=1`, BW64/H8, top-k 200
- A/B: persisted-head effective seed count 128 versus 200
- scales: 10k / 50k / 100k
- session GUC: `ec_distann.scan_profile_notice=on`
- command: `ecaz bench suite --config artifacts/task206-feature-seed-ab.json`
- install: `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18,distann-head-attribution-benchmark`
- timestamp: 2026-08-04, America/Los_Angeles
- surfaces: isolated one-index-per-table physical fixtures; no shared-table
  measurement

The feature build is an observability/seed-control diagnostic. Its latency
must not be pooled with the clean release matrix in packet 005. Every cited
result must come from the packet-local suite `results.jsonl` and the compact
evidence under `artifacts/run/`.

The completed suite is recorded in `artifacts/run/results.jsonl` and
`artifacts/run/suite-manifest.json`. The raw per-node logs and generated
predictions remain uncommitted operational output; `result-summary.md` and
`validation-final.md` contain the compact cited evidence.
