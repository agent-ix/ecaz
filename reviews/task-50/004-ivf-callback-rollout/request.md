# Task 50 Packet 004: IVF Callback Rollout

## Code Under Review

- Commit: `f69928abeb7acd46dff2b61fa538a4801b2906fb`.
- Task: `plan/tasks/50-unsafe-structural-reduction.md`.
- Slice: 1b IVF callback rollout.

## Scope

This packet extends the Slice 1a callback helper into the IVF callback surface:

- Added `pg_am_callback!` beside `am_callback` in `src/am/common/callback.rs`.
- Converted the remaining IVF `pgrx_extern_c_guard` callback wrappers in:
  - `src/am/ec_ivf/cost.rs`
  - `src/am/ec_ivf/insert.rs`
  - `src/am/ec_ivf/options.rs`
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_ivf/vacuum.rs`
- Left `src/am/ec_ivf/build.rs` for a later direct build-path slice; its
  callback body is tightly coupled to build-state training and deserves its own
  review packet.

The macro centralizes the PostgreSQL callback-duration invariant: raw
PostgreSQL pointers captured by the callback body remain live for the guarded
callback invocation, and Rust unwinds must not cross the C ABI.

## Structural Result

| file | before | after | delta | percent | top-15 target status |
| --- | ---: | ---: | ---: | ---: | --- |
| `src/am/ec_ivf/scan.rs` | 102 | 98 | -4 | -3.9% | Top-15 rank 7; partial Slice 1b progress, more reduction expected from Slice 2 and scorer work. |
| `src/am/ec_ivf/vacuum.rs` | 26 | 23 | -3 | -11.5% | Not top-15. |
| `src/am/ec_ivf/insert.rs` | 21 | 20 | -1 | -4.8% | Not top-15. |
| `src/am/ec_ivf/cost.rs` | 9 | 8 | -1 | -11.1% | Not top-15. |
| `src/am/ec_ivf/options.rs` | 8 | 7 | -1 | -12.5% | Not top-15. |
| `src/am/common/callback.rs` | 1 | 2 | +1 | n/a | Shared helper macro, now used by multiple IVF callback files. |

Net touched-surface direct unsafe count: 167 -> 158.

## Risk Register

Relevant row: **AM callback helper**.

- Failure mode: wrapper inhibits inlining or changes panic/error boundary
  shape.
- Mitigation: macro expands to the same `unsafe { pgrx::pgrx_extern_c_guard(|| { ... }) }`
  shape used before; no extra function call is introduced for raw-pointer
  callbacks.
- Verification: compile check passed; block counts captured before/after.
  Runtime benchmarks are skipped because this preserves callback guard shape
  and does not change scan scoring/traversal logic.

## Validation

Artifacts are under `artifacts/`:

- Block count: `block-count-before.log`, `block-count-after.log`.
- Formatting: `rustfmt-touched-check.log` passed for touched files.
- Compile: `cargo-check-pg18-bench.log` passed.
- Diff hygiene: `git-diff-check.log` passed.

Known validation limitations:

- `cargo fmt --all --check` still fails on unrelated formatting diffs in
  `hardening/careful` and `src/quant/simd.rs`; log captured.
- The required full clippy command still fails on existing repo-wide warnings;
  log captured. This packet also removes the pre-existing IVF clippy findings
  in the touched files, so the remaining failures are outside the touched
  production files.

## Tests / Benches

- Runtime tests: skipped. This is callback-boundary consolidation; callback
  body logic is intentionally unchanged.
- Benchmarks: skipped per `bench-baseline-plan.md`; no scoring, traversal,
  cache, build, or distributed-read hot path logic changed.
