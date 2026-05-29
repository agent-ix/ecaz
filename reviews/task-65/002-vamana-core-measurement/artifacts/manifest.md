# Task 65 Measurement Manifest

Head SHA: `8e860324c1fa2a009bab209502962375f0207642`

Task bucket: `reviews/task-65/002-vamana-core-measurement`

Lane: local PG18 DiskANN Vamana build performance, small corpus only.

Surface: isolated one-index-per-table prefixes unless noted.

## Code Under Measurement

- `351987249` - `Optimize DiskANN Vamana build core`
- `da2807c0e` - `Bound DiskANN greedy frontier with heaps`
- `4bd460081` - `Fix DiskANN build validation edges`
- `a8b0b8789` - `Fix DiskANN build visibility handling`
- `de2ef72e4` - `Trim DiskANN Vamana build hot path`
- `8e860324c` - `Add Vamana build dhat profiler`

## Artifacts

| Artifact | Lane | Fixture / Format | Command | Key result |
|---|---|---|---|---|
| `precheck-pg18.log` | PG18 precheck | `postgres`, socket `/Users/peter/.pgrx`, port `28818` | `/Users/peter/.cargo/bin/ecaz dev sql --pg 18 --db postgres --socket-dir /Users/peter/.pgrx --raw --sql "SELECT version();" --log-output reviews/task-65/002-vamana-core-measurement/artifacts/precheck-pg18.log` | PostgreSQL 18.3 Homebrew on aarch64. |
| `install-ecaz-pg-test-after-loader-fix.log` | install | PG18 `ecaz-pg-test` | `/Users/peter/.cargo/bin/ecaz dev install ecaz-pg-test --pg 18 --log-file reviews/task-65/002-vamana-core-measurement/artifacts/install-ecaz-pg-test-after-loader-fix.log` | Installed release backend after the loader/visibility fixes; installed dylib sha256 `fbe83817...`. |
| `install-ecaz-pg-test-after-hotpath-trim.log` | install | PG18 `ecaz-pg-test` | `/Users/peter/.cargo/bin/ecaz dev install ecaz-pg-test --pg 18 --log-file reviews/task-65/002-vamana-core-measurement/artifacts/install-ecaz-pg-test-after-hotpath-trim.log` | Installed release backend from current tree; installed dylib sha256 `b36dac9a...`. |
| `load-real10k-diskann-pq-fastscan-loaderfix-r32-l100-short.log` | performance + loaderfix | real10k, `pq_fastscan`, `graph_degree=32`, `build_list_size=100`, `alpha=1.2` | `cargo run -p ecaz-cli --bin ecaz -- --database postgres --host /Users/peter/.pgrx --port 28818 corpus load --prefix task65_lfix_r10k --profile ec_diskann --storage-format pq_fastscan --corpus-file fixtures/m5_diskann_real10k/m5_diskann_real10k_corpus.tsv --queries-file fixtures/m5_diskann_real10k/m5_diskann_real10k_queries.tsv --allow-manifest-mismatch --reloption graph_degree=32 --reloption build_list_size=100 --reloption alpha=1.2 --log-file reviews/task-65/002-vamana-core-measurement/artifacts/load-real10k-diskann-pq-fastscan-loaderfix-r32-l100-short.log` | Fixed loader copied/staged corpus in `5.73s`, inserted encoded rows in `1.84s`, built `task65_lfix_r10k_pq_fastscan_idx` in `7.62s`, completed prefix in `24.95s`. |
| `recall-real10k-diskann-pq-fastscan-loaderfix-r32-l100.log` | behavioral recall | real10k, fixed-loader prefix, `k=10`, L=`64,128,200`, 200 queries | `cargo run -p ecaz-cli --bin ecaz -- --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task65_lfix_r10k --profile ec_diskann --k 10 --sweep 64,128,200 --queries-limit 200 --force-index --truth-cache-file reviews/task-65/002-vamana-core-measurement/artifacts/truth-real10k-k10.json --log-output reviews/task-65/002-vamana-core-measurement/artifacts/recall-real10k-diskann-pq-fastscan-loaderfix-r32-l100.log` | Recall@10 `0.9965 / 0.9970 / 0.9975`; holds Task 29d final baseline `0.9965 / 0.9965 / 0.9970`. |
| `load-real10k-diskann-pq-fastscan-loaderfix-r32-l200.log` | performance + behavioral recall | real10k, `pq_fastscan`, `graph_degree=32`, `build_list_size=200`, `alpha=1.2` | `cargo run -p ecaz-cli --bin ecaz -- --database postgres --host /Users/peter/.pgrx --port 28818 corpus load --prefix task65_real_l200 --profile ec_diskann --storage-format pq_fastscan --corpus-file fixtures/m5_diskann_real10k/m5_diskann_real10k_corpus.tsv --queries-file fixtures/m5_diskann_real10k/m5_diskann_real10k_queries.tsv --allow-manifest-mismatch --reloption graph_degree=32 --reloption build_list_size=200 --reloption alpha=1.2 --log-file reviews/task-65/002-vamana-core-measurement/artifacts/load-real10k-diskann-pq-fastscan-loaderfix-r32-l200.log` | Built `task65_real_l200_pq_fastscan_idx` in `14.92s`, under the `16s` Task 65 target; completed prefix in `32.10s`. |
| `recall-real10k-diskann-pq-fastscan-loaderfix-r32-l200.log` | behavioral recall | real10k, R32/L200, `k=10`, L=`64,128,200`, 200 queries | `cargo run -p ecaz-cli --bin ecaz -- --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task65_real_l200 --profile ec_diskann --k 10 --sweep 64,128,200 --queries-limit 200 --force-index --truth-cache-file reviews/task-65/002-vamana-core-measurement/artifacts/truth-real10k-k10.json --log-output reviews/task-65/002-vamana-core-measurement/artifacts/recall-real10k-diskann-pq-fastscan-loaderfix-r32-l200.log` | Recall@10 `0.9975 / 0.9975 / 0.9975`; holds/improves Task 29d final baseline. |
| `load-real10k-diskann-pq-fastscan-release-r32-l100.log` | earlier performance | real10k, `pq_fastscan`, `graph_degree=32`, `build_list_size=100`, `alpha=1.2` | `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 corpus load --prefix task65_real10k_diskann_pq_rel_r32_l100 --profile ec_diskann --storage-format pq_fastscan --corpus-file fixtures/m5_diskann_real10k/m5_diskann_real10k_corpus.tsv --queries-file fixtures/m5_diskann_real10k/m5_diskann_real10k_queries.tsv --allow-manifest-mismatch --reloption graph_degree=32 --reloption build_list_size=100 --reloption alpha=1.2 --log-file reviews/task-65/002-vamana-core-measurement/artifacts/load-real10k-diskann-pq-fastscan-release-r32-l100.log` | Pre-loaderfix build `7.42s`, total `11.26s`; superseded by the fixed-loader run for closure evidence. |
| `recall-real10k-diskann-pq-fastscan-release-r32-l100.log` | earlier behavioral recall | real10k, `pq_fastscan`, `k=10`, L=`64,128,200`, 200 queries | `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task65_real10k_diskann_pq_rel_r32_l100 --profile ec_diskann --k 10 --sweep 64,128,200 --queries-limit 200 --force-index --truth-cache-file reviews/task-65/002-vamana-core-measurement/artifacts/truth-real10k-k10.json --log-output reviews/task-65/002-vamana-core-measurement/artifacts/recall-real10k-diskann-pq-fastscan-release-r32-l100.log` | Recall@10 `0.9965 / 0.9970 / 0.9975`. |
| `load-synth10k-diskann-pq-fastscan-release-r32-l100.log` | behavioral smoke | synth10k, `pq_fastscan`, `graph_degree=32`, `build_list_size=100`, `alpha=1.2` | `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 corpus load --prefix task65_synth10k_diskann_pq_rel_r32_l100 --profile ec_diskann --storage-format pq_fastscan --corpus-file fixtures/m5_diskann_synth10k/m5_diskann_synth10k_corpus.tsv --queries-file fixtures/m5_diskann_synth10k/m5_diskann_synth10k_queries.tsv --allow-manifest-mismatch --reloption graph_degree=32 --reloption build_list_size=100 --reloption alpha=1.2 --log-file reviews/task-65/002-vamana-core-measurement/artifacts/load-synth10k-diskann-pq-fastscan-release-r32-l100.log` | Built in `28.71s`, completed prefix in `31.65s`. |
| `recall-synth10k-diskann-pq-fastscan-release-r32-l100-l64-200-800.log` | behavioral smoke | synth10k, `pq_fastscan`, `k=10`, L=`64,200,800`, 200 queries | `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task65_synth10k_diskann_pq_rel_r32_l100 --profile ec_diskann --k 10 --sweep 64,200,800 --queries-limit 200 --force-index --truth-cache-file reviews/task-65/002-vamana-core-measurement/artifacts/truth-synth10k-k10.json --log-output reviews/task-65/002-vamana-core-measurement/artifacts/recall-synth10k-diskann-pq-fastscan-release-r32-l100-l64-200-800.log` | Recall@10 `0.1605 / 0.2500 / 0.3315`; L=200 remains `1.65pp` below old smoke value, while L=800 is above old value. |
| `load-synth10k-diskann-pq-fastscan-release-r32-l200.log` | behavioral smoke | synth10k, `pq_fastscan`, `graph_degree=32`, `build_list_size=200`, `alpha=1.2` | `cargo run -p ecaz-cli --bin ecaz -- --database postgres --host /Users/peter/.pgrx --port 28818 corpus load --prefix task65_syn_l200 --profile ec_diskann --storage-format pq_fastscan --corpus-file fixtures/m5_diskann_synth10k/m5_diskann_synth10k_corpus.tsv --queries-file fixtures/m5_diskann_synth10k/m5_diskann_synth10k_queries.tsv --allow-manifest-mismatch --reloption graph_degree=32 --reloption build_list_size=200 --reloption alpha=1.2 --log-file reviews/task-65/002-vamana-core-measurement/artifacts/load-synth10k-diskann-pq-fastscan-release-r32-l200.log` | Built in `35.67s`, completed prefix in `50.38s`. |
| `recall-synth10k-diskann-pq-fastscan-release-r32-l200-l64-200-800.log` | behavioral smoke | synth10k, R32/L200, `k=10`, L=`64,200,800`, 200 queries | `cargo run -p ecaz-cli --bin ecaz -- --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task65_syn_l200 --profile ec_diskann --k 10 --sweep 64,200,800 --queries-limit 200 --force-index --truth-cache-file reviews/task-65/002-vamana-core-measurement/artifacts/truth-synth10k-k10.json --log-output reviews/task-65/002-vamana-core-measurement/artifacts/recall-synth10k-diskann-pq-fastscan-release-r32-l200-l64-200-800.log` | Recall@10 `0.1610 / 0.2625 / 0.3270`; L=200 is `-0.40pp` vs old `0.2665`, within the 0.5pp gate. |
| `dhat-vamana-build-real1k-r32-l200-summary.md` | memory/profile smoke | first 1,000 rows from real10k, R32/L200 Vamana hot loop | `cargo run --release --features bench,dhat-heap --bin dhat_vamana_build -- --input fixtures/m5_diskann_real10k/m5_diskann_real10k_corpus.tsv --rows 1000 --graph-degree 32 --list-size 200 --alpha 1.2 --seed 42 --output reviews/task-65/002-vamana-core-measurement/artifacts/dhat-vamana-build-real1k-r32-l200.json --summary-output reviews/task-65/002-vamana-core-measurement/artifacts/dhat-vamana-build-real1k-r32-l200-summary.md` | Profiled only `build_vamana_graph_with_stats` after input parse/medoid selection; elapsed `14050ms`, greedy `924ms`, robust-prune `1880ms`, backlink `9735ms`, visited p95 `202`. |
| `dhat-vamana-build-real1k-r32-l200.json` | memory/profile smoke | dhat heap JSON for first 1,000 real10k rows | Same command as summary. | `dhatFileVersion=2`, mode `rust-heap`, command line records `--rows 1000`; backtraces show expected `SearchScratch::new` and graph/prune allocations. |
| `validation-summary.md` | validation | compile, test, install, loaderfix gates | Plain commands, no shell redirection. | `cargo fmt --check`, `cargo check` lanes, direct DiskANN test lane, release install, fixed-loader builds, recall checks, and dhat harness check all passed. |
| `hot-loop-static-audit.md` | memory/code audit | build hot loop | `rg -n "<hot-loop allocation and frontier patterns>" src/am/ec_diskann/ambuild.rs src/am/ec_diskann/vamana.rs src/am/ec_diskann/routine.rs` | Build pivot loop uses `SearchScratch` + `Vec<u64>` bitsets and bounded heaps. No build hot-loop `vec![false; n]`, `frontier.truncate`, or build-time exact-vector dedup remains. |

