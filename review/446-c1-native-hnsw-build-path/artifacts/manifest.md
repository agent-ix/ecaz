# Artifact Manifest: 446-c1-native-hnsw-build-path

- head_sha: `948d3c4b34f8498f885a7da540de8c68fe77b537`
- packet: `446-c1-native-hnsw-build-path`
- timestamp: `2026-04-19T12:38:47-07:00`

## Artifacts

### `code-graph-uniform-10k.log`

- lane: `10k uniform`
- fixture: `synthetic random_unit_vectors`
- storage_format: `TurboQuant / code graph`
- rerank_mode: `n/a`
- surface: `one-index-per-table oracle probe`
- command:
  `cargo test test_hnsw_rs_code_graph_recall_uniform_10k --no-default-features --features pg17 -- --ignored --nocapture`
- key_result_lines:
  - `hnsw-rs code graph timings: m=8 ef_search=128 build=228.746164235s search=39.286430847s`
  - `hnsw-rs code graph probe: queries=20 m=8 ef_search=128 hnsw=0.2900 build_code=0.8050 exact=0.8400`

### `source-graph-uniform-10k.log`

- lane: `10k uniform`
- fixture: `synthetic random_unit_vectors`
- storage_format: `source graph`
- rerank_mode: `n/a`
- surface: `one-index-per-table oracle probe`
- command:
  `cargo test test_hnsw_rs_source_graph_recall_uniform_10k --no-default-features --features pg17 -- --ignored --nocapture`
- key_result_lines:
  - `hnsw-rs source graph timings: m=8 ef_search=128 build=523.082011194s search=5.303376254s`
  - `hnsw-rs source graph probe: queries=20 m=8 ef_search=128 hnsw=0.3000`

### `source-graph-clustered-10k.log`

- lane: `10k clustered`
- fixture: `synthetic random_clustered_vectors`
- storage_format: `source graph`
- rerank_mode: `n/a`
- surface: `one-index-per-table oracle probe`
- command:
  `cargo test test_hnsw_rs_source_graph_recall_clustered_10k --no-default-features --features pg17 -- --ignored --nocapture`
- key_result_lines:
  - `hnsw-rs source graph timings: m=8 ef_search=128 build=416.330819783s search=5.309683839s`
  - `hnsw-rs source graph clustered probe: queries=20 m=8 ef_search=128 hnsw=0.2850`

### `source-graph-uniform-10k-m16-ef200.log`

- lane: `10k uniform`
- fixture: `synthetic random_unit_vectors`
- storage_format: `source graph`
- rerank_mode: `n/a`
- surface: `one-index-per-table oracle probe`
- command:
  `cargo test test_hnsw_rs_source_graph_recall_uniform_10k --no-default-features --features pg17 -- --ignored --nocapture`
- key_result_lines:
  - `hnsw-rs source graph timings: m=16 ef_search=200 build=862.663973048s search=6.605143507s`
  - `hnsw-rs source graph probe: queries=20 m=16 ef_search=200 hnsw=0.6550`

### `validation.log`

- lane: `repo validation`
- fixture: `full repo test/clippy gates`
- storage_format: `mixed`
- rerank_mode: `mixed`
- surface: `repo-wide validation`
- commands:
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- key_result_lines:
  - `cargo test`: `test result: ok. 505 passed; 0 failed; 7 ignored`
  - `scripts/run_pgrx_pg17_test.sh`: `test result: ok. 505 passed; 0 failed; 7 ignored`
  - `cargo clippy`: `Finished 'dev' profile [unoptimized + debuginfo] target(s) in 10.24s`
