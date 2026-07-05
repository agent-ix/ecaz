# Task 143 Packet 005 Artifact Manifest

- Head SHA: `900946c97f7ebc4f0c906129efd5c43fdb7159cc`
- Branch: `task-143-spire-leaf-ranking-route-overfetch`
- Task bucket: `reviews/task-143/005-release-100k-n1024-ab`
- Slice: release 100k/n1024 A/B evidence for baseline routing, leaf-score-only routing, and route overfetch alpha sweep `{1.25, 1.5, 2.0}`.
- Lane / fixture / storage / rerank mode: local PG18 release backend; `data/staged-current/ec_real_100k_*`; `ec_spire`; `bits=4`; `nlists=1024`; `boundary_replica_count=0`; `storage_format=rabitq`; default rerank width.
- Isolated/shared surface: isolated local table/index prefix `t143_100k_n1024_ab`.
- Backend profile: `ecaz_build_profile() = release`; suite manifest records `build_profile: release` and node profile `coordinator:28818:release`.
- Config SHA256: `bca7732d4d522ff0da23644e9fe1e4fe926d6dd7a5341ae6b6979872ee4c395a`.

## Artifacts

| Artifact | Command | Timestamp | Key result |
| --- | --- | --- | --- |
| `artifacts/suite-task143-100k-n1024-ab.json` | checked-in `ecaz bench suite` config | 2026-07-05 07:29:45-07:00 | Runs precheck, load, storage, truth-cache recall, and five `spire-pipeline` variants. |
| `artifacts/suite-dry-run.log`, `artifacts/suite-dry-run-manifest.json` | `target/release/ecaz bench suite run --dry-run --config artifacts/suite-task143-100k-n1024-ab.json ...` | 2026-07-05 07:28-07:29-07:00 | Dry-run emitted the expected nine-step plan. |
| `artifacts/suite-run.log` | `target/release/ecaz bench suite run --config artifacts/suite-task143-100k-n1024-ab.json --database tqvector_bench_task143 --host /tmp --port 28818 ...` | 2026-07-05 07:29-07:58-07:00 | Suite completed and wrote `suite-results.jsonl`; all nine steps succeeded in `suite-manifest.json`. |
| `artifacts/precheck-host.log` | suite `raw` precheck: `LOAD 'ecaz'; SELECT ... ecaz_build_profile(), current_setting(...)` | 2026-07-05 07:29:45-07:00 | `ecaz_build_profile = release`, `leaf_score_only_routing = off`, `route_overfetch_multiplier = 1`. |
| `artifacts/load-100k-n1024-index.log` | suite `load-100k-n1024-index` | 2026-07-05 07:29-07:32-07:00 | Corpus 100k, query 1000; corpus SHA `07275cfd...23a95`; query SHA `a7cbec6f...1782`; total load `153.24s`; index build `74.78s`. |
| `artifacts/storage-100k-n1024-index.log` | suite `storage-100k-n1024-index` | 2026-07-05 07:32-07:33-07:00 | `t143_100k_n1024_ab_idx` size `89.9 MiB`, `942.8 B` per row; table total `1.6 GiB`. |
| `artifacts/truth-cache-100k-q200-k10.log` | suite `truth-cache-100k-q200-k10` | 2026-07-05 07:33-07:34-07:00 | Release recall at nprobe 96: distinct recall `0.9300`, CI95 `[0.9180, 0.9404]`, mean q-time `375.18 ms`. |
| `artifacts/pipeline-baseline.log` plus compact table below | suite `pipeline-baseline`; GUCs `leaf_score_only_routing=off`, `route_overfetch_multiplier=1.0` | 2026-07-05 07:34-07:39-07:00 | Distinct recall `0.6785/0.7810/0.8585/0.9120/0.9300`; p50 `33.084/64.675/123.136/241.140/371.433 ms`. |
| `artifacts/pipeline-leaf-only.log` plus compact table below | suite `pipeline-leaf-only`; GUCs `leaf_score_only_routing=on`, `route_overfetch_multiplier=1.0` | 2026-07-05 07:39-07:44-07:00 | Distinct recall `0.7155/0.8270/0.8895/0.9375/0.9570`; p50 `31.428/61.388/118.427/246.891/362.912 ms`. |
| `artifacts/pipeline-overfetch-1_25.log` plus compact table below | suite `pipeline-overfetch-1_25`; GUCs `leaf_score_only_routing=off`, `route_overfetch_multiplier=1.25` | 2026-07-05 07:44-07:49-07:00 | Distinct recall `0.7045/0.8045/0.8680/0.9195/0.9405`; p50 `33.066/61.674/119.511/241.390/378.804 ms`. |
| `artifacts/pipeline-overfetch-1_5.log` plus compact table below | suite `pipeline-overfetch-1_5`; GUCs `leaf_score_only_routing=off`, `route_overfetch_multiplier=1.5` | 2026-07-05 07:49-07:54-07:00 | Distinct recall `0.7125/0.8140/0.8750/0.9225/0.9465`; p50 `32.884/60.892/119.418/236.499/369.890 ms`. |
| `artifacts/pipeline-overfetch-2_0.log` plus compact table below | suite `pipeline-overfetch-2_0`; GUCs `leaf_score_only_routing=off`, `route_overfetch_multiplier=2.0` | 2026-07-05 07:54-07:58-07:00 | Distinct recall `0.7125/0.8160/0.8840/0.9315/0.9505`; p50 `33.141/61.480/117.992/239.017/365.220 ms`. |
| `artifacts/suite-manifest.json`, `artifacts/suite-results.jsonl` | emitted by suite run | 2026-07-05 07:58-07:00 | Structured source of truth for commands, statuses, backend profile, storage, recall, pipeline metrics, and artifact paths. |

