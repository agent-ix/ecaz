# SPIRE Phase 9 Quality Baseline Artifact Manifest

Head SHA: `974ad3b17c6bcd6bbfd98a5386b8668263bd3107`
Packet/topic: `30686-spire-phase9-quality-baseline`
Timestamp: `2026-05-09T15:17:50-07:00`

This packet records the canonical local pre-Phase-9.7 baseline on the main
machine. It is local development evidence only, not an AWS/RDS-class product
scale claim.

## Environment

- Host class: local main development machine
- PostgreSQL: `18.3`, pgrx socket `/home/peter/.pgrx`, port `28818`
- Database: `tqvector_bench`
- Extension install command:
  `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18`
- Fixture availability: `target/real-corpus/ec_hnsw_real_10k` only;
  `target/real-corpus` did not contain real50k or real100k fixture directories.
- Fixture: `ec_hnsw_real_10k`, 10,000 corpus rows, 200 query rows, 1536
  dimensions
- Query subset for recall/latency: first 100 query rows, `k=10`, seed `42`
- Profile / AM: `ec_spire`
- Storage format: `turboquant`
- Rerank modes swept: `rerank_width=0`, `25`, `50`
- Nprobe sweep: `8`, `16`, `24`, `32`
- Reloptions: `nlists=32`, `nprobe=24`, `rerank_width=25`,
  `recursive_fanout=2`, `nprobe_per_level=2`
- Isolated one-index-per-table: yes, prefix
  `task30_p9_quality_base_c5ed545`

## Artifacts

