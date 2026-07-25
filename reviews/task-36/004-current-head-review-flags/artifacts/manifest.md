# Artifact manifest

- Head SHA: `a915b062bac167532f961ee77dd184905db58d90`
- Task bucket: `reviews/task-36/`
- Packet: `reviews/task-36/004-current-head-review-flags/`
- Date: 2026-07-25
- Host: Apple arm64, NEON + dot-product/SDOT detected
- Review source:
  `reviews/task-36/002-current-quant-distann-simd-differential/feedback/2026-07-25-01-reviewer.md`
- Hardware boundary: AVX2/AVX-512 and SVE/SVE2 changes are statically reviewed
  only in this packet; their hardware runs remain separate later evidence.
- Benchmark applicability: none. These changes affect test-lane enforcement,
  assertions, documentation, and a manual workflow command; production
  quantizer/index/scan/rerank/posting/storage behavior is unchanged.

## Commits under review

- `911e543ed` — run every filtered test stage through a counted wrapper that
  rejects missing summaries and unexpected pass counts.
- `e37df9309` + `a915b062b` — require every x86 public forced hook to return an
  AVX2/FMA result rather than silently skip, plus its formatting-only follow-up.
- `052a55553` — restrict QJL tolerance to observed SIMD block candidates,
  retain bit equality for scalar tails, and print the block ISA returned by the
  scorer.
- `90999afa3` — make the manual CI SIMD matrix invoke the authoritative
  `make simd-diff` lane and document its trigger/hardware boundary.

The earlier `07aa36712` and `6df8d4a30` commits, documented in packet 003,
address the tiled-LUT regression and int8 ISA pin called out again by this
review.

## Artifacts

### `make-simd-diff.log`

- Command: `make simd-diff`
- Status: PASS
- Every stage ends with
  `counted cargo test: observed expected N passed tests`.
- Counts: public 10; RaBitQ arithmetic 2; `rabitq32` 7; `qjl32` 10;
  `lut32` 9; grouped PQ 8; int8/SDOT 5; tiled-LUT guards 3;
  `hamming32` 3; DistANN 2.
- QJL reports
  `observed_block_isa=neon scalar_tail=bit-exact`.

### `empty-filter-negative-control.log`

- Command:
  `bash scripts/run-counted-cargo-test.sh 1 --lib --features bench
  quant::nonexistent_task36_filter -- --test-threads=1`
- Status: EXPECTED FAILURE
- Cargo itself reports 0 passed and exit 0; the wrapper rejects it with
  `expected 1 passed tests, observed 0`.

### `quant-lib-tests.log`

- Command:
  `cargo test --lib --features bench quant:: -- --test-threads=1 --nocapture`
- Status: PASS
- Result: 188 passed, 0 failed, 3 ignored.

### `cargo-fmt-check.log`

- Command: `cargo fmt --all -- --check`
- Status: FAIL on the verified inherited repository-wide rustfmt drift.
- No diff is reported for the changed Rust files
  `tests/simd_diff.rs` or `src/quant/qjl32/mod.rs`.
- `git diff --check` passes.

### Workflow syntax

- Command:
  `ruby -e 'require "yaml"; YAML.parse_file(".github/workflows/ci.yml");
  puts "ci.yml syntax ok"'`
- Status: PASS (`ci.yml syntax ok`)

## Flag disposition

1. **Lossy name filters / missing prod stage:** the prod guard stage was added
   in `07aa36712`; all ten stages now have explicit expected counts. The empty-
   filter negative control proves fail-closed behavior.
2. **x86 forced hooks could skip:** all four x86 areas now use `expect`/`assert`
   for HNSW, DiskANN, Prod direct/code-to-code, and FWHT. Intel execution is
   still owed and is not inferred from Apple compilation.
3. **int8 lacked an ISA assertion:** fixed by `6df8d4a30`, with scalar-cap
   negative evidence in packet 003.
4. **QJL tolerance was overly broad/underexplained:** scalar-only widths and
   the tail after a block are bit-exact. Only candidates actually scored by a
   non-scalar 32-wide block receive 4 ULP/relative `1e-6`, because vector
   reduction changes accumulation order. The test prints the returned block
   ISA, not host capability.
5. **Manual CI ran less than the local lane:** its SIMD matrix now runs
   `make simd-diff`. Triggers remain manual-only and no automatic CI claim is
   made.

## Reviewer verification artifacts (2026-07-25, seq 02)

Added by `feedback/2026-07-25-02-reviewer.md`. Verified at branch head
`8923245ee` (packet head `a915b062b` plus the packet-004 request commit) in a
clean worktree on Apple M5 (aarch64, NEON + dotprod, no SVE).

### `2026-07-25-reviewer-make-simd-diff-repro.log`

- Command: `make simd-diff`
- Status: PASS (exit 0), independently reproducing `make-simd-diff.log`.
- Key result: all ten counted stages report
  `counted cargo test: observed expected N passed tests` with N =
  10 / 2 / 7 / 10 / 9 / 8 / 5 / 3 / 3 / 2.

### `2026-07-25-reviewer-counted-wrapper-controls.log`

- Commands: three direct invocations of `scripts/run-counted-cargo-test.sh`.
- Status: all three correctly fail.
- Key result:
  - wrong expected count (99 vs 3) → `exit=1`
  - filter matching nothing → `exit=1, observed 0` (the packet-002 fail-open
    hole)
  - cargo itself failing → `exit=1`, cargo's own status propagated rather than
    swallowed by the summary parsing.

### `2026-07-25-reviewer-isa-cap-control.log`

- Command: `ECAZ_ISA_CAP=scalar cargo test --lib --features bench quant::int8_approx32:: -- --test-threads=1`
- Status: FAIL, as designed.
- Key result: `int8 differential test did not execute the host's preferred SIMD
  ISA` on every int8 differential test, confirming the pinning compares against
  the uncapped host preference and cannot pass vacuously under a cap.
