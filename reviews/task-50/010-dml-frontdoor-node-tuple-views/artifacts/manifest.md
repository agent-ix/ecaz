# Manifest

- head SHA: `55d0ef521fa6`
- task bucket: `reviews/task-50/010-dml-frontdoor-node-tuple-views`
- timestamp: `2026-05-19T23:30:23-07:00`
- lane: SPIRE DML frontdoor structural unsafe reduction
- fixture / storage format / rerank mode: not applicable; planner/catalog helper refactor only
- table surface: not applicable; no benchmark table was created

## Artifacts

| artifact | command | result |
| --- | --- | --- |
| `block-count-before.log` | `make unsafe-block-count PATHS='src/am/ec_spire/dml_frontdoor/mod.rs'` before edit | `160 src/am/ec_spire/dml_frontdoor/mod.rs` |
| `block-count-after.log` | `make unsafe-block-count PATHS='src/am/ec_spire/dml_frontdoor/mod.rs'` after edit | `146 src/am/ec_spire/dml_frontdoor/mod.rs` |
| `rustfmt-check.log` | `rustfmt --edition 2021 --check src/am/ec_spire/dml_frontdoor/mod.rs` | passed; stable rustfmt warns that repo config uses nightly-only import grouping knobs |
| `git-diff-check.log` | `git diff --check` | passed |
| `cargo-check-pg18-bench.log` | `cargo check --all-targets --no-default-features --features pg18,bench` | passed with existing unused-import warnings |
| `clippy-pg18.log` | `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` | failed on existing repo-wide lint backlog; local DML file lints observed in the first run were cleaned before this artifact run |
| `cargo-test-dml-frontdoor.log` | `cargo test dml_frontdoor --lib --no-default-features --features pg18` | built, then failed to launch outside PostgreSQL with `undefined symbol: CacheRegisterRelcacheCallback` |

## Key Result Lines

```text
before: 160 src/am/ec_spire/dml_frontdoor/mod.rs
after:  146 src/am/ec_spire/dml_frontdoor/mod.rs
delta:  -14 unsafe blocks (-8.75%)
```
