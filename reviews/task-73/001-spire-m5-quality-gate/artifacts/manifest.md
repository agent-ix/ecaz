# Task 73 M5 SPIRE Quality Gate Manifest

- head SHA: `81b7f7ea6c3902aa90c126e88d705f586811d174`
- task bucket: `reviews/task-73/001-spire-m5-quality-gate`
- timestamp: `2026-05-30T23:37:28Z`
- lane: `m5-local`
- host: M5 laptop, 64 GB RAM
- PostgreSQL: PG18 via socket `/Users/peter/.pgrx`, database `task73_spire_gate`
- runner: `ecaz bench suite`
- suite config: `reviews/task-73/001-spire-m5-quality-gate/artifacts/suite.json`
- suite config sha256: `d4fa31686908047ab3ee7cc738f98b32e298ceae94781531584d928bf2a5ab57`
- suite manifest: `reviews/task-73/001-spire-m5-quality-gate/artifacts/suite-manifest.json`
- suite results: `reviews/task-73/001-spire-m5-quality-gate/artifacts/results.jsonl`
- isolation: one prefix per surface, one index per table; no shared-table benchmark surface

## Setup Artifacts

| artifact | command / surface | key result |
| --- | --- | --- |
| `install-pg18.log` | `./target/debug/ecaz dev install ecaz-pg-test --pg 18` | Installed `ecaz` into PG18; dylib sha256 `a4f29f1de85d3a1fb1196e5f2238cf2a9fefc2f71b822226b59af054364b547e`. |
| `drop-database.log` | `./target/debug/ecaz dev sql --pg 18 --db postgres --sql "DROP DATABASE IF EXISTS task73_spire_gate"` | Fresh gate database setup. |
| `create-database.log` | `./target/debug/ecaz dev sql --pg 18 --db postgres --sql "CREATE DATABASE task73_spire_gate"` | Fresh gate database setup. |
| `precheck-host-and-inputs.log` | suite `precheck-host-and-inputs` raw PG18 SQL | Verified PG18, `ecaz` extension, and `ec_spire`/`ec_ivf` access methods. |

## Main SPIRE Artifacts

