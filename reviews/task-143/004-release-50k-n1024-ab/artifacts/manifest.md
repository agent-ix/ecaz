# Task 143 Packet 004 Artifact Manifest

- Head SHA: `a363c847b066e913775fdd825be6a9c90d9d9861`
- Branch: `task-143-spire-leaf-ranking-route-overfetch`
- Task bucket: `reviews/task-143/004-release-50k-n1024-ab`
- Slice: release 50k/n1024 A/B evidence for baseline routing, leaf-score-only routing, and route overfetch alpha sweep `{1.25, 1.5, 2.0}`.
- Lane / fixture / storage / rerank mode: local PG18 release backend; `data/staged-current/ec_real_50k_*`; `ec_spire`; `bits=4`; `nlists=1024`; `boundary_replica_count=0`; `storage_format=rabitq`; default rerank width.
- Isolated/shared surface: isolated local table/index prefix `t143_50k_n1024_ab`.
- Backend profile: `ecaz_build_profile() = release`; suite manifest records `build_profile: release` and node profile `coordinator:28818:release`.
- Config SHA256: `09a037f80a8fb6d9d14b2c5d78de1f79f113b77643f9b9c2db76e8ad2a7eb713`.

## Artifacts

| Artifact | Command | Timestamp | Key result |
| --- | --- | --- | --- |
| `artifacts/suite-task143-50k-n1024-ab.json` | checked-in `ecaz bench suite` config | 2026-07-05 07:04:43-07:00 | Runs precheck, load, storage, truth-cache recall, and five `spire-pipeline` variants. |
| `artifacts/suite-dry-run.log`, `artifacts/suite-dry-run-manifest.json` | `target/release/ecaz bench suite run --dry-run --config artifacts/suite-task143-50k-n1024-ab.json ...` | 2026-07-05 07:02-07:03-07:00 | Dry-run emitted the expected nine-step plan. |
| `artifacts/suite-run.log` | `target/release/ecaz bench suite run --config artifacts/suite-task143-50k-n1024-ab.json --database tqvector_bench_task143 --host /tmp --port 28818 ...` | 2026-07-05 07:04-07:26-07:00 | Suite completed and wrote `suite-results.jsonl`; all nine steps succeeded in `suite-manifest.json`. |
| `artifacts/precheck-host.log` | suite `raw` precheck: `LOAD 'ecaz'; SELECT ... ecaz_build_profile(), current_setting(...)` | 2026-07-05 07:04:43-07:00 | `ecaz_build_profile = release`, `leaf_score_only_routing = off`, `route_overfetch_multiplier = 1`. |
| `artifacts/load-50k-n1024-index.log` | suite `load-50k-n1024-index` | 2026-07-05 07:04-07:06-07:00 | Corpus 50k, query 1000; corpus SHA `56023baa...40133`; query SHA `95ac7992...a9fa3`; total load `77.92s`; index build `39.64s`. |
| `artifacts/storage-50k-n1024-index.log` | suite `storage-50k-n1024-index` | 2026-07-05 07:06-07:07-07:00 | `t143_50k_n1024_ab_idx` size `50.7 MiB`, `1062.8 B` per row; table total `846.4 MiB`. |
| `artifacts/truth-cache-50k-q200-k10.log` | suite `truth-cache-50k-q200-k10` | 2026-07-05 07:07-07:08-07:00 | Release recall at nprobe 96: distinct recall `0.9590`, CI95 `[0.9494, 0.9668]`, mean q-time `179.95 ms`. |
| `artifacts/pipeline-baseline.log` plus compact table below | suite `pipeline-baseline`; GUCs `leaf_score_only_routing=off`, `route_overfetch_multiplier=1.0` | 2026-07-05 07:08-07:12-07:00 | Distinct recall `0.7525/0.8365/0.8965/0.9390/0.9590`; p50 `22.116/36.226/65.688/128.159/187.015 ms`. |
| `artifacts/pipeline-leaf-only.log` plus compact table below | suite `pipeline-leaf-only`; GUCs `leaf_score_only_routing=on`, `route_overfetch_multiplier=1.0` | 2026-07-05 07:12-07:15-07:00 | Distinct recall `0.7590/0.8490/0.9105/0.9475/0.9595`; p50 `22.710/37.568/66.356/122.661/182.717 ms`. |
| `artifacts/pipeline-overfetch-1_25.log` plus compact table below | suite `pipeline-overfetch-1_25`; GUCs `leaf_score_only_routing=off`, `route_overfetch_multiplier=1.25` | 2026-07-05 07:15-07:19-07:00 | Distinct recall `0.7575/0.8450/0.9070/0.9440/0.9590`; p50 `22.528/37.118/67.732/126.423/183.245 ms`. |
| `artifacts/pipeline-overfetch-1_5.log` plus compact table below | suite `pipeline-overfetch-1_5`; GUCs `leaf_score_only_routing=off`, `route_overfetch_multiplier=1.5` | 2026-07-05 07:19-07:22-07:00 | Distinct recall `0.7585/0.8480/0.9090/0.9475/0.9600`; p50 `22.546/37.197/65.906/123.916/184.283 ms`. |
| `artifacts/pipeline-overfetch-2_0.log` plus compact table below | suite `pipeline-overfetch-2_0`; GUCs `leaf_score_only_routing=off`, `route_overfetch_multiplier=2.0` | 2026-07-05 07:22-07:26-07:00 | Distinct recall `0.7585/0.8490/0.9110/0.9485/0.9605`; p50 `23.370/38.493/67.585/124.577/183.365 ms`. |
| `artifacts/suite-manifest.json`, `artifacts/suite-results.jsonl` | emitted by suite run | 2026-07-05 07:26-07:00 | Structured source of truth for commands, statuses, backend profile, storage, recall, pipeline metrics, and artifact paths. |