| Artifact | Lane | Fixture | Storage format | Rerank mode | Command | Timestamp | Isolated one-index-per-table | Key result lines |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `local-pg18-database-check.log` | setup | local PG18 | n/a | n/a | `target/debug/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --sql "select datname from pg_database where datname = 'tqvector_bench'" --log-output review/30686-spire-phase9-quality-baseline/artifacts/local-pg18-database-check.log` | `2026-05-09T15:17:50-07:00` | n/a | `tqvector_bench` exists. |
| `local-prefix-check.log` | setup | real10k | n/a | n/a | `target/debug/ecaz dev sql --pg 18 --db tqvector_bench --socket-dir /home/peter/.pgrx --raw --sql "select to_regclass('task30_p9_quality_base_c5ed545_corpus') as corpus, to_regclass('task30_p9_quality_base_c5ed545_idx') as idx" --log-output review/30686-spire-phase9-quality-baseline/artifacts/local-prefix-check.log` | `2026-05-09T15:17:50-07:00` | yes | Prefix was unused before load. |
| `load-real10k-recursive.log` | load | real10k | turboquant | relation `rerank_width=25` | `target/debug/ecaz corpus load --database tqvector_bench --host /home/peter/.pgrx --port 28818 --prefix task30_p9_quality_base_c5ed545 --profile ec_spire --corpus-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv --queries-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv --manifest-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_manifest.json --allow-manifest-mismatch --bits 4 --seed 42 --reloption storage_format=turboquant --reloption nlists=32 --reloption nprobe=24 --reloption rerank_width=25 --reloption recursive_fanout=2 --reloption nprobe_per_level=2 --log-file review/30686-spire-phase9-quality-baseline/artifacts/load-real10k-recursive.log` | `2026-05-09T15:17:50-07:00` | yes | Built `task30_p9_quality_base_c5ed545_idx` in 84.18s; completed prefix in 113.97s. |
| `storage-real10k-recursive.log` | storage | real10k | turboquant | relation `rerank_width=25` | `target/debug/ecaz bench storage --database tqvector_bench --host /home/peter/.pgrx --port 28818 --prefix task30_p9_quality_base_c5ed545 --log-file review/30686-spire-phase9-quality-baseline/artifacts/storage-real10k-recursive.log` | `2026-05-09T15:17:50-07:00` | yes | Total table size 168.0 MiB; SPIRE index 8.2 MiB, 859.3 B/row. |
| `explain-real10k-nprobe-rerank-matrix.sql` / `explain-real10k-nprobe-rerank-matrix.log` | explain/planner | real10k | turboquant | `0`, `25`, `50` | `target/debug/ecaz dev sql --pg 18 --db tqvector_bench --socket-dir /home/peter/.pgrx --raw --file review/30686-spire-phase9-quality-baseline/artifacts/explain-real10k-nprobe-rerank-matrix.sql --log-output review/30686-spire-phase9-quality-baseline/artifacts/explain-real10k-nprobe-rerank-matrix.log` | `2026-05-09T15:17:50-07:00` | yes | Index size 8,593,408 bytes. Execution time matrix: rw0 649.738/1093.483/1532.316/1872.325 ms; rw25 67.419/113.707/146.152/183.033 ms; rw50 70.537/115.800/151.689/190.904 ms for nprobe 8/16/24/32. |
| `latency-real10k-nprobe-8-16-24-32-rw0-cli.log` / `latency-real10k-nprobe-8-16-24-32-rw0-table.log` | latency | real10k | turboquant | `rerank_width=0` | `target/debug/ecaz bench latency --database tqvector_bench --host /home/peter/.pgrx --port 28818 --prefix task30_p9_quality_base_c5ed545 --profile ec_spire --k 10 --concurrency 1 --iterations 100 --sweep 8,16,24,32 --rerank-width 0 --bits 4 --seed 42 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-file review/30686-spire-phase9-quality-baseline/artifacts/latency-real10k-nprobe-8-16-24-32-rw0-cli.log --log-output review/30686-spire-phase9-quality-baseline/artifacts/latency-real10k-nprobe-8-16-24-32-rw0-table.log` | `2026-05-09T15:17:50-07:00` | yes | p50/p95/p99 ms: nprobe8 576.3/616.3/629.9; nprobe16 1019.7/1076.2/1127.0; nprobe24 1438.7/1530.5/1540.2; nprobe32 1896.5/1932.0/1954.4. |
| `latency-real10k-nprobe-8-16-24-32-rw25-cli.log` / `latency-real10k-nprobe-8-16-24-32-rw25-table.log` | latency | real10k | turboquant | `rerank_width=25` | `target/debug/ecaz bench latency --database tqvector_bench --host /home/peter/.pgrx --port 28818 --prefix task30_p9_quality_base_c5ed545 --profile ec_spire --k 10 --concurrency 1 --iterations 100 --sweep 8,16,24,32 --rerank-width 25 --bits 4 --seed 42 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-file review/30686-spire-phase9-quality-baseline/artifacts/latency-real10k-nprobe-8-16-24-32-rw25-cli.log --log-output review/30686-spire-phase9-quality-baseline/artifacts/latency-real10k-nprobe-8-16-24-32-rw25-table.log` | `2026-05-09T15:17:50-07:00` | yes | p50/p95/p99 ms: nprobe8 73.9/101.6/113.7; nprobe16 112.0/125.2/145.4; nprobe24 150.6/160.4/168.5; nprobe32 188.1/197.6/231.5. |
| `latency-real10k-nprobe-8-16-24-32-rw50-cli.log` / `latency-real10k-nprobe-8-16-24-32-rw50-table.log` | latency | real10k | turboquant | `rerank_width=50` | `target/debug/ecaz bench latency --database tqvector_bench --host /home/peter/.pgrx --port 28818 --prefix task30_p9_quality_base_c5ed545 --profile ec_spire --k 10 --concurrency 1 --iterations 100 --sweep 8,16,24,32 --rerank-width 50 --bits 4 --seed 42 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-file review/30686-spire-phase9-quality-baseline/artifacts/latency-real10k-nprobe-8-16-24-32-rw50-cli.log --log-output review/30686-spire-phase9-quality-baseline/artifacts/latency-real10k-nprobe-8-16-24-32-rw50-table.log` | `2026-05-09T15:17:50-07:00` | yes | p50/p95/p99 ms: nprobe8 78.0/89.1/100.9; nprobe16 117.1/123.5/130.0; nprobe24 154.7/168.3/179.4; nprobe32 192.7/224.7/255.2. |
| `truth-real10k-k10-queries100.json` | recall truth | real10k | n/a | n/a | Produced by first recall command with `--truth-cache-file review/30686-spire-phase9-quality-baseline/artifacts/truth-real10k-k10-queries100.json`. | `2026-05-09T15:17:50-07:00` | yes | Exact truth for 100 query rows at `k=10`. |
| `recall-real10k-nprobe-8-16-24-32-rw0-cli.log` / `recall-real10k-nprobe-8-16-24-32-rw0-table.log` | recall | real10k | turboquant | `rerank_width=0` | `target/debug/ecaz bench recall --database tqvector_bench --host /home/peter/.pgrx --port 28818 --prefix task30_p9_quality_base_c5ed545 --profile ec_spire --k 10 --sweep 8,16,24,32 --rerank-width 0 --queries-limit 100 --bits 4 --seed 42 --force-index --truth-cache-file review/30686-spire-phase9-quality-baseline/artifacts/truth-real10k-k10-queries100.json --log-file review/30686-spire-phase9-quality-baseline/artifacts/recall-real10k-nprobe-8-16-24-32-rw0-cli.log --log-output review/30686-spire-phase9-quality-baseline/artifacts/recall-real10k-nprobe-8-16-24-32-rw0-table.log` | `2026-05-09T15:17:50-07:00` | yes | recall@10: 0.9950/1.0000/1.0000/1.0000; NDCG@10: 0.9998/1.0000/1.0000/1.0000; mean q-time 566.49/1020.17/1446.22/1892.03 ms for nprobe 8/16/24/32. |
| `recall-real10k-nprobe-8-16-24-32-rw25-cli.log` / `recall-real10k-nprobe-8-16-24-32-rw25-table.log` | recall | real10k | turboquant | `rerank_width=25` | `target/debug/ecaz bench recall --database tqvector_bench --host /home/peter/.pgrx --port 28818 --prefix task30_p9_quality_base_c5ed545 --profile ec_spire --k 10 --sweep 8,16,24,32 --rerank-width 25 --queries-limit 100 --bits 4 --seed 42 --force-index --truth-cache-file review/30686-spire-phase9-quality-baseline/artifacts/truth-real10k-k10-queries100.json --log-file review/30686-spire-phase9-quality-baseline/artifacts/recall-real10k-nprobe-8-16-24-32-rw25-cli.log --log-output review/30686-spire-phase9-quality-baseline/artifacts/recall-real10k-nprobe-8-16-24-32-rw25-table.log` | `2026-05-09T15:17:50-07:00` | yes | recall@10: 0.9950/1.0000/1.0000/1.0000; NDCG@10: 0.9998/1.0000/1.0000/1.0000; mean q-time 72.65/114.10/149.72/191.50 ms. |
| `recall-real10k-nprobe-8-16-24-32-rw50-cli.log` / `recall-real10k-nprobe-8-16-24-32-rw50-table.log` | recall | real10k | turboquant | `rerank_width=50` | `target/debug/ecaz bench recall --database tqvector_bench --host /home/peter/.pgrx --port 28818 --prefix task30_p9_quality_base_c5ed545 --profile ec_spire --k 10 --sweep 8,16,24,32 --rerank-width 50 --queries-limit 100 --bits 4 --seed 42 --force-index --truth-cache-file review/30686-spire-phase9-quality-baseline/artifacts/truth-real10k-k10-queries100.json --log-file review/30686-spire-phase9-quality-baseline/artifacts/recall-real10k-nprobe-8-16-24-32-rw50-cli.log --log-output review/30686-spire-phase9-quality-baseline/artifacts/recall-real10k-nprobe-8-16-24-32-rw50-table.log` | `2026-05-09T15:17:50-07:00` | yes | recall@10: 0.9950/1.0000/1.0000/1.0000; NDCG@10: 0.9998/1.0000/1.0000/1.0000; mean q-time 77.57/118.04/155.20/193.18 ms. |

