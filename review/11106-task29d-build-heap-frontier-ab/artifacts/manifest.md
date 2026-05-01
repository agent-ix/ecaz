# Artifact Manifest

Packet: `11106-task29d-build-heap-frontier-ab`
Timestamp: `2026-04-30T21:39:12-07:00`

Branch head after recording the decision:
`d4cbcd86c24a9f73cb91e9da3688f588a20665e8`

Experimental code under measurement: current Task 29d branch with
`d2e0e9fc` and `36f0c3d5` applied to `src/am/ec_diskann/vamana.rs` without
committing or pushing. The patch replaced the linear "closest unvisited"
frontier scan in Vamana build greedy search with a
`BinaryHeap<Reverse<Candidate>>` next-node heap and kept the truncation fix from
`36f0c3d5`.

## Environment

- PostgreSQL: local pgrx PG18 scratch server, socket directory
  `/home/peter/.pgrx`, port `28818`.
- Database: `task29_diskann_baseline`.
- Corpus prefix/table: `task29c_phase_profile_corpus`.
- Storage format: `ec_diskann` `pq_fastscan` tuple format with binary sidecar.
- Measurement surface: isolated DROP+CREATE INDEX on the existing real-10k
  corpus table.
- Reloptions: `graph_degree=32`, `build_list_size=100`, `alpha=1.2`.
- Baseline comparator: packet `11104` active-mask release build,
  `total_ms=70678`, `core_graph_ms=67571`.

## Artifacts

### `install-heap-frontier-pg18-release.log`

- Lane / fixture: release PG18 install of the experimental heap-frontier
  working tree.
- Command:
  `script -q -e -c "cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18" review/11106-task29d-build-heap-frontier-ab/artifacts/install-heap-frontier-pg18-release.log`
- Key result: release build completed and installed `ecaz.so` plus
  `ecaz--0.1.1.sql` into the PG18 scratch install tree.

### `create-index-task29d-heap-frontier-release.log`

- Lane / fixture: release-mode build A/B, isolated one-index DROP+CREATE.
- Command:
  `target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11106-task29d-build-heap-frontier-ab/artifacts/create-index-task29d-heap-frontier-release.log --sql "DROP INDEX IF EXISTS task29c_phase_profile_idx; CREATE INDEX task29c_phase_profile_idx ON task29c_phase_profile_corpus USING ec_diskann (embedding ecvector_diskann_ip_ops) WITH (graph_degree=32, build_list_size=100, alpha=1.2);"`
- Key result lines:
  - pass 0: `elapsed_ms=21933`, `greedy_search_ms=11627`,
    `robust_prune_ms=3`, `backlink_ms=0`
  - pass 1: `elapsed_ms=49683`, `greedy_search_ms=18492`,
    `robust_prune_ms=5860`, `backlink_ms=9114`
  - complete: `build_persist_ms=73242`, `core_medoid_ms=1616`,
    `core_graph_ms=71617`, `flush_total_ms=73779`, `total_ms=75492`

### `reinstall-current-head-pg18-release.log`

- Lane / fixture: restore local PG18 scratch server to the non-experimental
  branch head after the A/B.
- Command:
  `script -q -e -c "cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18" review/11106-task29d-build-heap-frontier-ab/artifacts/reinstall-current-head-pg18-release.log`
- Key result: current branch release build completed and reinstalled `ecaz.so`
  plus `ecaz--0.1.1.sql`.
