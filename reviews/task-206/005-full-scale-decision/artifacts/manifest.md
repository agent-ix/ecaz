# Artifact manifest

> Superseded note: packet `../006-re-review-corrections/` retracts the
> release k-head comparison because the benchmark seed controls were compiled
> out.

- code head for the release decision matrix: `a6289dddf7802097f81d46ab8956e417222f0263` (clean release source)
- task bucket / packet: `reviews/task-206/005-full-scale-decision`
- preregistration/config: `../004-corrected-closeout/artifacts/task206-corrected.json`
- lane: PG18, three-owner physical distann, fixed `build_shards=1`, BW64/H8
- A/B: persisted-head `head_seed_count=128` versus `200`; owner-traversal control is a separate diagnostic lane
- scales: 10k, 50k, 100k; recall plus storage and 50 warm-cache latency iterations with 10 warmups
- release command: `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-206/004-corrected-closeout/artifacts/task206-corrected.json --artifact-dir reviews/task-206/004-corrected-closeout/artifacts/run`
- audit command: `/home/peter/.cargo-target/release/ecaz bench suite audit --config reviews/task-206/004-corrected-closeout/artifacts/task206-corrected.json`
- extension install: `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18`
- release artifact provenance: every decision-arm summary reports unanimous three-node release SHA above
- timestamp: 2026-08-04, America/Los_Angeles
- fixture: isolated one physical index per table/arm; run directories were under `$ECAZ_CLUSTER_ROOT` and removed after capture
- corpus/query inputs: external staged `ec_real_10k`, `ec_real_50k`, `ec_real_100k`; corpus files and truth caches are not committed
- NFR-021: preregistered conforming; `evidence_complete=true`, `decision_eligible=true`, max normalized growth `1.094707`, threshold `2.0`, max non-owned/orphans/derived `0`
- telemetry: `task206-100k-round-telemetry` forwarded `ec_distann.scan_profile_notice=on`; no runtime `ec_distann_scan_round` records were emitted in the 100k rerun or 10k smoke, so per-round attribution is explicitly unavailable
- owner-control source: feature-enabled install at `ab22db162`; the completed 10k owner arm reports membership recall `0.9727`, mean/p50/p95/p99/max latency `411.00/400.70/504.20/521.50/527.10` ms, and traversal request/response bytes `14,667.28/26,713.76` per scan. Its suite-level NFR rollup remains `actual_admissibility=unavailable` because this diagnostic rerun selected only 10k; the release decision matrix is the NFR-021 closeout evidence. Preflight records the expected `-dirty` suffix caused by packet artifacts, and this diagnostic lane is not used for release latency claims
