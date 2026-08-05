# Artifact manifest

- code head: `a6289dddf`
- task bucket / packet: `reviews/task-206/004-corrected-closeout`
- lane: PG18, three-owner physical distann, fixed `build_shards=1`
- decision axis: BW64/H8; `head_seed_count` 128 versus 200; owner-oracle control
- NFR-021 preregistration: `task206-distributed` and `task206-owner-control`, both conforming before measurement
- command: `/home/peter/.cargo-target/release/ecaz bench suite audit --config artifacts/task206-corrected.json`
- run command: `/home/peter/.cargo-target/release/ecaz bench suite run --config artifacts/task206-corrected.json --artifact-dir artifacts/run`
- extension install: `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18`
- CLI build: `cargo build --release --bin ecaz --package ecaz-cli --offline`
- timestamp: pending corrected run
- corpus/query inputs: external staged `ec_real_10k`, `ec_real_50k`, `ec_real_100k`; no corpus files committed

The latency lane uses 10 warmups and 50 timed queries. The focused telemetry
rerun uses `bench_session_gucs=["ec_distann.scan_profile_notice=on"]` and
captures per-round NOTICE records for requested/expanded nodes, transport wait,
straggler spread, and request/response bytes. Aggregate stage counters and the
owner-oracle control are isolated in `task206-owner-control.json`, which uses
the benchmark-feature extension.
