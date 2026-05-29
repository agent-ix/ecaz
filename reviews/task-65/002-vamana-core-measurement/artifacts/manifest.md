# Task 65 Measurement Manifest

Head SHA: `de2ef72e40472b8c9259e203be76cfdc0313c4d5`

Task bucket: `reviews/task-65/002-vamana-core-measurement`

Lane: local PG18 DiskANN Vamana build performance, small corpus only.

Surface: isolated one-index-per-table prefixes unless noted.

## Code Under Measurement

- `351987249` - `Optimize DiskANN Vamana build core`
- `da2807c0e` - `Bound DiskANN greedy frontier with heaps`
- `4bd460081` - `Fix DiskANN build validation edges`
- `a8b0b8789` - `Fix DiskANN build visibility handling`
- `de2ef72e4` - `Trim DiskANN Vamana build hot path`

## Artifacts

| Artifact | Lane | Fixture / Format | Command | Key result |
|---|---|---|---|---|
| `precheck-pg18.log` | PG18 precheck | `postgres`, socket `/Users/peter/.pgrx`, port `28818` | `/Users/peter/.cargo/bin/ecaz dev sql --pg 18 --db postgres --socket-dir /Users/peter/.pgrx --raw --sql "SELECT version();" --log-output reviews/task-65/002-vamana-core-measurement/artifacts/precheck-pg18.log` | PostgreSQL 18.3 Homebrew on aarch64. |
| `install-ecaz-pg-test-after-loader-fix.log` | install | PG18 `ecaz-pg-test` | `/Users/peter/.cargo/bin/ecaz dev install ecaz-pg-test --pg 18 --log-file reviews/task-65/002-vamana-core-measurement/artifacts/install-ecaz-pg-test-after-loader-fix.log` | Installed release backend after the loader/visibility fixes; installed dylib sha256 `fbe83817...`. |
| `load-real10k-diskann-pq-fastscan-loaderfix-r32-l100-short.log` | performance + loaderfix | real10k, `pq_fastscan`, `graph_degree=32`, `build_list_size=100`, `alpha=1.2` | `cargo run -p ecaz-cli --bin ecaz -- --database postgres --host /Users/peter/.pgrx --port 28818 corpus load --prefix task65_lfix_r10k --profile ec_diskann --storage-format pq_fastscan --corpus-file fixtures/m5_diskann_real10k/m5_diskann_real10k_corpus.tsv --queries-file fixtures/m5_diskann_real10k/m5_diskann_real10k_queries.tsv --allow-manifest-mismatch --reloption graph_degree=32 --reloption build_list_size=100 --reloption alpha=1.2 --log-file reviews/task-65/002-vamana-core-measurement/artifacts/load-real10k-diskann-pq-fastscan-loaderfix-r32-l100-short.log` | Fixed loader copied/staged corpus in `5.73s`, inserted encoded rows in `1.84s`, built `task65_lfix_r10k_pq_fastscan_idx` in `7.62s`, completed prefix in `24.95s`. |
| `recall-real10k-diskann-pq-fastscan-loaderfix-r32-l100.log` | behavioral recall | real10k, fixed-loader prefix, `k=10`, L=`64,128,200`, 200 queries | `cargo run -p ecaz-cli --bin ecaz -- --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task65_lfix_r10k --profile ec_diskann --k 10 --sweep 64,128,200 --queries-limit 200 --force-index --truth-cache-file reviews/task-65/002-vamana-core-measurement/artifacts/truth-real10k-k10.json --log-output reviews/task-65/002-vamana-core-measurement/artifacts/recall-real10k-diskann-pq-fastscan-loaderfix-r32-l100.log` | Recall@10 `0.9965 / 0.9970 / 0.9975`; holds Task 29d final baseline `0.9965 / 0.9965 / 0.9970`. |
| `load-real10k-diskann-pq-fastscan-release-r32-l100.log` | earlier performance | real10k, `pq_fastscan`, `graph_degree=32`, `build_list_size=100`, `alpha=1.2` | `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 corpus load --prefix task65_real10k_diskann_pq_rel_r32_l100 --profile ec_diskann --storage-format pq_fastscan --corpus-file fixtures/m5_diskann_real10k/m5_diskann_real10k_corpus.tsv --queries-file fixtures/m5_diskann_real10k/m5_diskann_real10k_queries.tsv --allow-manifest-mismatch --reloption graph_degree=32 --reloption build_list_size=100 --reloption alpha=1.2 --log-file reviews/task-65/002-vamana-core-measurement/artifacts/load-real10k-diskann-pq-fastscan-release-r32-l100.log` | Pre-loaderfix build `7.42s`, total `11.26s`; superseded by the fixed-loader run for closure evidence. |
| `recall-real10k-diskann-pq-fastscan-release-r32-l100.log` | earlier behavioral recall | real10k, `pq_fastscan`, `k=10`, L=`64,128,200`, 200 queries | `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task65_real10k_diskann_pq_rel_r32_l100 --profile ec_diskann --k 10 --sweep 64,128,200 --queries-limit 200 --force-index --truth-cache-file reviews/task-65/002-vamana-core-measurement/artifacts/truth-real10k-k10.json --log-output reviews/task-65/002-vamana-core-measurement/artifacts/recall-real10k-diskann-pq-fastscan-release-r32-l100.log` | Recall@10 `0.9965 / 0.9970 / 0.9975`. |
| `load-synth10k-diskann-pq-fastscan-release-r32-l100.log` | behavioral smoke | synth10k, `pq_fastscan`, `graph_degree=32`, `build_list_size=100`, `alpha=1.2` | `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 corpus load --prefix task65_synth10k_diskann_pq_rel_r32_l100 --profile ec_diskann --storage-format pq_fastscan --corpus-file fixtures/m5_diskann_synth10k/m5_diskann_synth10k_corpus.tsv --queries-file fixtures/m5_diskann_synth10k/m5_diskann_synth10k_queries.tsv --allow-manifest-mismatch --reloption graph_degree=32 --reloption build_list_size=100 --reloption alpha=1.2 --log-file reviews/task-65/002-vamana-core-measurement/artifacts/load-synth10k-diskann-pq-fastscan-release-r32-l100.log` | Built in `28.71s`, completed prefix in `31.65s`. |
| `recall-synth10k-diskann-pq-fastscan-release-r32-l100-l64-200-800.log` | behavioral smoke | synth10k, `pq_fastscan`, `k=10`, L=`64,200,800`, 200 queries | `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task65_synth10k_diskann_pq_rel_r32_l100 --profile ec_diskann --k 10 --sweep 64,200,800 --queries-limit 200 --force-index --truth-cache-file reviews/task-65/002-vamana-core-measurement/artifacts/truth-synth10k-k10.json --log-output reviews/task-65/002-vamana-core-measurement/artifacts/recall-synth10k-diskann-pq-fastscan-release-r32-l100-l64-200-800.log` | Recall@10 `0.1605 / 0.2500 / 0.3315`; L=200 remains `1.65pp` below old smoke value, while L=800 is above old value. |
| `validation-summary.md` | validation | compile, test, install, loaderfix gates | Plain commands, no shell redirection. | `cargo fmt --check`, `cargo check` lanes, direct DiskANN test lane against `de2ef72e4`, release install, fixed-loader build, and fixed-loader recall all passed. |
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
- Performance: fixed-loader real10k index build `7.62s`; target was `<=16s`.
- Memory: static audit confirms the build hot loop has no N-sized
  `vec![false; n]` allocation and no linear scan/sort/truncate frontier.
  `heaptrack` / standalone `dhat` were not installed on this host.
- Behavioral: real10k recall holds/improves; synth10k stays in the known
  low-recall smoke envelope but L=200 is weaker than the nominal `0.5pp` gate.