## Baseline References

- Task 29c active-mask floor: `70.678s` real10k release-mode build, cited in
  `plan/tasks/29c-diskann-build-perf.md` and
  `plan/tasks/29d-diskann-pre-landing-perf-sweep.md`.
- Task 29d real10k final recall baseline:
  `reviews/task-29d/003-11109-task29d-final-readiness/artifacts/manifest.md`
  records L=`64,128,200` recall@10 `0.9965 / 0.9965 / 0.9970`.
- Task 29 synth10k smoke baseline:
  `reviews/task-29/017-30204-task29-diskann-m5-neon-rerank/request.md`
  records synthetic recall@10 `0.1650 / 0.2665 / 0.3260` at
  L=`64,200,800` and explicitly says this fixture is kernel-correctness only
  because high-dimensional synthetic vectors are nearly equidistant.

## Gate Assessment

- Functional: passed via direct `cargo test -p ecaz --features pg18 ec_diskann`
  (`182 passed; 0 failed`) plus compile/fmt lanes.
- Performance: fixed-loader real10k index build is `7.62s` at R32/L100 and
  `14.92s` at R32/L200; target was `<=16s`.
- Memory: static audit plus `dhat_vamana_build` 1k smoke confirms the build
  hot loop uses reusable scratch bitsets/heaps rather than N-sized
  `vec![false; n]` per-search allocations.
- Behavioral: real10k recall holds/improves; synth10k R32/L200 L=200 is
  `0.2625` vs old `0.2665`, within the 0.5pp gate.
