# Task 143 Packet 003 Artifact Manifest

- Head SHA: `368907103c68ef9e91118678d7c6755df6bc8500`
- Branch: `task-143-spire-leaf-ranking-route-overfetch`
- Task bucket: `reviews/task-143/003-release-10k-ab`
- Slice: release 10k A/B evidence for baseline routing, leaf-score-only routing, and route overfetch `alpha=1.5`.
- Lane / fixture / storage / rerank mode: local PG18 release backend; `data/staged-current/ec_real_10k_*`; `ec_spire`; `bits=4`; `nlists=128`; `boundary_replica_count=0`; `storage_format=rabitq`; default rerank width.
- Isolated/shared surface: isolated local table/index prefix `t143_10k_ab`.
- Backend profile: `ecaz_build_profile() = release`; suite manifest records `build_profile: release` and node profile `coordinator:28818:release`.

## Artifacts

| Artifact | Command | Timestamp | Key result |
| --- | --- | --- | --- |
| `artifacts/suite-task143-10k-ab.json` | checked-in `ecaz bench suite` config | 2026-07-05 06:47:09-07:00 | Runs precheck, load, storage, truth-cache recall, and three `spire-pipeline` variants. |
| `artifacts/suite-dry-run.log`, `artifacts/suite-dry-run-manifest.json` | `target/release/ecaz bench suite run --dry-run --config artifacts/suite-task143-10k-ab.json ...` | 2026-07-05 06:40:xx-07:00 | Dry-run emitted expected seven-step plan. |
| `artifacts/suite-audit.log` | `target/release/ecaz bench suite audit --config artifacts/suite-task143-10k-ab.json ...` | 2026-07-05 06:41:xx-07:00 | `audit passed: 7 steps`. |
| `artifacts/suite-run.log` | `target/release/ecaz bench suite run --config artifacts/suite-task143-10k-ab.json --database tqvector_bench_task143 --host /tmp --port 28818 ...` | 2026-07-05 06:47:09-07:00 | Suite completed and wrote `suite-results.jsonl`; all seven steps succeeded in `suite-manifest.json`. |
| `artifacts/precheck-host.log` | suite `raw` precheck: `LOAD 'ecaz'; SELECT ... ecaz_build_profile(), current_setting(...)` | 2026-07-05 06:47:09-07:00 | `ecaz_build_profile = release`, `leaf_score_only_routing = off`, `route_overfetch_multiplier = 1`. |
| `artifacts/load-10k-baseline-index.log` | suite `load-10k-baseline-index` | 2026-07-05 06:47:xx-07:00 | Build index completed; total load `9.400000` seconds; index build `1.760000` seconds. |
| `artifacts/storage-10k-baseline-index.log` | suite `storage-10k-baseline-index` | 2026-07-05 06:47:xx-07:00 | `t143_10k_ab_idx` size `9.4 MiB`, `980.6 B` per row; total `168.6 MiB`. |
| `artifacts/truth-cache-10k-q200-k10.log` | suite `truth-cache-10k-q200-k10` | 2026-07-05 06:47:xx-07:00 | Release recall at nprobe 96: distinct recall `1.0000`, CI95 `[0.9981, 1.0000]`, mean q-time `257.70 ms`. |
| `artifacts/pipeline-baseline.log` plus compact table below | suite `pipeline-baseline`; GUCs `leaf_score_only_routing=off`, `route_overfetch_multiplier=1.0` | 2026-07-05 06:48-06:50-07:00 | Distinct recall `0.9905/0.9940/0.9965/0.9995/1.0000` at nprobe `8/16/32/64/96`; p50 `24.850/47.922/92.721/174.386/264.676 ms`. |
| `artifacts/pipeline-leaf-only.log` plus compact table below | suite `pipeline-leaf-only`; GUCs `leaf_score_only_routing=on`, `route_overfetch_multiplier=1.0` | 2026-07-05 06:50-06:53-07:00 | Distinct recall `0.9935/0.9970/1.0000/1.0000/1.0000`; p50 `23.951/45.032/89.706/171.849/259.680 ms`. |
| `artifacts/pipeline-overfetch-1_5.log` plus compact table below | suite `pipeline-overfetch-1_5`; GUCs `leaf_score_only_routing=off`, `route_overfetch_multiplier=1.5` | 2026-07-05 06:53-06:56-07:00 | Distinct recall `0.9920/0.9965/0.9985/1.0000/1.0000`; p50 `24.888/45.426/89.657/171.751/270.816 ms`. |
| `artifacts/suite-manifest.json`, `artifacts/suite-results.jsonl` | emitted by suite run | 2026-07-05 06:56:xx-07:00 | Structured source of truth for commands, statuses, backend profile, storage, recall, pipeline metrics, and artifact paths. |

The suite generated `artifacts/truth-cache-10k-q200-k10.json`, but it is intentionally not committed because review truth-cache JSON is gitignored as regenerable cache data. The suite also generated raw per-query `pipeline-*-funnel.jsonl` and `pipeline-*-stage-containment.jsonl` diagnostics; those are intentionally not committed because `.gitignore` treats them as large regenerable pipeline diagnostics. The compact route-containment table below is derived from the `topology_route_set` stage rows.

## 10k A/B Summary

| Variant | nprobe | distinct recall@10 | p50 | p95 | route containment |
| --- | ---: | ---: | ---: | ---: | ---: |
| baseline | 8 | 0.9905 | 24.850 ms | 30.974 ms | 1981/2000 |
| baseline | 16 | 0.9940 | 47.922 ms | 54.288 ms | 1988/2000 |
| baseline | 32 | 0.9965 | 92.721 ms | 98.947 ms | 1993/2000 |
| baseline | 64 | 0.9995 | 174.386 ms | 187.147 ms | 1999/2000 |
| baseline | 96 | 1.0000 | 264.676 ms | 284.417 ms | 2000/2000 |
| leaf-only | 8 | 0.9935 | 23.951 ms | 29.376 ms | 1987/2000 |
| leaf-only | 16 | 0.9970 | 45.032 ms | 51.803 ms | 1994/2000 |
| leaf-only | 32 | 1.0000 | 89.706 ms | 97.131 ms | 2000/2000 |
| leaf-only | 64 | 1.0000 | 171.849 ms | 182.783 ms | 2000/2000 |
| leaf-only | 96 | 1.0000 | 259.680 ms | 272.683 ms | 2000/2000 |
| overfetch-1.5 | 8 | 0.9920 | 24.888 ms | 29.992 ms | 1984/2000 |
| overfetch-1.5 | 16 | 0.9965 | 45.426 ms | 51.930 ms | 1993/2000 |
| overfetch-1.5 | 32 | 0.9985 | 89.657 ms | 97.291 ms | 1997/2000 |
| overfetch-1.5 | 64 | 1.0000 | 171.751 ms | 183.367 ms | 2000/2000 |
| overfetch-1.5 | 96 | 1.0000 | 270.816 ms | 286.550 ms | 2000/2000 |

## Notes

- Leaf-only is the strongest 10k result in this slice: it reaches perfect route containment / distinct recall at nprobe 32 and is faster than baseline across the tested ladder.
- Overfetch `alpha=1.5` improves baseline containment at nprobe 8, 16, and 32, but does not beat leaf-only at 10k and is slower than leaf-only at nprobe 96.
- This is not Task 143 closeout. The task still requires release A/B evidence at 50k and 100k, plus the remaining alpha sweep `{1.25, 2}` or a documented decision to stop expanding overfetch based on release evidence.
