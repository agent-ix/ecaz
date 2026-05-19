# Review Request: Quant Prod SIMD Dispatch Safety

Head: `72ebc9ca7acb544a11d6b8a90c9046834ece9034`

Scope:
- `src/quant/prod.rs`
- `scripts/unsafe_comment_baseline.txt`
- `reviews/task-35/023-quant-prod-simd-dispatch-safety/artifacts/*`

What changed:
- Documented the remaining unsafe SIMD dispatch and test-harness boundaries in
  `src/quant/prod.rs`.
- Covered AVX2/FMA and NEON runtime dispatch for product-quantizer exact
  scoring.
- Covered 3-bit MSE-code SIMD scoring dispatch.
- Covered the AVX2 test-only lane decode block and unaligned store target.

Baseline accounting:
- Global baseline: 2,989 -> 2,977.
- `src/quant/prod.rs`: 12 -> 0.
- `src/quant/*`: 12 -> 0.
- Baseline files: 100 -> 99.

Unsafe sites:
- Removed no unsafe blocks in this slice.
- Added nearby `// SAFETY:` contracts for all remaining `src/quant/prod.rs`
  unsafe blocks.
- The `src/quant` production surface is now clear from
  `scripts/unsafe_comment_baseline.txt`.

Validation:
- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/unsafe-audit-after.log`
  - result: passes.
- `make unsafe-baseline-report`
  - artifacts: `artifacts/unsafe-baseline-report-before.log`,
    `artifacts/unsafe-baseline-report-after.log`
  - result: 2,989 -> 2,977.
- `cargo fmt --all`
  - artifact: `artifacts/cargo-fmt.log`
  - result: passes; unrelated fmt churn was restored before commit.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18-bench.log`
  - result: passes with the existing unused-import warnings.
- `git diff --check`
  - artifact: `artifacts/git-diff-check.log`
  - result: passes.

Reviewer focus:
- Whether the dispatch comments sufficiently bind the runtime backend selection
  to the required target features and quantizer mode preconditions.
- Whether closing the `src/quant` baseline surface is acceptable as two focused
  packets: 022 for Hadamard, 023 for product-quantizer dispatch.
