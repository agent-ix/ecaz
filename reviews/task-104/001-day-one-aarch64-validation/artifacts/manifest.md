# Task 104 packet 001 — day-one aarch64 validation artifacts

- Task bucket: `reviews/task-104/`
- Packet: `001-day-one-aarch64-validation/`
- Host: Apple M5 Pro, `aarch64-apple-darwin` (see `host-info.log`)
- Base SHA at first run: `2dcbb8cc0abeec5d27bcd41ac10edba7bc7e1fe2` (latest main)
- Code-fix SHA: `16133580a` (Task 104: align qjl32 NEON production scorer, fix stale counter/cap tests)
- Branch: `task-104-m5-bench-optimization`
- Date: 2026-06-11
- Lane: local M5 unit/parity validation — no PG runtime, no bench surfaces;
  all runs are `cargo test --lib` (debug) against default `pg18` feature.
- Isolation: not applicable (pure unit tests, no tables).

## Commands

Each family log is one invocation of:

    cargo test <filter> --lib -- --nocapture --color never

### Day-one smoke set (the G4 day-one set, at base SHA)

| Artifact | Filter | Result |
| --- | --- | --- |
| `cargo-test-lut32-neon-transpose.log` | `transpose_8x16_yields_byte_columns` | ok, 1 passed |
| `cargo-test-lut32.log` | `quant::lut32` | ok, 9 passed |
| `cargo-test-qjl32.log` | `quant::qjl32` | **FAILED, 7 passed / 3 failed** (pre-fix — kept as the failure evidence) |
| `cargo-test-rabitq32.log` | `quant::rabitq32` | ok, 6 passed |
| `cargo-test-int8-approx32.log` | `quant::int8_approx32` | ok, 4 passed |
| `cargo-test-hamming32.log` | `quant::hamming32` | ok, 3 passed |
| `cargo-test-grouped-pq.log` | `grouped_pq` | ok, 35 passed |
| `cargo-test-candidate-batch.log` | `candidate_batch` | ok, 19 passed |
| `cargo-test-diskann-prefilter-batch.log` | `prefilter_batch` | ok, 4 passed |
| `cargo-test-diskann-codec-batch-counters.log` | `diskann_grouped_pq_prefilter_codec_batch_uses_block_kernel_counters` | ok, 1 passed |
| `host-info.log` | rustc host triple, CPU brand, base SHA | — |

### Post-fix validation (at code-fix SHA)

| Artifact | Filter | Result |
| --- | --- | --- |
| `cargo-test-qjl32-after-neon-alignment.log` | `quant::qjl32` | ok, 10 passed |
| `cargo-test-qjl-neon-prod-parity.log` | `qjl_neon_production` | ok, 1 passed (1000-candidate gate on aligned NEON production path) |
| `cargo-test-qjl-pre-task104-diagnostic.log` | `qjl_pre_task104` | ok (diagnostic print, see key lines) |
| `cargo-test-ivf-pqfastscan-isolated.log` | `pq_fastscan_payload_batch_scores_match_scalar_and_records_counters` | ok, 1 passed |
| `cargo-test-spire-executor-isolated.log` | `production_receive_adapters_reject_selected_pid_batches_before_connection` | ok, 1 passed |
| `cargo-test-spire-qjl-assignment-isolated.log` | `turboquant_qjl_assignment_batch_uses_qjl32_path` | ok, 1 passed (fresh process — confirms the full-sweep failure was a poisoned-mutex cascade, not a real failure) |
| `cargo-test-full-lib-single-thread.log` | `--lib -- --skip pg_test --test-threads=1` | see log tail (definitive full unit sweep at code-fix SHA) |
| `cargo-test-full-lib.log` | `--lib -- --skip pg_test` (multi-thread, pre-fix) | FAILED — kept as evidence; failures were the two root causes below plus `postgres FFI may not be called from multiple threads` panics and PoisonError cascades that do not reproduce single-threaded/fresh-process |
| `cargo-test-ivf-quantizer-isolated.log` | `am::ec_ivf::quantizer` (pre-fix) | FAILED 1/27 — isolates the IVF counter failure; rabitq bits1 tests pass fresh |
| `cargo-clippy.log` | `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` | clean |

`pg_test_*` live-PG tests are excluded on macOS per the standing policy (known
macOS pgrx runtime blocker); they remain Linux/G4 coverage.

## Key result lines

Pre-fix qjl32 failures (base SHA, first-ever aarch64 execution):

    qjl32_block32_matches_pre_slice_scorer_bits: assertion failed: matches!(isa, Isa::Scalar | Isa::Avx2)
    qjl32_block32_matches_production_dispatch_tolerance: actual=0.036570866 expected=0.03657082 ulp=12 rel=1.2223812e-6
    qjl32_scalar_reference_matches_production_dispatch_tolerance: actual=0.035220683 expected=0.035220638 ulp=12 rel=1.2692411e-6

Pre-fix NEON production-scorer divergence quantified (diagnostic over 1000
candidates, dim=1024 bits=4, preserved old shape):

    qjl_pre_task104_neon_multi_accum_tolerance_diagnostic dim=1024 bits=4 candidates=1000 max_ulp=13600 max_rel=9.433868108e-4 violations=298 worst_seed=924

Post-fix: `quant::qjl32` 10/10 ok; `qjl_neon_production_path_matches_scalar_reference_tolerance`
passes the production gate (ulp<=4 or rel<=1e-6) on all 1000 candidates.

Other root-caused failures (both stale tests exposed on first SIMD-host /
current-cap execution, fixed in the same commit):

    am::ec_ivf::quantizer pq_fastscan_payload_batch_scores_match_scalar_and_records_counters:
      assertion failed: grouped_pq.iter().any(|snapshot| snapshot.kernel_candidates == 32)
      — hardcoded pre-F8 32-kernel/7-scalar split; the Task 101/94-F8 width-cascade
      merge gives full-coverage sub-width SIMD dispatch (39 kernel / 0 scalar on any
      SIMD host). Aligned with the ISA-aware pattern already used by the shared
      candidate_batch grouped-PQ test.

    am::ec_spire production_receive_adapters_reject_selected_pid_batches_before_connection:
      expected remote_payload_too_large, got connect_failed — test sends 65 PIDs
      against a cap raised 64 -> 128 by 553cd24ec (2026-05-28); broken on every host
      since. Now derives the oversized count from
      current_session_max_remote_payload_rows_per_batch() + 1.
