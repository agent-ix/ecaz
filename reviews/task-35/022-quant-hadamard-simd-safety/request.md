# Review Request: Quant Hadamard SIMD Safety

Head: `ce898c89904c2eedbf088f34e49692b3f7920067`

Scope:
- `src/quant/hadamard.rs`
- `scripts/unsafe_comment_baseline.txt`
- `reviews/task-35/022-quant-hadamard-simd-safety/artifacts/*`

What changed:
- Documented the unsafe SIMD boundaries in the Hadamard FWHT implementation.
- Covered AVX2/FMA and NEON dispatch with runtime feature-detection contracts.
- Covered AVX2 bootstrap, two-level tiling, staged transform pointer arithmetic,
  unaligned load/store blocks, recursive block transforms, and test-only SIMD
  harness blocks.
- Refreshed the unsafe baseline after the current checkout's IVF page line
  drift, while producing a net baseline decrease.

Baseline accounting:
- Global baseline: 3,050 -> 2,989.
- Baseline files: 101 -> 100.
- `src/quant/hadamard.rs`: 62 -> 0.
- `src/am/ec_ivf/page.rs`: 133 -> 134 from pre-existing exact-line drift on
  current main; no IVF source was changed in this packet.

Unsafe sites:
- Removed no unsafe blocks in this slice.
- Added nearby `// SAFETY:` contracts for the Hadamard SIMD unsafe blocks.
- Remaining quant baseline after this packet is `src/quant/prod.rs` with 12
  entries.

Validation:
- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/unsafe-audit-after.log`
  - result: passes.
- `make unsafe-baseline-report`
  - artifacts: `artifacts/unsafe-baseline-report-before.log`,
    `artifacts/unsafe-baseline-report-after.log`
  - result: 3,050 -> 2,989.
- `cargo fmt --all`
  - artifact: `artifacts/cargo-fmt.log`
  - result: passes; unrelated fmt churn was restored before commit.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18-bench.log`
  - result: passes with the existing unused-import warnings.
- `git diff --check`
  - artifact: `artifacts/git-diff-check.log`
  - result: passes.
- `cargo test --no-default-features --features pg18,bench fwht`
  - artifact: `artifacts/cargo-test-fwht-pg18-bench.log`
  - result: compiles, then test binary exits before running tests with
    `undefined symbol: BufferBlocks`.
- `cargo test --no-default-features fwht`
  - artifact: `artifacts/cargo-test-fwht-no-features.log`
  - result: blocked by `pgrx-pg-sys` requiring a `pg17`/`pg18` feature.

Reviewer focus:
- Whether the SIMD comments are specific enough for the AVX2/NEON feature,
  slice-length, and unaligned load/store contracts.
- Whether the baseline refresh is acceptable: it clears Hadamard and absorbs
  existing IVF page exact-line drift while reducing the global baseline by 61.