## Baseline Summary

Recall saturates on real10k by `nprobe=16` for every rerank mode measured.
At `nprobe=8`, all three rerank modes report recall@10 `0.9950`, leaving only
one miss across the 100-query subset.

| rerank_width | nprobe | recall@10 | NDCG@10 | latency p50 | latency p95 | latency p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 8 | 0.9950 | 0.9998 | 576.3 ms | 616.3 ms | 629.9 ms |
| 0 | 16 | 1.0000 | 1.0000 | 1019.7 ms | 1076.2 ms | 1127.0 ms |
| 0 | 24 | 1.0000 | 1.0000 | 1438.7 ms | 1530.5 ms | 1540.2 ms |
| 0 | 32 | 1.0000 | 1.0000 | 1896.5 ms | 1932.0 ms | 1954.4 ms |
| 25 | 8 | 0.9950 | 0.9998 | 73.9 ms | 101.6 ms | 113.7 ms |
| 25 | 16 | 1.0000 | 1.0000 | 112.0 ms | 125.2 ms | 145.4 ms |
| 25 | 24 | 1.0000 | 1.0000 | 150.6 ms | 160.4 ms | 168.5 ms |
| 25 | 32 | 1.0000 | 1.0000 | 188.1 ms | 197.6 ms | 231.5 ms |
| 50 | 8 | 0.9950 | 0.9998 | 78.0 ms | 89.1 ms | 100.9 ms |
| 50 | 16 | 1.0000 | 1.0000 | 117.1 ms | 123.5 ms | 130.0 ms |
| 50 | 24 | 1.0000 | 1.0000 | 154.7 ms | 168.3 ms | 179.4 ms |
| 50 | 32 | 1.0000 | 1.0000 | 192.7 ms | 224.7 ms | 255.2 ms |