The suite generated `artifacts/truth-cache-50k-q200-k10.json`, but it is intentionally not committed because review truth-cache JSON is gitignored as regenerable cache data. The suite also generated raw per-query `pipeline-*-funnel.jsonl` and `pipeline-*-stage-containment.jsonl` diagnostics; those are intentionally not committed because `.gitignore` treats them as large regenerable pipeline diagnostics. The compact route-containment table below is derived from the `topology_route_set` stage rows.

## Step Status

| Step | Kind | Status | Duration |
| --- | --- | --- | ---: |
| precheck-host | raw | succeeded | 0.012s |
| load-50k-n1024-index | load | succeeded | 77.924s |
| storage-50k-n1024-index | storage | succeeded | 0.053s |
| truth-cache-50k-q200-k10 | recall | succeeded | 39.757s |
| pipeline-baseline | spire-pipeline | succeeded | 218.158s |
| pipeline-leaf-only | spire-pipeline | succeeded | 215.353s |
| pipeline-overfetch-1_25 | spire-pipeline | succeeded | 219.212s |
| pipeline-overfetch-1_5 | spire-pipeline | succeeded | 214.345s |
| pipeline-overfetch-2_0 | spire-pipeline | succeeded | 220.725s |

## 50k/n1024 A/B Summary

| Variant | nprobe | distinct recall@10 | p50 | p95 | route containment |
| --- | ---: | ---: | ---: | ---: | ---: |
| baseline | 8 | 0.7525 | 22.116 ms | 27.083 ms | 1505/2000 |
| baseline | 16 | 0.8365 | 36.226 ms | 43.880 ms | 1673/2000 |
| baseline | 32 | 0.8965 | 65.688 ms | 75.619 ms | 1793/2000 |
| baseline | 64 | 0.9390 | 128.159 ms | 147.240 ms | 1878/2000 |
| baseline | 96 | 0.9590 | 187.015 ms | 211.835 ms | 1918/2000 |
| leaf-only | 8 | 0.7590 | 22.710 ms | 28.753 ms | 1518/2000 |
| leaf-only | 16 | 0.8490 | 37.568 ms | 44.622 ms | 1698/2000 |
| leaf-only | 32 | 0.9105 | 66.356 ms | 76.189 ms | 1821/2000 |
| leaf-only | 64 | 0.9475 | 122.661 ms | 138.060 ms | 1895/2000 |
| leaf-only | 96 | 0.9595 | 182.717 ms | 201.087 ms | 1919/2000 |
| overfetch-1.25 | 8 | 0.7575 | 22.528 ms | 27.747 ms | 1515/2000 |
| overfetch-1.25 | 16 | 0.8450 | 37.118 ms | 45.013 ms | 1690/2000 |
| overfetch-1.25 | 32 | 0.9070 | 67.732 ms | 77.073 ms | 1814/2000 |
| overfetch-1.25 | 64 | 0.9440 | 126.423 ms | 141.631 ms | 1888/2000 |
| overfetch-1.25 | 96 | 0.9590 | 183.245 ms | 202.207 ms | 1918/2000 |
| overfetch-1.5 | 8 | 0.7585 | 22.546 ms | 27.642 ms | 1517/2000 |
| overfetch-1.5 | 16 | 0.8480 | 37.197 ms | 43.410 ms | 1696/2000 |
| overfetch-1.5 | 32 | 0.9090 | 65.906 ms | 75.642 ms | 1818/2000 |
| overfetch-1.5 | 64 | 0.9475 | 123.916 ms | 139.207 ms | 1895/2000 |
| overfetch-1.5 | 96 | 0.9600 | 184.283 ms | 202.204 ms | 1920/2000 |
| overfetch-2.0 | 8 | 0.7585 | 23.370 ms | 28.653 ms | 1517/2000 |
| overfetch-2.0 | 16 | 0.8490 | 38.493 ms | 45.582 ms | 1698/2000 |
| overfetch-2.0 | 32 | 0.9110 | 67.585 ms | 77.377 ms | 1822/2000 |
| overfetch-2.0 | 64 | 0.9485 | 124.577 ms | 138.141 ms | 1897/2000 |
| overfetch-2.0 | 96 | 0.9605 | 183.365 ms | 200.178 ms | 1921/2000 |

## Notes

- Route containment matches final distinct recall in every row, so this packet continues to localize the loss to route/leaf selection at 50k/n1024.
- Leaf-only improves baseline recall at every nprobe and is faster at nprobe 64 and 96.
- Overfetch improves baseline recall monotonically with alpha at nprobe 32/64/96, but alpha 2.0 adds latency at nprobe 8/16/32 and does not beat leaf-only by enough to justify promotion from this slice alone.
- This is not Task 143 closeout. The task still requires release A/B evidence at 100k and a final promote/iterate/negative decision packet.
