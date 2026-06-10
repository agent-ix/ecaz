# Task 93 Packet 002: RaBitQ32 Scalar Reference + IVF Batch Routing

This packet implements Phase 2 of the approved RaBitQ32 design
(`reviews/task-93/001-rabitq32-design/`, approved in feedback seq 02): the
scalar block kernel with the revised parity contract, the `Diskann` counter
surface obligation, AM codec registration, and the first runtime routing with
local bench evidence.

## Commits under review

- `5ea1a1945` — Task 93 Phase 2: rabitq32 scalar block kernel + bits=1 batch
  registration.
- `b3dcf46d7` — merge of `origin/main` (Task 94 grouped-PQ landed there
  mid-slice): main's `candidate_batch.rs` structure (`BLOCK_KERNEL_ALL` /
  4-wide `TASK87_ALL`, its own `Diskann` surface,
  `CANDIDATE_BATCH_COUNTER_TEST_LOCK`) was taken wholesale and the Phase 2
  work re-applied on top. Reviewing the merged state is sufficient; the
  pre-merge commit is retained for history.

## Code

- `src/quant/rabitq32/{mod,scalar,neon,sve,avx2}.rs` (ADR-076 layout):
  - `scalar.rs` reproduces the forced-scalar bits=1 byte-LUT operation order
    (`sum_query_dequant_bits1_byte_lut_scalar` + `finish_scalar_only_estimate`
    from `src/quant/rabitq.rs`) exactly — this is the deterministic parity
    anchor agreed in packet 001.
  - `neon.rs` / `sve.rs` / `avx2.rs` are safe fallback stubs that delegate to
    scalar and return `Isa::Scalar` (no `unsafe`, no `# Safety` surface yet).
  - `PreparedBits1` carries `{dimensions, code_len, query_rotated,
    bits1_byte_lut}` borrowed from prepared-query state; `validate()` checks
    query/code-length invariants before any scoring.
- Narrow read-only accessors `bits1_block_prepared(code_len)` on
  `PreparedEstimator` and `RaBitQScorer` (`src/quant/rabitq.rs`); both return
  `None` unless `bits_per_dim == 1` and the byte LUT exists.
- Shared wrapper `score_rabitq_bits1_batch_for` in
  `src/am/common/candidate_batch.rs`: 32-wide blocking with scalar tail,
  shape/meta/count validation strictly before counters, kernel rows recorded
  under the backend-returned ISA, tails under
  `(surface, rabitq, scalar)`.
- `QuantCodec::score_ip_batch` overrides for RaBitQ bits=1 on
  `IvfQuantCodec` (surface `Ivf`), `DiskannRaBitQPrefilterCodec` (surface
  `Diskann`), `HnswRaBitQScanCodec` (surface `Hnsw`); non-bits=1 falls back to
  per-candidate scoring.
- Runtime routing (IVF only in this slice):
  `IvfQuantizer::score_ip_bits1_batch_from_payloads` bits=1 arm now builds a
  `CandidateBatch` and routes through the kernel wrapper, mirroring the
  Task 87 TurboQuant arm at the same call site; bits=8 stays on
  `estimate_ip_batch`. This path only engages when the default-off
  `ec_ivf.scratch_soa_batch_decode` diagnostic GUC is on, so default
  production scoring is unchanged. HNSW/DiskANN runtime scan loops score
  per-candidate today and their batch accumulation is a follow-up slice; their
  codec overrides are registration-complete per the design's Phase 2 scope.

## Parity contract (as approved in packet 001 feedback seq 02)

- Strict `f32::to_bits()` vs forced-scalar anchor: `rabitq32` tests build an
  independent byte-LUT reference with identical operation order and assert
  bit-equality for the scalar tail, the 32-wide block, and (via the
  candidate_batch tests) block+tail batches.
- ADR-076 tolerance vs production-dispatched scorers:
  `production_dispatch_is_within_phase2_tolerance` compares the kernel anchor
  against `estimate_ip_scalar_only` and `estimate_ip_bits1_batch` (both select
  NEON kernels on this host) under 1e-6 relative tolerance, and includes the
  one-shot production per-candidate vs production batch agreement check.
- Width gates: `<32` (scalar-only attribution), `==32`, `>32` (block + tail);
  shape, metadata, and output-count mismatches reject before counters
  increment (`rabitq_bits1_batch_shape_mismatch_rejects_before_counters`).

## Validation

All on merged HEAD `b3dcf46d7`, logs in `artifacts/`:

- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` clean.
- Focused tests: rabitq32 3/3, candidate_batch 10/10, ec_ivf 27/27,
  ec_diskann 12/12, ec_hnsw scan 81/81.
- `cargo pgrx test` not run on this host (known macOS `_BufferBlocks` dyld
  blocker); live-callback behavior is covered by the bench run below against
  a real PG18 backend.

## Bench evidence (local M5, PG18, `ecaz bench suite`)

Suite config `crates/ecaz-cli/suites/task93-phase2-ivf-rabitq.json`; full
details and key result lines in `artifacts/manifest.md`.

- **Recall byte-equal at every cell** (gate 1): real10k nprobe∈{8,32} both
  0.8953 with identical percentiles; real100k nprobe=32 both 0.7719.
- **Direct `[block-kernel-counters]` rows** with `quant=rabitq isa=scalar`:
  98.6–99.9% of candidates through the 32-wide kernel (e.g. real100k:
  409568 of 410113), zero rows in the kernel-off cell.
- **Latency**: kernel-on (forced-scalar kernel behind the diagnostic GUC) is
  slower than the kernel-off NEON-dispatched per-candidate baseline
  (real10k nprobe=32 p50 3.71 ms vs 1.45 ms). Expected for the Phase 2 scalar
  baseline; production default path is untouched, and the ≥2× scoring-share
  gate applies per-ISA when the NEON/SVE/AVX2 backends land (Phases B–D).
  Documented per the task's stop conditions, not backed out.

## Review request

Please review the scalar kernel parity contract implementation, the counter
attribution wiring, the IVF runtime routing (including that it is gated
behind the default-off SoA GUC), and the merge-resolution choices against
main's Task 94 structure. Next slice after approval: NEON backend
(`rabitq32/neon.rs`) with forced-NEON differential tests, then HNSW/DiskANN
runtime batch accumulation.
