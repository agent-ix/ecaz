# Task 93 Packet 003: RaBitQ32 NEON Backend (Phase B)

Builds on the approved packet 002 (scalar reference + IVF routing). This
packet lands the real NEON backend and its local Phase B measurement on the
M5 (native NEON host).

## Commit under review

- `1b447f544` — Task 93 Phase B: real NEON rabitq32 backend via production
  pair primitive.

## Design choice: reuse the production NEON primitive

`rabitq32/neon.rs` does not introduce a new NEON summation. It calls the
existing production pair primitive `sum_query_dequant_neon_bits1_pair`
(`src/quant/rabitq.rs`, now `pub(crate)`; a `# Safety` doc was added to the
single-candidate variant which lacked one). Consequences:

- the kernel's floating-point operation order is **identical** to the
  production `estimate_ip_bits1_batch` NEON path — proven by the new
  aarch64 test `neon_block32_is_bit_equal_with_production_neon_batch`
  (bit-equality by construction, same pairing);
- one NEON implementation to maintain; no drift between kernel and
  production scoring;
- `PreparedBits1` gains `dequant_lut` (the primitive's <32-dim tail needs
  `lut[0]`/`lut[1]`), populated by both accessors.

Dispatch returns `Isa::Neon` only when runtime detection succeeds;
non-aarch64 builds and NEON-less hosts fall back to the scalar backend. The
`unsafe` boundary stays inside `rabitq32/neon.rs` + the pre-existing
primitive; both carry `# Safety` docs covering feature detection and
shape/length invariants (validated by the batch wrapper before dispatch).

## Tolerance contract (one deliberate revision — please review)

Strict `f32::to_bits()` parity remains pinned to the forced-scalar anchor,
now via direct scalar-backend calls (`scalar_block32_matches_forced_scalar_anchor_bits`).

For the dispatched SIMD kernel vs the scalar anchor, the nominal
"≤4 ULP or 1e-6 relative" figure is **not achievable** for reordered FMA
summation at production dimensions: measured 22 ULP / 1.55e-6 relative at
dim=1536 on NEON (the production NEON scorer has the same property vs the
scalar order — this is inherent to accumulator trees, not a kernel defect).
Rather than weaken the anchor or modify any production path, the test suite
uses:

1. **bit-equality with the production NEON batch path** (the same-order
   reference) — the tight SIMD anchor;
2. a documented **1e-5 envelope** vs the forced-scalar anchor, matching the
   existing production differential precedent in `rabitq.rs` tests
   (`bits8_batch_estimator_matches_scalar_order` et al. use 1e-5);
3. **recall byte-equality at bench level** as the binding correctness gate —
   it passes at every cell below.

If the reviewer prefers a different envelope shape (e.g. dimension-scaled
ULP), happy to adjust; the measured numbers are in the manifest.

## Validation

All on HEAD `1b447f544` (logs in `artifacts/`): clippy `-D warnings` clean;
rabitq32 5/5, candidate_batch 10/10, ec_ivf 27/27 (plus ec_diskann 12/12,
ec_hnsw 81/81 run pre-commit). Counter and routing tests are ISA-aware and
assert `isa=neon` rows on this host.

## Bench evidence (local M5, PG18, `ecaz bench suite`)

Same fixtures/cells as packet 002; full numbers in `artifacts/manifest.md`.

- **Recall byte-equal at every cell** (real10k 0.8953 ×2, real100k 0.7719).
- **Per-ISA scoring-share gate (≥2×) passes on every cell** vs the packet-002
  scalar kernel: 3.55× (10k np8), 2.69× (10k np32), 2.90× (100k np32),
  measured from direct `[block-kernel-counters]` `isa=neon` rows.
- **End-to-end**: kernel-on now at parity on real10k and faster than
  kernel-off on real100k (p50 3.57 vs 3.82 ms). The Phase 2 scalar-kernel
  latency gap is closed.
- Run note: a latent `ec_ivf` planner interaction (`count(*)` over a table
  with an existing ec_ivf index errors with "requires exactly one ORDER BY
  query") forced a table-drop + rerun; see manifest. Flagged as a follow-up
  outside Task 93 scope.

## Review request

Please review the NEON dispatch/safety boundary, the primitive reuse
decision, the revised SIMD tolerance contract (§above), and the Phase B
measurement. Next slices: HNSW + DiskANN runtime batch accumulation, then
SVE (Graviton lane) and AVX2 (Intel lane) backends.
