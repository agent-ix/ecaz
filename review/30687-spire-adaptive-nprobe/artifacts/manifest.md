# SPIRE Adaptive Nprobe Artifact Manifest

Head SHA: `31f6f30200116aee77c90ac2b2c0ce4cdf70d392`
Packet/topic: `30687-spire-adaptive-nprobe`
Timestamp: `2026-05-09T16:01:26-07:00`

This packet records a local Phase 9.7 adaptive-`nprobe` treatment on the same
main-machine real10k fixture used by
`review/30686-spire-phase9-quality-baseline`. It is local development evidence
only, not an AWS/RDS-class product-scale claim.

## Environment

- Host class: local main development machine
- PostgreSQL: `18.3`, pgrx socket `/home/peter/.pgrx`, port `28818`
- Database: `tqvector_bench`
- Fixture: `ec_hnsw_real_10k`, loaded under prefix
  `task30_p9_quality_base_c5ed545`
- Profile / AM: `ec_spire`
- Storage format: `turboquant`
- Query subset: first 100 query rows, `k=10`, seed `42`
- Isolated one-index-per-table: yes, inherited from baseline prefix
- Baseline packet: `review/30686-spire-phase9-quality-baseline`

## Code / Validation Artifacts

| Artifact | Command | Key result lines |
| --- | --- | --- |
| `cargo-test-lib-pg18-no-run.log` | `cargo test --no-default-features --features pg18 --lib --no-run` | Finished test-profile lib build. |
| `cargo-test-adaptive-nprobe-reduces.log` | `cargo test --no-default-features --features pg18 am::ec_spire::scan::tests::adaptive_nprobe_reduces_routing_width_when_boundary_gap_is_large --lib -- --exact` | `1 passed; 0 failed`; verifies deterministic reduction from `nprobe=4` to `2` when the boundary score gap clears the threshold. |
| `cargo-test-adaptive-nprobe-keeps.log` | `cargo test --no-default-features --features pg18 am::ec_spire::scan::tests::adaptive_nprobe_keeps_configured_width_when_boundary_gap_is_small --lib -- --exact` | `1 passed; 0 failed`; verifies configured width is kept when the boundary gap is below threshold. |
| `cargo-test-ecaz-cli-adaptive-nprobe.log` | `cargo test -p ecaz-cli adaptive_nprobe` | `2 passed; 0 failed`; validates SPIRE-only CLI flag gating and threshold validation. |
| `cargo-fmt-check.log` | `cargo fmt --check` | Exit 0 with the existing stable-rustfmt warnings about unstable import options. |
| `git-diff-check.log` | `git diff --check` | Exit 0. |
| `cargo-test-pg18-adaptive-nprobe.log` | `cargo test --no-default-features --features pg18 adaptive_nprobe --lib` | Packet-local rerun was sandbox-blocked during pgrx install: `/home/peter/.pgrx/.../ecaz.control` was read-only. The log still shows the two pure adaptive tests passed before install. |

## Diagnostic SQL Artifacts

| Artifact | Command | Key result lines |
| --- | --- | --- |
| `diagnostic-routing-snapshot-function-catalog.log` | `target/debug/ecaz dev sql --pg 18 --db tqvector_bench --socket-dir /home/peter/.pgrx --raw --sql "SELECT ... WHERE p.proname LIKE 'ec_spire%routing%snapshot%'"` | Existing benchmark DB exposes root/centroid routing snapshot functions but not the newly added `ec_spire_index_scan_routing_snapshot` SQL function. |
| `diagnostic-routing-snapshot-gap300000.log` | `target/debug/ecaz dev sql ... SELECT ... FROM ec_spire_index_scan_routing_snapshot(...)` | Failed because the existing benchmark DB was created before the new SQL function existed. Fresh-extension coverage is represented in the added `pg_test`, but the packet-local rerun was sandbox-blocked as noted above. |

## Recall / Latency Treatment Artifacts

All treatment commands used:

`target/debug/ecaz bench recall --database tqvector_bench --host /home/peter/.pgrx --port 28818 --prefix task30_p9_quality_base_c5ed545 --profile ec_spire --k 10 --sweep <nprobe> --rerank-width <rw> --adaptive-nprobe --adaptive-nprobe-score-gap-micros <gap> --queries-limit 100 --bits 4 --seed 42 --force-index --truth-cache-file review/30686-spire-phase9-quality-baseline/artifacts/truth-real10k-k10-queries100.json --log-output <artifact>`

`target/debug/ecaz bench latency --database tqvector_bench --host /home/peter/.pgrx --port 28818 --prefix task30_p9_quality_base_c5ed545 --profile ec_spire --k 10 --concurrency 1 --iterations 100 --sweep <nprobe> --rerank-width <rw> --adaptive-nprobe --adaptive-nprobe-score-gap-micros <gap> --bits 4 --seed 42 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output <artifact>`

Same-build controls omitted the adaptive flags.

