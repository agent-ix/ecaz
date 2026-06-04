# Task 79 Packet 026 Artifact Manifest

- packet: `reviews/task-79/026-leaf-block-rank-diagnostic`
- task: `plan/tasks/79-spire-candidate-surface-reduction.md`
- code checkpoint: `e7a956cd4` (`Add SPIRE leaf block rank diagnostic`)
- timestamp: `2026-06-01T22:46:48-07:00`
- lane: local PG18, Intel local, RaBitQ primary/default
- fixture: existing `task79_spire_candidate_surface` database, `task79_surface_100k_corpus`, `task79_surface_100k_queries`
- surface: shared Task 79 100k fixture table with existing clustered block64 index `task79_surface_100k_idx`
- storage format: RaBitQ
- index shape: `nlists=128`, `recursive_fanout=8`, `boundary_replica_count=0`, top graph enabled with search list size 96
- query shape: 200 queries, `nprobe=96`, `rerank_width=25`, recall@10
- block selector: `leaf_block_rows=64`, global block cap 384, summary radius weight 0.0, sampled probing disabled
- truth corpus: `target/real-corpus/staged-task50/ec_real_100k_corpus.tsv`
- suite config: `../suite-leaf-block-rank-diagnostic.json`
- suite config SHA256: `e22bba2d6e108dddc662325e11c4c279ec245b431cb1f69381380ade55f21d3b`
- rank JSONL SHA256: `c1fa51ced5840d5e5a74238a3e8c8b530e717080265037ee7b135c6cdd844111`
- installed backend SHA256: `a1f0b106800c3e1a4cd24f548634c0dabfdbad1073ec4ca0f766d64136289417`

No AWS commands were used for this packet.

## Commands

- `script -q -c "cargo fmt --check" reviews/task-79/026-leaf-block-rank-diagnostic/artifacts/cargo-fmt-check-final.log`
- `script -q -c "cargo check -p ecaz --no-default-features --features pg18" reviews/task-79/026-leaf-block-rank-diagnostic/artifacts/cargo-check-pg18.log`
- `script -q -c "cargo check -p ecaz-cli" reviews/task-79/026-leaf-block-rank-diagnostic/artifacts/cargo-check-ecaz-cli.log`
- `script -q -c "cargo build -p ecaz-cli" reviews/task-79/026-leaf-block-rank-diagnostic/artifacts/cargo-build-ecaz-cli.log`
- `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/026-leaf-block-rank-diagnostic/artifacts/install-ecaz-pg18.log`
- `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/026-leaf-block-rank-diagnostic/artifacts/pg18-restart.log restart -m fast`
- `target/debug/ecaz dev sql --pg 18 --db task79_spire_candidate_surface --socket-dir /home/peter/.pgrx --raw --file reviews/task-79/026-leaf-block-rank-diagnostic/artifacts/register-leaf-block-rank-function.sql --log-output reviews/task-79/026-leaf-block-rank-diagnostic/artifacts/register-leaf-block-rank-function.log`
- `target/debug/ecaz bench suite audit --config reviews/task-79/026-leaf-block-rank-diagnostic/suite-leaf-block-rank-diagnostic.json`
- `target/debug/ecaz bench suite run --config reviews/task-79/026-leaf-block-rank-diagnostic/suite-leaf-block-rank-diagnostic.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818`

## Artifact Index

- `suite-leaf-block-rank-diagnostic.json`: checked-in suite config for the local diagnostic run.
- `artifacts/cargo-fmt-check-final.log`: rustfmt check log. The repo's stable-rustfmt warnings are expected; command exited 0.
- `artifacts/cargo-check-pg18.log`: backend PG18 check log; exited 0.
- `artifacts/cargo-check-ecaz-cli.log`: CLI check log; exited 0 with the existing `LoadedDistributedPlacementConfig.path` dead-code warning.
- `artifacts/cargo-build-ecaz-cli.log`: CLI build log.
- `artifacts/install-ecaz-pg18.log`: local PG18 extension install log; records backend SHA256.
- `artifacts/pg18-restart.log`: local PG18 restart after install.
- `artifacts/register-leaf-block-rank-function.sql`: packet-local SQL registration for the new diagnostic function in the existing test database.
- `artifacts/register-leaf-block-rank-function.log`: manual function registration log.
- `artifacts/register-leaf-block-rank-function-suite.log`: suite setup registration log.
- `artifacts/leaf-block-rank-smoke.log`: single-query smoke of the SQL-visible diagnostic.
- `artifacts/suite-manifest.json`: suite run manifest.
- `artifacts/results.jsonl`: structured suite results.
- `artifacts/pipeline-leaf-block-rank-100k-rabitq-global384-rw0.log`: pipeline log for the diagnostic run.
- `artifacts/leaf-block-rank-100k-rabitq-global384-rw0.jsonl`: per-exact-target block-rank JSONL, 2,000 records.
- `artifacts/leaf-block-rank-analysis.md`: packet-local analysis of the JSONL rank file.

## Key Results

Pipeline row:

| candidates | p50 | p95 | recall@10 | routed leaves |
| ---: | ---: | ---: | ---: | ---: |
| 4,764,181 | 43.218 ms | 52.380 ms | 0.9690 | 19,200 |

Block-rank diagnostic:

| status | count |
| --- | ---: |
| `block_ranked` | 1,995 |
| `not_found_in_routed_leaves` | 5 |

| cap | selected exact top-10 targets | missed |
| ---: | ---: | ---: |
| 384 | 1,938 | 62 |
| 416 | 1,944 | 56 |
| 512 | 1,965 | 35 |
| 768 | 1,986 | 14 |
| 1024 | 1,994 | 6 |

Rank distribution for routed/ranked exact top-10 targets:

| p50 | p90 | p95 | p97.5 | p99 | max |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 9 | 109 | 223 | 419 | 588 | 1099 |

## Interpretation

The 384-block cap recall loss is fully explained by block selection: 1,938 selected exact top-10 targets out of 2,000 equals 0.9690 recall. Only 5 exact targets are absent from the routed leaves; the remaining misses are routed but ranked too low by the current single-summary block score.

The same rank file needs about a 768-block cap to reach the recall gate, but packet 025 measured cap 768 at 9,525,502 candidates and p50 56.486 ms. That is outside Task 79's candidate and latency gates. The next local slice should improve block-score information content rather than widen the candidate surface.
