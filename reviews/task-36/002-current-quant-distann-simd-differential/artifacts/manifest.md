# Artifact manifest

- Head SHA: `c373eb51f61654226f81b489a4be4a40e6e45025`
- Task bucket: `reviews/task-36/`
- Packet: `reviews/task-36/002-current-quant-distann-simd-differential/`
- Captured: 2026-07-25 UTC
- Host: Apple arm64 (`Darwin 25.4.0`, `RELEASE_ARM64_T6050`)
- Scope: local differential tests and review evidence; no benchmark corpus, index,
  rerank, or storage-format measurement applies because the checkpoint changes
  only test/bench hooks, test cases, the local make lane, and documentation.
- Isolation: unit/integration process only; no PostgreSQL table or index was used.

## Artifacts

### `make-simd-diff.log`

- Command: `make simd-diff`
- Status: PASS
- Locally executed ISA: AArch64 NEON and dot-product/SDOT.
- Reported unavailable on this host: x86 AVX2/FMA, x86 AVX-512, AArch64
  SVE/SVE2.
- Key results:
  - public bench-hook integration: 10 passed
  - RaBitQ arithmetic inventory: 2 passed
  - `rabitq32`: 7 passed
  - `qjl32`: 10 passed
  - `lut32`: 9 passed
  - grouped PQ: 8 passed
  - int8/SDOT: 5 passed
  - `hamming32`: 3 passed
  - DistANN composition: 2 passed

### `distann-simd-diff.log`

- Command:
  `cargo test --lib --features bench simd_diff_ -- --test-threads=1 --nocapture`
- Status: PASS (2 passed)
- Key results:
  - exact DistANN distance is the negation of the shared DiskANN source-inner-
    product path
  - grouped PQ (64 dimensions), RaBitQ (1536), and TurboQuant (1536) prepared
    batch scores match direct per-code scores for widths
    `1,7,8,9,16,17,31,32,33`
  - poison payload beyond `count * stride` is ignored

### `mutation-control-failure.log`

- Command: `make simd-diff`
- Temporary mutation:
  `score_candidate_neon_dotprod` returned
  `sum as f32 * prepared.score_scale + 1.0`.
- Status: EXPECTED FAILURE (`make` exit 2; Rust test exit 101).
- Key result: four int8/SDOT differential tests failed, covering direct
  legacy-vs-SDOT, block/partial dispatch, extreme i8 inputs, and dimension
  tails. The mutation was then reverted; `git diff --exit-code --
  src/quant/int8_approx32/neon.rs` passed before the final green lane.

### `cargo-fmt-check.log`

- Command: `cargo fmt --all -- --check`
- Status: FAIL (pre-existing repository-wide formatting drift).
- The output includes many files and older regions outside this checkpoint.
  It also reports older formatting drift elsewhere in the two DistANN files
  that received narrowly appended tests. A repository-wide rewrite was not
  included in Task 36.
- `git diff --cached --check` passed before the implementation commit, and the
  committed checkpoint has no whitespace errors reported by `git diff`.

## Claim boundary

This packet makes a local Apple arm64 claim only. It does not claim CI coverage
or execution on Intel or Graviton. AVX2/AVX-512 and SVE/SVE2 implementations
that exist in the source inventory remain hardware validation follow-ups.
