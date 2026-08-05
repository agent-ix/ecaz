# Artifact manifest

- code head: `a6289dddf`
- task bucket / packet: `reviews/task-207/004-search-and-sharding`
- lane: PG18, three-owner physical distann, fixed `build_shards=4`
- A/B: `stitched_bfs` control versus `partition_union` candidate; all other build/search settings fixed
- search path: persisted Vamana head (`persisted_head`, width 128, k_head 128); no production default change
- owner evidence: `owner_scan` variants report exact owner membership and bounded/exact overlap at k
- NFR-021 preregistration: all arms are conforming physical multi-owner arms; bounded coordinator membership-only state is admissible control-plane state
- command: `/home/peter/.cargo-target/release/ecaz bench suite audit --config artifacts/task207-corrected.json`
- run command: `/home/peter/.cargo-target/release/ecaz bench suite run --config artifacts/task207-corrected.json --artifact-dir artifacts/run`
- extension install: `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18`
- CLI build: `cargo build --release --bin ecaz --package ecaz-cli --offline`
- timestamp: pending corrected run
- corpus/query inputs: external staged corpora; no corpus files committed
