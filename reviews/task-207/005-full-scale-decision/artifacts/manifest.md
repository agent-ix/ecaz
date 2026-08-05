# Artifact manifest

- code head for the release decision matrix: `a6289dddf7802097f81d46ab8956e417222f0263` (clean release source)
- task bucket / packet: `reviews/task-207/005-full-scale-decision`
- preregistration/config: `../004-search-and-sharding/artifacts/task207-corrected.json`
- lane: PG18, three-owner physical distann, fixed `build_shards=4`
- A/B: `stitched_bfs` control versus `partition_union` candidate; persisted head search width 128, seed count 128; top-k 200
- scales: 10k, 50k, 100k; recall plus storage and 50 warm-cache latency iterations with 10 warmups
- release command: `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-207/004-search-and-sharding/artifacts/task207-corrected.json --artifact-dir reviews/task-207/004-search-and-sharding/artifacts/run`
- audit command: `/home/peter/.cargo-target/release/ecaz bench suite audit --config reviews/task-207/004-search-and-sharding/artifacts/task207-corrected.json`
- extension install: `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18,distann-head-attribution-benchmark`
- release artifact provenance: every decision-arm summary reports unanimous three-node release SHA above
- timestamp: 2026-08-04, America/Los_Angeles
- fixture: isolated one physical index per table/arm; run directories were under `$ECAZ_CLUSTER_ROOT` and removed after capture
- corpus/query inputs: external staged `ec_real_10k`, `ec_real_50k`, `ec_real_100k`; corpus files and truth caches are not committed
- NFR-021: preregistered conforming before measurement; physical topology reports zero non-owned rows and zero orphans
- owner diagnostic: `../004-search-and-sharding/artifacts/task207-owner-control.json`, fixed build_shards=4, owner_scan, separate feature-enabled lane; 50k and 100k stitched/union membership rows are in `../004-search-and-sharding/artifacts/owner-run/`; the final union 100k recall row is `0.7893`, while its wrapper stopped before latency/storage follow-ons. Source `ab22db162` with the packet-artifact `-dirty` preflight suffix