| Artifact pair | Lane | Fixture | Storage format | Rerank mode | Adaptive gap | Key result lines |
| --- | --- | --- | --- | --- | ---: | --- |
| `recall-real10k-nprobe16-rw25-control-*` | recall control | real10k | turboquant | rw25 | off | recall@10 `1.0000`, NDCG@10 `1.0000`, mean `112.64 ms`. |
| `latency-real10k-nprobe16-rw25-control-*` | latency control | real10k | turboquant | rw25 | off | mean/p50/p95/p99 `113.7/112.9/120.9/123.9 ms`. |
| `recall-real10k-nprobe16-rw25-adaptive-gap1000-*` | recall tuning | real10k | turboquant | rw25 | 1000 | recall@10 `0.9220`, mean `76.76 ms`; too aggressive. |
| `recall-real10k-nprobe16-rw25-adaptive-gap100000-*` | recall tuning | real10k | turboquant | rw25 | 100000 | recall@10 `0.9760`, mean `108.09 ms`; too aggressive. |
| `recall-real10k-nprobe16-rw25-adaptive-gap125000-*` | recall tuning | real10k | turboquant | rw25 | 125000 | recall@10 `0.9890`, mean `109.28 ms`; too aggressive. |
| `recall-real10k-nprobe16-rw25-adaptive-gap140000-*` | recall tuning | real10k | turboquant | rw25 | 140000 | recall@10 `0.9900`, mean `110.72 ms`; too aggressive. |
| `recall-real10k-nprobe16-rw25-adaptive-gap150000-*` | recall treatment | real10k | turboquant | rw25 | 150000 | recall@10 `1.0000`, NDCG@10 `1.0000`, mean `109.76 ms`. |
| `latency-real10k-nprobe16-rw25-adaptive-gap150000-*` | latency treatment | real10k | turboquant | rw25 | 150000 | mean/p50/p95/p99 `110.7/113.7/119.8/126.0 ms`; lower mean, noisier p50/p99 versus same-build control. |
| `recall-real10k-nprobe16-rw25-adaptive-gap200000-*` | recall tuning | real10k | turboquant | rw25 | 200000 | recall@10 `1.0000`, mean `109.98 ms`. |
| `latency-real10k-nprobe16-rw25-adaptive-gap200000-*` | latency tuning | real10k | turboquant | rw25 | 200000 | mean/p50/p95/p99 `113.7/112.4/138.1/181.8 ms`; tail regressed. |
| `recall-real10k-nprobe16-rw25-adaptive-gap250000-*` | recall tuning | real10k | turboquant | rw25 | 250000 | recall@10 `1.0000`, mean `112.09 ms`. |
| `recall-real10k-nprobe16-rw25-adaptive-gap300000-*` | recall tuning | real10k | turboquant | rw25 | 300000 | recall@10 `1.0000`, mean `113.15 ms`. |
| `latency-real10k-nprobe16-rw25-adaptive-gap300000-*` | latency tuning | real10k | turboquant | rw25 | 300000 | first run mean/p50/p95/p99 `112.5/111.7/119.9/128.6 ms`; rerun `113.9/113.7/120.6/124.7 ms`. |
| `recall-real10k-nprobe16-rw50-control-*` | recall control | real10k | turboquant | rw50 | off | recall@10 `1.0000`, NDCG@10 `1.0000`, mean `118.66 ms`. |
| `latency-real10k-nprobe16-rw50-control-*` | latency control | real10k | turboquant | rw50 | off | mean/p50/p95/p99 `117.7/117.1/122.7/131.6 ms`. |
| `recall-real10k-nprobe16-rw50-adaptive-gap150000-*` | recall treatment | real10k | turboquant | rw50 | 150000 | recall@10 `1.0000`, NDCG@10 `1.0000`, mean `115.18 ms`. |
| `latency-real10k-nprobe16-rw50-adaptive-gap150000-*` | latency treatment | real10k | turboquant | rw50 | 150000 | mean/p50/p95/p99 `113.1/115.9/121.4/125.4 ms`; improves all reported latency stats versus same-build rw50 control. |
| `recall-real10k-nprobe24-rw25-adaptive-gap100000-*` | recall tuning | real10k | turboquant | rw25 | 100000 | recall@10 `0.9760`, mean `127.19 ms`; too aggressive. |
| `recall-real10k-nprobe24-rw25-adaptive-gap300000-*` | recall tuning | real10k | turboquant | rw25 | 300000 | recall@10 `1.0000`, mean `150.18 ms`; roughly baseline-neutral. |
| `recall-real10k-nprobe32-rw25-adaptive-gap300000-*` | recall tuning | real10k | turboquant | rw25 | 300000 | recall@10 `1.0000`, mean `188.05 ms`; roughly baseline-neutral. |

## Summary

The safe treatment point is `nprobe=16`, `rerank_width=50`,
`adaptive_nprobe_score_gap_micros=150000`. It preserves recall@10 and NDCG@10
at `1.0000` while reducing same-build latency:

| mode | recall@10 | NDCG@10 | mean q-time | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| control rw50 | 1.0000 | 1.0000 | 118.66 ms | 117.1 ms | 122.7 ms | 131.6 ms |
| adaptive rw50 gap150000 | 1.0000 | 1.0000 | 115.18 ms | 115.9 ms | 121.4 ms | 125.4 ms |

The rw25 treatment finds recall-safe thresholds but latency is noisy; this
packet treats rw50 gap150000 as the cited local treatment.
