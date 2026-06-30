# Task 124 / 032 TQ2 SIMD Scorer Manifest

- head SHA: `10d734062a0a1fe60d02ecb017e705bd124d68c2`
- task bucket: `reviews/task-124/032-tq2-simd-scorer`
- lane: local release unit/profiler
- fixture: synthetic 1536-dim TurboQuant2 QJL scorer inputs
- storage format: TurboQuant2 scorer payload code, `[mse_1bit][qjl_1bit]`
- rerank mode: TurboQuant2 estimator / least-squares scorer path
- timestamp: 2026-06-30
- isolated one-index-per-table: not applicable; scorer-only microprofile

## Artifacts

### `tq2-qjl-profile.log`

Command:

```sh
ECAZ_TQ2_QJL_PROFILE_LOG=reviews/task-124/032-tq2-simd-scorer/artifacts/tq2-qjl-profile.log cargo test --release --lib --features bench task124_profile_tq2_qjl_flush_widths -- --ignored --nocapture
```

Key result lines:

```text
task124_tq2_qjl_profile backend=neon dim=1536 total_candidates=192000
width=8 scalar_ns_per_candidate=2098.4 batch_ns_per_candidate=282.8
width=16 scalar_ns_per_candidate=2100.6 batch_ns_per_candidate=281.8
width=25 scalar_ns_per_candidate=2103.9 batch_ns_per_candidate=318.8
width=32 scalar_ns_per_candidate=2114.9 batch_ns_per_candidate=285.1
width=64 scalar_ns_per_candidate=2105.3 batch_ns_per_candidate=282.6
width=96 scalar_ns_per_candidate=2106.5 batch_ns_per_candidate=280.6
width=100 scalar_ns_per_candidate=2100.5 batch_ns_per_candidate=320.3
width=128 scalar_ns_per_candidate=2108.6 batch_ns_per_candidate=280.6
```

### `validation.log`

Commands:

```sh
cargo fmt --check
cargo test --release --lib --features bench qjl2 -- --nocapture
cargo test --release --lib --features bench turboquant2 -- --nocapture
```

Key result lines:

```text
cargo fmt --check: passed (stable rustfmt warns that repo rustfmt.toml has nightly-only import options)
qjl2 tests: 3 passed; 0 failed
turboquant2 tests: 4 passed; 0 failed
```
