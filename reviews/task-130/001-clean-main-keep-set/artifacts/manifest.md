# Task 130 Packet 001 Artifact Manifest

- branch: `task-130-tq-cleanup-main`
- base: `origin/main` at `67f23b56b`
- task bucket: `reviews/task-130/001-clean-main-keep-set/`
- timestamp: 2026-06-30

## Keep-Set Commits Cherry-Picked From Task 124

- `60d17f34d` — Add IVF TQ stage2 final rerank width
- `b87a851f1` — Add IVF TQ stage2 attribution counters
- `eff61e0aa` — Optimize IVF TQ rerank sidecar payload reads
- `d542e0fae` — Add IVF TQ rerank group width control
- `2744aa4f2` — Elide no-QJL TQ rerank gamma payload
- `0af6745d9` — Use slab for selected TQ rerank payloads
- `3be1ba32e` — Add macOS relation cache eviction fallback
- `541993ff2` — Add TQ LUT32 scorer profiler
- `3306ccf7f` — Optimize TQ no-QJL LUT query prep
- `d648f88b2` — Optimize TQ batch scorer payload cascade
- `3058c38b6` — Optimize TQ QJL batch scorer payload cascade
- `cfb209bba` — Add TQ prefetch scorer profiler

Skipped from the keep-set:

- `f1095da15` / `ed95ab3e8` / `0b3fd57f7` and their packet commits because they add recall-broken IVF binary/TQ2/reduced-dimension formats.
- `10d734062` because it adds the TQ2 SIMD path and `qjl2_32`.
- `1285dd489` because its dimension/subspace profiler depends on TQ2 and `qjl2_32`.

## Validation Commands

Completed for this packet:

- `rg "TurboQuant2|TurboQuantBinary|qjl2_32|turboquant2|turboquant_binary|tq2" src/am/ec_ivf src/am/common/candidate_batch src/quant crates/ecaz-cli`
- `git diff --name-only origin/main...HEAD`
- `git check-ignore -v reviews/task-124/037-tq2-dim768-real-index/artifacts/tq2-dim768-final15-suite/truth-100k-k10.json`
- `cargo build --release -p ecaz`
- `cargo clippy -p ecaz --lib --no-default-features --features pg18 -- -D warnings`
- `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_ivf::options -- --nocapture --test-threads=1`
- `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_ivf::scan -- --nocapture --test-threads=1`
- `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_ivf::rerank -- --nocapture --test-threads=1`
- `cargo test -p ecaz --lib --no-default-features --features pg18 am::common::candidate_batch -- --nocapture --test-threads=1`
- `cargo test -p ecaz --lib --no-default-features --features pg18 quant -- --nocapture --test-threads=1`
- `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config`
- `./target/release/ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 bench suite audit --config reviews/task-130/001-clean-main-keep-set/artifacts/task130-tq4-smoke-suite.json`
- `./target/release/ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-130/001-clean-main-keep-set/artifacts/task130-tq4-smoke-suite.json`
- `./target/release/ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 bench suite status --manifest reviews/task-130/001-clean-main-keep-set/artifacts/tq4-smoke-suite/suite-manifest.json`
- `./target/release/ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 bench suite report --manifest reviews/task-130/001-clean-main-keep-set/artifacts/tq4-smoke-suite/suite-manifest.json --results-output reviews/task-130/001-clean-main-keep-set/artifacts/tq4-smoke-suite/results-report.jsonl`

## Current Static Checks

- Source search after cherry-pick has no IVF `TurboQuant2`, IVF `TurboQuantBinary`, `qjl2_32`, or IVF `turboquant2` references. The only `turboquant_binary` references on the clean main branch are pre-existing HNSW runtime/test code from `origin/main`.
- The broad source search only returns `rabitq2` false positives in `src/am/ec_ivf/options.rs`.
- `git diff --check` passed.
- `.gitignore` ignores generated `truth-*.json` caches under `reviews/`.

## Build / Test Results

- `cargo build --release -p ecaz`: passed.
- `cargo clippy -p ecaz --lib --no-default-features --features pg18 -- -D warnings`: passed.
- `am::ec_ivf::options`: 27 passed.
- `am::ec_ivf::scan`: 30 passed.
- `am::ec_ivf::rerank`: 20 passed.
- `am::common::candidate_batch`: 13 passed, 2 ignored explicit Task 124 profiling tests.
- `quant`: 305 passed, 2 ignored explicit profiling tests.

## 4-bit TurboQuant Recall Smoke

- suite config: `reviews/task-130/001-clean-main-keep-set/artifacts/task130-tq4-smoke-suite.json`
- suite manifest: `reviews/task-130/001-clean-main-keep-set/artifacts/tq4-smoke-suite/suite-manifest.json`
- normalized results: `reviews/task-130/001-clean-main-keep-set/artifacts/tq4-smoke-suite/results.jsonl`
- report rows: `reviews/task-130/001-clean-main-keep-set/artifacts/tq4-smoke-suite/results-report.jsonl`
- load log: `reviews/task-130/001-clean-main-keep-set/artifacts/tq4-smoke-suite/load-10k-tq4-final15.log`
- recall log: `reviews/task-130/001-clean-main-keep-set/artifacts/tq4-smoke-suite/recall-10k-tq4-final15.log`

Suite status:

- completed: 2
- failed: 0
- missing artifacts: 0

Key recall lines:

- nprobe 32: recall@10 `1.0000`, 100 queries / 1000 trials, mean q-time `0.95 ms`.
- nprobe 64: recall@10 `1.0000`, 100 queries / 1000 trials, mean q-time `1.12 ms`.