The suite generated `artifacts/truth-cache-100k-q200-k10.json`, but it is intentionally not committed because review truth-cache JSON is gitignored as regenerable cache data. The suite also generated raw per-query `pipeline-*-funnel.jsonl` and `pipeline-*-stage-containment.jsonl` diagnostics; those are intentionally not committed because `.gitignore` treats them as large regenerable pipeline diagnostics. The compact route-containment table below is derived from the `topology_route_set` stage rows.

## Step Status

| Step | Kind | Status | Duration |
| --- | --- | --- | ---: |
| precheck-host | raw | succeeded | 0.012s |
| load-100k-n1024-index | load | succeeded | 153.244s |
| storage-100k-n1024-index | storage | succeeded | 0.058s |
| truth-cache-100k-q200-k10 | recall | succeeded | 82.624s |
| pipeline-baseline | spire-pipeline | succeeded | 304.889s |
| pipeline-leaf-only | spire-pipeline | succeeded | 301.200s |
| pipeline-overfetch-1_25 | spire-pipeline | succeeded | 307.890s |
| pipeline-overfetch-1_5 | spire-pipeline | succeeded | 302.754s |
| pipeline-overfetch-2_0 | spire-pipeline | succeeded | 301.210s |

## 100k/n1024 A/B Summary

| Variant | nprobe | distinct recall@10 | p50 | p95 | route containment |
| --- | ---: | ---: | ---: | ---: | ---: |
| baseline | 8 | 0.6785 | 33.084 ms | 43.826 ms | 1357/2000 |
| baseline | 16 | 0.7810 | 64.675 ms | 82.625 ms | 1562/2000 |
| baseline | 32 | 0.8585 | 123.136 ms | 143.556 ms | 1717/2000 |
| baseline | 64 | 0.9120 | 241.140 ms | 272.086 ms | 1824/2000 |
| baseline | 96 | 0.9300 | 371.433 ms | 406.925 ms | 1860/2000 |
| leaf-only | 8 | 0.7155 | 31.428 ms | 41.269 ms | 1431/2000 |
| leaf-only | 16 | 0.8270 | 61.388 ms | 75.454 ms | 1654/2000 |
| leaf-only | 32 | 0.8895 | 118.427 ms | 136.310 ms | 1779/2000 |
| leaf-only | 64 | 0.9375 | 246.891 ms | 279.359 ms | 1875/2000 |
| leaf-only | 96 | 0.9570 | 362.912 ms | 396.376 ms | 1914/2000 |
| overfetch-1.25 | 8 | 0.7045 | 33.066 ms | 43.379 ms | 1409/2000 |
| overfetch-1.25 | 16 | 0.8045 | 61.674 ms | 77.981 ms | 1609/2000 |
| overfetch-1.25 | 32 | 0.8680 | 119.511 ms | 139.707 ms | 1736/2000 |
| overfetch-1.25 | 64 | 0.9195 | 241.390 ms | 270.932 ms | 1839/2000 |
| overfetch-1.25 | 96 | 0.9405 | 378.804 ms | 419.612 ms | 1881/2000 |
| overfetch-1.5 | 8 | 0.7125 | 32.884 ms | 42.245 ms | 1425/2000 |
| overfetch-1.5 | 16 | 0.8140 | 60.892 ms | 74.106 ms | 1628/2000 |
| overfetch-1.5 | 32 | 0.8750 | 119.418 ms | 140.036 ms | 1750/2000 |
| overfetch-1.5 | 64 | 0.9225 | 236.499 ms | 268.234 ms | 1845/2000 |
| overfetch-1.5 | 96 | 0.9465 | 369.890 ms | 409.921 ms | 1893/2000 |
| overfetch-2.0 | 8 | 0.7125 | 33.141 ms | 43.693 ms | 1425/2000 |
| overfetch-2.0 | 16 | 0.8160 | 61.480 ms | 74.846 ms | 1632/2000 |
| overfetch-2.0 | 32 | 0.8840 | 117.992 ms | 137.425 ms | 1768/2000 |
| overfetch-2.0 | 64 | 0.9315 | 239.017 ms | 265.947 ms | 1863/2000 |
| overfetch-2.0 | 96 | 0.9505 | 365.220 ms | 399.664 ms | 1901/2000 |

## Notes

- Route containment matches final distinct recall in every row, so this packet continues to localize the loss to route/leaf selection at 100k/n1024.
- Leaf-only improves baseline recall at every nprobe by `+0.0370/+0.0460/+0.0310/+0.0255/+0.0270` and is faster at nprobe 8, 16, 32, and 96.
- Overfetch improves baseline recall monotonically with alpha at most probes, but no overfetch variant catches leaf-only recall at any tested nprobe in this 100k slice.
- This completes the Task 143 required 10k/50k/100k release A/B evidence matrix. A separate decision packet should promote/iterate/negative based on packets 003-005.