| artifact | fixture / storage / rerank | command / setting | key result lines |
| --- | --- | --- | --- |
| `load-10k-spire-default.log` | 10k DBPedia, `turboquant`, `rerank_width=25` | suite load `task73_spire_10k` | Load completed in `9.74s`; index `task73_spire_10k_idx`. |
| `pipeline-10k-reproduce-default.log` | 10k DBPedia, `turboquant`, `rerank_width=25` | suite `bench spire-pipeline`, `nprobe=16`, `top_graph_search_list_size=16`, `boundary_replica_count=0` | recall@10 `0.9995`; p50 `5.939 ms`, p95 `6.246 ms`, p99 `6.344 ms`. |
| `load-100k-spire-default.log` | 100k DBPedia, `turboquant`, `rerank_width=25` | suite load `task73_spire_100k` | Load/inspection completed in `96.87s`; index `task73_spire_100k_idx`. |
| `pipeline-100k-tg16-b0.log` | 100k DBPedia, `turboquant`, `rerank_width=25` | `top_graph_search_list_size=16`, `boundary_replica_count=0`, sweep `8,16` | nprobe 16: recall@10 `0.8525`; p50 `13.505 ms`, p95 `15.410 ms`, p99 `15.868 ms`. |
| `rebuild-100k-tg32-b0.log` | 100k DBPedia, `turboquant`, `boundary_replica_count=0` | rebuild with `top_graph_search_list_size=32` | Build total `2599 ms`; top graph `270 ms`. |
| `pipeline-100k-tg32-b0.log` | 100k DBPedia, `turboquant`, `rerank_width=25` | `top_graph_search_list_size=32`, sweep `8,16,32` | nprobe 32: recall@10 `0.9310`; p50 `27.115 ms`, p95 `29.740 ms`, p99 `30.339 ms`. |
| `rebuild-100k-tg64-b0.log` | 100k DBPedia, `turboquant`, `boundary_replica_count=0` | rebuild with `top_graph_search_list_size=64` | Build total `2661 ms`; top graph `268 ms`. |
| `pipeline-100k-tg64-b0.log` | 100k DBPedia, `turboquant`, `rerank_width=25` | `top_graph_search_list_size=64`, sweep `8,16,32,64` | nprobe 64: recall@10 `0.9825`; p50 `52.631 ms`, p95 `56.755 ms`, p99 `58.806 ms`. |
| `rebuild-100k-tg128-b0.log` | 100k DBPedia, `turboquant`, `boundary_replica_count=0` | rebuild with `top_graph_search_list_size=128` | Build total `2671 ms`; top graph `258 ms`. |
| `pipeline-100k-tg128-b0.log` | 100k DBPedia, `turboquant`, `rerank_width=25` | `top_graph_search_list_size=128`, sweep `8,16,32,64,96,128` | nprobe 96: recall@10 `0.9975`, p50 `75.790 ms`, p95 `79.387 ms`, p99 `82.456 ms`; nprobe 128: recall@10 `1.0000`, p50 `95.960 ms`, p95 `96.476 ms`, p99 `99.049 ms`. |
| `rebuild-100k-tg128-b1.log` | 100k DBPedia, `turboquant`, `boundary_replica_count=1` | rebuild with `top_graph_search_list_size=128` | Build total `22041 ms`; draft leaf rows `19173 ms`; top graph `505 ms`. |
| `pipeline-100k-tg128-b1.log` | 100k DBPedia, `turboquant`, `rerank_width=25` | `boundary_replica_count=1`, sweep `8,16,32,64,96,128` | nprobe 64: recall@10 `0.9940`, p50 `108.444 ms`, p95 `116.407 ms`, p99 `119.364 ms`; nprobe 128: recall@10 `1.0000`, p50 `219.524 ms`, p95 `248.196 ms`, p99 `260.280 ms`. |
| `rebuild-100k-tg128-b2.log` | 100k DBPedia, `turboquant`, `boundary_replica_count=2` | rebuild with `top_graph_search_list_size=128` | Build total `22688 ms`; draft leaf rows `19371 ms`; top graph `859 ms`. |
| `pipeline-100k-tg128-b2.log` | 100k DBPedia, `turboquant`, `rerank_width=25` | `boundary_replica_count=2`, sweep `8,16,32,64,96,128` | nprobe 64: recall@10 `0.9970`, p50 `167.272 ms`, p95 `180.893 ms`, p99 `184.764 ms`; nprobe 96: recall@10 `1.0000`, p50 `254.368 ms`, p95 `267.700 ms`, p99 `272.306 ms`. |

## IVF Control Artifacts

| artifact | fixture / storage / rerank | command / setting | key result lines |
| --- | --- | --- | --- |
| `load-100k-ivf-control.log` | 100k DBPedia, `pq_fastscan`, `rerank=heap_f32`, `rerank_width=500` | suite load `task73_ivf_100k_control`, `nlists=128`, `nprobe=96` | Load completed in `164.09s`; index build `2.20s`. |
| `truth-100k-ivf-control-k10.json` | 100k DBPedia truth cache | suite `bench recall` generated k=10 truth for 200 queries | Ground truth computed in `55.54s`. |
| `recall-100k-ivf-control.log` | 100k DBPedia, `pq_fastscan`, `rerank_width=500` | `bench recall`, sweep `48,64,80,96,128` | nprobe 80: recall@10 `0.9950`, mean `9.43 ms`; nprobe 96: recall@10 `0.9980`, mean `10.81 ms`; nprobe 128: recall@10 `1.0000`, mean `12.76 ms`. |
| `latency-100k-ivf-control.log` | 100k DBPedia, `pq_fastscan`, `rerank_width=500` | `bench latency`, sweep `48,64,80,96,128`, post-recall warm | nprobe 80: p50 `9.38 ms`, p95 `10.5 ms`, p99 `12.7 ms`; nprobe 96: p50 `10.6 ms`, p95 `11.9 ms`, p99 `14.0 ms`; nprobe 128: p50 `12.7 ms`, p95 `13.8 ms`, p99 `14.3 ms`. |

## Notes

- Cross-AM comparisons in this packet use recall@10 for both SPIRE and IVF. Older IVF recall@100 citations are not used as apples-to-apples evidence.
- `nprobe > top_graph_search_list_size` is rejected by SPIRE (`top graph search list size must be at least route count`), so the valid sweep only includes nprobe values at or below each top-graph search list size.
- SPIRE `include-production-read-profile` on `turboquant` reports local heap candidate totals but remote tuple timing remains not applicable; raw local pipeline counters are the durable overhead evidence for this packet.
