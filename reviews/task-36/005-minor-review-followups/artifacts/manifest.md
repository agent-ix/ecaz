# Artifact manifest

- Head SHA: `fbaa20a421eb3523caaccc92554df573f5771232`
- Task bucket: `reviews/task-36/`
- Packet: `reviews/task-36/005-minor-review-followups/`
- Timestamp: `2026-07-25T19:59:54Z`
- Host: Apple arm64, NEON + dot-product/SDOT detected
- Review source:
  `reviews/task-36/004-current-head-review-flags/feedback/2026-07-25-02-reviewer.md`
- Hardware boundary: AVX2/AVX-512 and SVE/SVE2 were not executed. Their
  hardware runs remain separate later evidence.
- Benchmark applicability: none. The commit changes test-only differential
  coverage, test-lane enforcement, and documentation; quantizer, index, scan,
  rerank, posting, and storage behavior are unchanged.
- Isolation/storage-format/rerank mode: not applicable to this test-only
  packet; no corpus, index, table, or benchmark surface was used.

## Commit under review

- `fbaa20a42` — add a production-entry QJL cascade differential, remove the
  test-helper ISA assertion, reject multiple Cargo summaries, and correct the
  Task 36 coverage/CI documentation.

## Artifacts

### `make-simd-diff.log`

- Head SHA: `fbaa20a421eb3523caaccc92554df573f5771232`
- Command: `make simd-diff`
- Timestamp: 2026-07-25
- Status: PASS
- Every one of the ten stages ends with
  `counted cargo test: observed expected N passed tests`.
- Counts: public 10; RaBitQ arithmetic 2; `rabitq32` 7; `qjl32` 10;
  `lut32` 9; grouped PQ 8; int8/SDOT 5; tiled-LUT guards 3;
  `hamming32` 3; production QJL plus DistANN composition/source IP 3.
- Host line:
  `arch=aarch64 backend=neon neon=true dotprod=true sve=false sve2=false`.
- New composition result:
  `production_cascade_isa=neon widths=1,7,8,9,16,17,31,32,33`.

### `multiple-summary-negative-control.log`

- Head SHA: `fbaa20a421eb3523caaccc92554df573f5771232`
- Command:
  `bash scripts/run-counted-cargo-test.sh 0 --lib --test simd_diff
  --features bench task36_no_matching_test -- --test-threads=1`
- Timestamp: 2026-07-25
- Status: EXPECTED FAILURE from the counted wrapper.
- Cargo emits two individually successful zero-test summaries, one for the
  library and one for `tests/simd_diff.rs`.
- Key result:
  `expected exactly one test-result summary, observed 2`.

### `clippy-lib.log`

- Head SHA: `fbaa20a421eb3523caaccc92554df573f5771232`
- Command:
  `cargo clippy --lib --features bench --no-deps -- -D warnings`
- Timestamp: 2026-07-25
- Status: FAIL on an unrelated existing finding at
  `src/am/ec_ivf/quantizer.rs:695`.
- Key result: Clippy reports `manual checked division`; it reports no finding
  in the six files changed by the Task 36 commit.

## Additional local checks

- `rustfmt --edition 2021 --check
  src/am/common/candidate_batch/mod.rs src/quant/qjl32/mod.rs`: PASS
- `bash -n scripts/run-counted-cargo-test.sh`: PASS
- `git diff --check`: PASS before the code commit
