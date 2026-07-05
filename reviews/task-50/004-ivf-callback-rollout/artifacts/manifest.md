# Task 50 Packet 004 Artifact Manifest

| Field | Value |
| --- | --- |
| Head SHA | `f69928abeb7acd46dff2b61fa538a4801b2906fb` |
| Task bucket | `reviews/task-50/004-ivf-callback-rollout` |
| Timestamp | `2026-05-19 22:11:41 PDT` |
| Slice | 1b IVF callback rollout |

## Artifacts

| Artifact | Command | Result |
| --- | --- | --- |
| `block-count-before.log` | `git show HEAD:<file>` counted for touched files before code commit | Baseline after Packet 003: `scan.rs` 102, `vacuum.rs` 26, `insert.rs` 21, `cost.rs` 9, `options.rs` 8, `callback.rs` 1. |
| `block-count-after.log` | `make unsafe-block-count PATHS='src/am/common/callback.rs src/am/ec_ivf/cost.rs src/am/ec_ivf/insert.rs src/am/ec_ivf/options.rs src/am/ec_ivf/scan.rs src/am/ec_ivf/vacuum.rs'` | After rollout: `scan.rs` 98, `vacuum.rs` 23, `insert.rs` 20, `cost.rs` 8, `options.rs` 7, `callback.rs` 2. |
| `rustfmt-touched-check.log` | `rustfmt --check src/am/common/callback.rs src/am/ec_ivf/cost.rs src/am/ec_ivf/insert.rs src/am/ec_ivf/options.rs src/am/ec_ivf/scan.rs src/am/ec_ivf/vacuum.rs` | Passed; log contains only stable-toolchain warnings about unstable rustfmt options. |
| `cargo-fmt-check.log` | `cargo fmt --all --check` | Failed on existing unrelated formatting diffs in `hardening/careful` and `src/quant/simd.rs`; touched files were checked separately. |
| `cargo-check-pg18-bench.log` | `cargo check --all-targets --no-default-features --features pg18,bench` | Passed; emitted existing unrelated warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`. |
| `cargo-clippy-pg18.log` | `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` | Failed on existing repo-wide warnings. After local IVF lint cleanup, no diagnostics target the touched production files. |
| `git-diff-check.log` | `git diff --check` | Passed with no output. |

## Bench Policy

No runtime benchmark was run. This packet preserves the existing
`pgrx_extern_c_guard` call shape through a macro expansion and does not change
scoring, traversal, cache, build training, or distributed-read logic.
