# Artifact Manifest

Head SHA: `b0deb879dfd95eff0094929031015095da9473e2`

Packet: `review/11103-task29-landing-readiness-refresh`

Lane: Task 29 DiskANN initial tuning, local PG18 landing refresh after Task 29b
and Task 29c.

Fixture: local PG18, real-10k 1536-d corpus, 200 query rows. Packet-local logs
are copied from the source packets listed below.

Storage format: `ec_diskann` `pq_fastscan` tuple format with persisted binary
sidecar payload.

Rerank mode: heap-f32 exact rerank, existing reloption default
`rerank_budget=64`.

Table model: isolated one-index-per-table prefixes for real-10k benchmark
artifacts.

Cache state: warm local PG18 cache state as recorded by the source packets.

Timestamp: 2026-04-30T13:39:46-07:00

## Source Packets

- `review/11099-task29-diskann-landing-readiness/`
- `review/11100-task29b-diskann-vacuum-prefilter-consistency/`
- `review/11102-task29c-vamana-core-profile/`

## Artifacts

### `pg18-diskann-callback-smoke.log`

Source: copied from `review/11099-task29-diskann-landing-readiness`.

Key result lines:

- `running 19 tests`
- `test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 860 filtered out; finished in 53.45s`

### `recall-sidecar-early-stop-table.log`

Source: copied from `review/11099-task29-diskann-landing-readiness`.

Key result rows:

- L=64: recall@10 `0.9955`, NDCG `0.9997`, mean `50.36 ms`
- L=128: recall@10 `0.9960`, NDCG `0.9999`, mean `48.80 ms`
- L=200: recall@10 `0.9970`, NDCG `0.9999`, mean `53.15 ms`
- L=400: recall@10 `0.9970`, NDCG `0.9999`, mean `58.89 ms`
- L=800: recall@10 `0.9975`, NDCG `0.9999`, mean `68.90 ms`

### `latency-sidecar-early-stop-table.log`

Source: copied from `review/11099-task29-diskann-landing-readiness`.

Key result rows:

- L=64: mean `48.5 ms`, p50 `47.8 ms`, p95 `54.1 ms`, p99 `57.0 ms`, HWM `65024 KiB`
- L=128: mean `54.1 ms`, p50 `50.3 ms`, p95 `76.3 ms`, p99 `88.7 ms`, HWM `64544 KiB`
- L=200: mean `58.5 ms`, p50 `55.9 ms`, p95 `75.0 ms`, p99 `90.1 ms`, HWM `64544 KiB`
- L=400: mean `61.7 ms`, p50 `61.2 ms`, p95 `74.6 ms`, p99 `82.9 ms`, HWM `65268 KiB`
- L=800: mean `67.7 ms`, p50 `66.7 ms`, p95 `76.9 ms`, p99 `80.0 ms`, HWM `66640 KiB`

### `storage-diskann-sidecar-cli.log`

Source: copied from `review/11099-task29-diskann-landing-readiness` artifact
`storage-task29a-sidecar-fresh-cli.log`.

Key result row:

- DiskANN index size `4.7 MiB`, bytes per row `494.0 B`

### `recall-ec-hnsw-reference-table.log`

Source: copied from `review/11099-task29-diskann-landing-readiness`.

Key result row:

- ef=200: recall@10 `0.9700`, NDCG `0.9993`, mean `35.25 ms`

### `latency-ec-hnsw-reference-table.log`

Source: copied from `review/11099-task29-diskann-landing-readiness`.

Key result row:

- ef=200: mean `34.5 ms`, p50 `33.1 ms`, p95 `39.4 ms`, p99 `49.1 ms`, HWM `49028 KiB`

### `storage-ec-hnsw-reference-cli.log`

Source: copied from `review/11099-task29-diskann-landing-readiness`.

Key result row:

- HNSW index size `13.0 MiB`, bytes per row `1366.4 B`

### `recall-task29b-prevacuum-table.log`

Source: copied from `review/11100-task29b-diskann-vacuum-prefilter-consistency`.

Key result row:

- L=200: recall@10 `0.9970`, NDCG `0.9999`, mean `52.52 ms`

### `recall-task29b-postvacuum-table.log`

Source: copied from `review/11100-task29b-diskann-vacuum-prefilter-consistency`.

Key result row:

- L=200: recall@10 `0.9975`, NDCG `0.9999`, mean `52.33 ms`

### `hamming-xor-popcount-asm.log`

Source: copied from `review/11100-task29b-diskann-vacuum-prefilter-consistency`.

Key result: generated assembly includes AVX2 vector work and `popcntq`
instructions for the sidecar Hamming prefilter path.

### `create-index-task29c-vamana-core-profile-release-extension.log`

Source: copied from `review/11102-task29c-vamana-core-profile`.

Key result lines:

- release-installed extension total: `79.238s`
- `heap_scan_ms=1261`
- `training_ms=130`
- `payload_derivation_ms=293`
- `build_persist_ms=77485`
- `core_medoid_ms=1566`
- `core_graph_ms=75903`
- `core_persist_ms=14`
- `write_pages_ms=59`
- pass 0 elapsed: `21.539s`
- pass 1 elapsed: `54.363s`

### `load-task29c-hnsw-reference-release-extension.log`

Source: copied from `review/11102-task29c-vamana-core-profile`.

Key result lines:

- built `task29c_phase_profile_m32_idx` in `5.23s`
- completed prefix in `7.24s`

### `size-task29c-diskann-hnsw-release-extension.log`

Source: copied from `review/11102-task29c-vamana-core-profile`.

Key result rows:

- `task29c_phase_profile_idx`: `4824 kB`, `4939776` bytes
- `task29c_phase_profile_m32_idx`: `14 MB`, `15130624` bytes
