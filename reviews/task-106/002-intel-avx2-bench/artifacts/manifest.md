# Task 106 packet 002 — Intel AVX2 provenance manifest

- Head SHA: `46d79ff8e43b0047902508476d7ac88b84fb2ac9` before the
  warning-only `mb_neon.rs` import fix in this packet.
- Host / lane: Intel local, `Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz`,
  `x86_64`, AVX2 + FMA present. Local pgrx PG18 instance at
  `/home/peter/.pgrx`, port `28818`.
- Task bucket / packet: `reviews/task-106/002-intel-avx2-bench/`.
- Fixture: existing local DBpedia 10k fixture at
  `/home/peter/dev/datasets/tqhnsw_real_10k/`.
  - Corpus: `tqhnsw_real_10k_corpus.tsv`, 10000 rows, sha256
    `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75`.
  - Queries: `tqhnsw_real_10k_queries.tsv`, 200 rows, sha256
    `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`.
  - These hashes match the M5 Task 106 / Task 31 baseline fixture. The suite
    uses `allow_manifest_mismatch` only because benchmark table prefixes differ
    from the fixture manifest prefix.
- Note: an attempted prepare to `data/task106_intel_dbpedia_staged/` was not
  used for suite evidence. The accepted suite evidence below uses the existing
  `tqhnsw_real_10k` fixture.

## Commands

Build / static:
- `cargo build --release`
  - raw: `raw-cargo-build-release.log`
  - result: passed with an x86-only warning from `mb_neon.rs`.
- `cargo build --release` after cfg import fix
  - raw: `raw-cargo-build-release-after-neon-import-fix.log`
  - result: passed warning-free for the touched library.
- `cargo check --all-targets --features bench`
  - raw: `raw-cargo-check-all-targets-bench.log`
  - result: passed.
- `cargo clippy --lib`
  - raw: `raw-cargo-clippy-lib.log`
  - result: passed.
- `cargo build --release -p ecaz-cli`
  - raw: `raw-cargo-build-release-ecaz-cli.log`
  - result: passed; pre-existing `LoadedDistributedPlacementConfig::path`
    dead-code warning remains in the CLI.

Rust focused tests:
- `cargo test --lib rabitq32`
  - raw: `raw-cargo-test-lib-rabitq32.log`
  - result: 7 passed.
- `cargo test --lib ec_ivf::quantizer`
  - raw: `raw-cargo-test-lib-ec-ivf-quantizer.log`
  - result: 28 passed.
- `cargo test --lib ec_ivf::scan`
  - raw: `raw-cargo-test-lib-ec-ivf-scan.log`
  - result: 24 passed.
- `cargo test --lib ec_spire::options`
  - raw: `raw-cargo-test-lib-ec-spire-options.log`
  - result: 24 passed.

PG18 smoke:
- `cargo pgrx test pg18 test_ec_ivf_rabitq_storage_build_scan_insert_vacuum`
  - raw: `raw-pgtest-ivf-rabitq-roundtrip.log`
  - result: passed.
- `cargo pgrx test pg18 test_ec_ivf_recall_smoke_compares_exact_hnsw_ivf`
  - raw: `raw-pgtest-ivf-recall-smoke.log`
  - result: passed.
- `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`
  - raw: `raw-cargo-pgrx-install-release.log`
  - result: release backend installed before suite run.

Microbench:
- `cargo bench --features bench --bench quant_score -- rabitq32_multibit`
  - raw: `raw-criterion-multibit-sweep-avx2.log`
  - key 1536-dim lines:
    - bits=2 scalar estimate: median `138.88 us`
    - bits=2 block dispatch: median `69.377 us`
    - bits=4 scalar estimate: median `12.810 us`
    - bits=4 block dispatch: median `72.900 us`

Index-level suite:
- `target/release/ecaz bench suite audit --config reviews/task-106/002-intel-avx2-bench/task106-intel-ivf-rabitq-multibit.json`
  - raw: `suite-audit.log`
  - result: audit passed, 8 steps.
- `target/release/ecaz --database postgres --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-106/002-intel-avx2-bench/task106-intel-ivf-rabitq-multibit.json --manifest-output reviews/task-106/002-intel-avx2-bench/artifacts/suite/suite-manifest.json --results-output reviews/task-106/002-intel-avx2-bench/artifacts/suite/results.jsonl`
  - raw: `suite/` plus `suite-report.log`
  - result: 8 completed, 0 failed.

## Key Suite Results

- bits=1: p50 `1.61 ms`; block counters present:
  `quant=rabitq isa=avx2 kernel_candidates=530088 scalar_candidates=0`.
- bits=2: p50 `7.93 ms`; block counters present:
  `quant=rabitq isa=avx2 kernel_candidates=530088 scalar_candidates=0`.
- bits=4: p50 `2.76 ms`; no block-kernel counter row, confirming estimator
  routing.
- bits=8: p50 `2.28 ms`; no block-kernel counter row, confirming estimator
  routing.

## AWS Prep

- Use the same Task 106 suite shape but bind corpus paths to the existing
  canonical AWS/staged fixture for the 10k DBpedia corpus, not to the m5
  absolute `/Users/...` path.
- Confirm fixture hashes match the same corpus/query hashes above before using
  AWS latency numbers as comparable evidence.
- Expected routing gate on AWS Intel: bits=1 and bits=2 emit
  `quant=rabitq isa=avx2`; bits=4 and bits=8 emit no RaBitQ block-kernel
  counter rows.
