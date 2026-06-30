# Task 124 / 034 TQ Dimension/Subspace Manifest

- head SHA: `1285dd489fc5e44540b19930b6b615d4259b2747`
- task bucket: `reviews/task-124/034-tq-dimension-subspace`
- lane: local release unit/profiler
- fixture: synthetic TurboQuant2 QJL scorer inputs, dimensions swept from 1536 to 256
- storage format: TurboQuant2 scorer payload code, `[mse_1bit][qjl_1bit]`
- rerank mode: scorer-only TQ2 QJL batch path
- timestamp: 2026-06-30
- isolated one-index-per-table: not applicable; scorer-only microprofile

## Artifacts

### `tq2-dimension-profile.log`

Command:

```sh
ECAZ_TQ2_DIM_PROFILE_LOG=reviews/task-124/034-tq-dimension-subspace/artifacts/tq2-dimension-profile.log cargo test --release --lib --features bench task124_profile_tq2_dimension_sweep -- --ignored --nocapture
```

Key result lines:

```text
task124_tq2_dimension_profile backend=neon total_candidates=192000
dim=1536 width=100 code_bytes=384 batch_ns_per_candidate=330.2
dim=1280 width=100 code_bytes=320 batch_ns_per_candidate=276.2
dim=1024 width=100 code_bytes=256 batch_ns_per_candidate=218.4
dim=768 width=100 code_bytes=192 batch_ns_per_candidate=163.1
dim=512 width=100 code_bytes=128 batch_ns_per_candidate=111.7
dim=384 width=100 code_bytes=96 batch_ns_per_candidate=82.4
dim=256 width=100 code_bytes=64 batch_ns_per_candidate=56.8
```

### `validation.log`

Commands:

```sh
cargo fmt --check
ECAZ_TQ2_DIM_PROFILE_LOG=reviews/task-124/034-tq-dimension-subspace/artifacts/tq2-dimension-profile.log cargo test --release --lib --features bench task124_profile_tq2_dimension_sweep -- --ignored --nocapture
```

Key result lines:

```text
cargo fmt --check: passed (stable rustfmt warns that repo rustfmt.toml has nightly-only import options)
dimension sweep profiler: 1 passed; 0 failed
```
