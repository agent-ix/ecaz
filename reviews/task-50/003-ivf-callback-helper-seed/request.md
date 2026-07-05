# Task 50 Packet 003: IVF Callback Helper Seed

## Code Under Review

- Commit: `9cb75a30cba1424ac4749f61f1c62976139ecec3`.
- Task: `plan/tasks/50-unsafe-structural-reduction.md`.
- Slice: 1a callback helper seed.

## Scope

This packet introduces the shared AM callback guard helper and applies it to
the smallest low-risk IVF callback surface:

- `src/am/common/callback.rs`: new `#[inline] am_callback` wrapper around
  `pgrx::pgrx_extern_c_guard`.
- `src/am/common/mod.rs`: exports the callback module.
- `src/am/ec_ivf/cost.rs`: converts the PG18 tree-height,
  strategy-translation, and compare-type translation callbacks to the helper.

The raw-pointer `amcostestimate` callback is deliberately left unchanged in
this seed packet; converting it would require narrowing pointer dereferences
inside the closure and belongs in a follow-up rollout.

## Structural Result

| file | before | after | delta | percent | top-15 target status |
| --- | ---: | ---: | ---: | ---: | --- |
| `src/am/ec_ivf/cost.rs` | 12 | 9 | -3 | -25.0% | Not top-15; helper seed for Slice 1b. |
| `src/am/common/callback.rs` | 0 | 1 | +1 | n/a | New centralized FFI-boundary helper, used by 3 call sites. |

Net touched-surface direct unsafe count: 12 -> 10.

The helper is used by three call sites in this packet, satisfying the Task 50
rule that moving unsafe into a helper only counts when the helper has multiple
callers.

## Risk Register

Relevant row: **AM callback helper**.

- Failure mode: wrapper inhibits inlining or changes panic/error boundary
  shape.
- Mitigation: helper is `#[inline]` and preserves the existing
  `pgrx_extern_c_guard(callback)` call shape.
- Verification: compile check passed; no runtime bench required because this
  seed only touches planner callback mapping helpers, not scan/build hot loops.

## Validation

Artifacts are under `artifacts/`:

- Block count: `block-count-before.log`, `block-count-after.log`.
- Formatting: `rustfmt-touched-check.log` passed for touched files.
- Compile: `cargo-check-pg18-bench.log` passed.
- Diff hygiene: `git-diff-check.log` passed.

Known validation limitations:

- `cargo fmt --all --check` currently fails on unrelated formatting diffs in
  `hardening/careful` and `src/quant/simd.rs`; the log is captured in
  `cargo-fmt-check.log`.
- The required full clippy command currently fails on existing repo-wide
  warnings; logs are captured in `cargo-clippy-pg18.log` and
  `cargo-clippy-lib-pg18.log`. Neither log reports diagnostics for the touched
  Slice 1a files.

## Tests / Benches

- Runtime tests: skipped. The converted callbacks only map constants/enums
  through the same guard boundary.
- Benchmarks: skipped per `bench-baseline-plan.md`; no scoring, traversal,
  cache, build, or distributed-read hot path changed.
