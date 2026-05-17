# Task 36: SIMD↔Scalar Differential Validation

Status: **implemented locally** — successor to Task 34 (comprehensive
hardening). The local implementation adds scalar-reference hooks,
`tests/simd_diff.rs`, `make simd-diff`, and `hardening-local` wiring. Current
validation on Linux x86 passed `cargo test --features bench --test simd_diff
-- --test-threads=1` with 5/5 tests passing, including the production 1536/4-bit
score path.

## Scope

Add randomized property-based tests that compare every SIMD scoring/decoding
implementation against a scalar reference for the same input, and gate them in
the hardening lanes so regressions are caught at PR time.

Coverage targets — every code path that has both a SIMD and a scalar variant:

- `src/quant/prod.rs`:
  - `score_ip_from_split_parts_avx2_fma`
  - `score_ip_from_split_parts_avx512`
  - `score_ip_from_split_parts_neon`
  - `score_ip_mse_codes_*` for `mse_bits` ∈ {3, 4}
  - `unpack_mse_indices` SIMD paths
  - `encode` and any vectorized inner loops in the quantizer's hot path
- `src/quant/hadamard.rs` SIMD rotation paths.
- `src/quant/rotation.rs` if it grows arch-specific variants.
- `src/am/ec_*/scan.rs` SIMD distance accumulators (DiskANN/IVF/HNSW/SPIRE).
- Any future `core::simd` or `std::simd::Simd<f32, N>` paths.

## Why

The current hardening stack does not exercise SIMD against scalar at all:

- Miri force-falls back to scalar via the `cfg!(miri)` branches added in Task
  34, so any divergence between `Avx2Fma`/`Neon` and the scalar baseline is
  invisible to Miri.
- Fuzz targets cover decoders, not scoring math.
- Property tests cover the quantizer's algebraic shape but do not pin SIMD to
  scalar bit-for-bit (or within an explicit numeric tolerance).
- Recall tests run end-to-end and would catch a catastrophic SIMD bug but not a
  subtle one — a 1% recall drop from one SIMD lane miscomputing a tail can
  easily slip past the recall floor.

SIMD bugs in scoring code are silent: no panic, no UB-by-Miri, just lower
recall. They are the single most likely class of correctness regression for a
vector database that ships hand-written intrinsics across AVX2/AVX-512/NEON.

## Approach

1. Add a `tests/simd_diff/` (or `crates/ecaz-simd-diff/`) module that exposes a
   harness:
   - Generate inputs with `proptest` strategies sized to the realistic
     production envelope (dims 4..4096, bits 2..8, gamma ∈ [-1, 1], codes from
     uniform `u8`).
   - For each backend reachable on the host CPU, run the SIMD path and the
     scalar reference on identical inputs and assert either bit-exact equality
     (integer paths) or `(a - b).abs() <= tol` with `tol` documented per metric
     (relative ≤ 1e-6 for f32 IP/L2 on bounded inputs).
2. Force-enable every SIMD backend via test-only entry points (e.g.,
   `score_ip_from_split_parts_scalar` and `..._avx2_fma_for_test`) so the host
   CPU does not silently skip a backend. Gate `_for_test` exposure behind a
   `cfg(test)` or `feature = "simd_diff"` flag so production keeps the dispatch
   path.
3. Run the same proptest under Miri's scalar mode to validate the *reference*
   path. SIMD intrinsics remain out of Miri's model, but the scalar baseline
   must itself be UB-free.
4. Add a `simd-diff` Makefile lane and add it to `hardening-local`.
5. Capture a baseline numeric-tolerance table in `docs/hardening.md` so future
   changes that need a wider tolerance must justify it in a review packet.

Optional follow-on:

- Add the same diff harness for any future `pulp` / `wide` / `core::simd`
  adoption.
- Add a "force-scalar GUC" for diagnostic recall comparisons in live PG18
  sessions.

## Validation

- `make simd-diff` passes on macOS (Neon + scalar) and Linux x86 (Avx2Fma /
  Avx512 if available + scalar).
- `make hardening-local` includes `simd-diff` and stays under the existing wall
  clock budget by capping proptest cases per backend.
- A deliberately mutated SIMD path (flip a sign in one branch) is caught by
  `simd-diff` and reported as a non-trivial diff in the packet.

## Exit Criteria

- Every SIMD scoring/decoding function listed above has a paired scalar
  reference and a property-based diff test.
- `simd-diff` runs in `hardening-local` on at least the host CPU and on the CI
  matrix (per Task 48) for both NEON and AVX2 paths.
- `docs/hardening.md` documents the per-metric tolerance and the rationale.
- The Task 34 `cfg!(miri)` scalar fallbacks remain (they cover Miri runs), but
  are documented as Miri-only — production SIMD coverage is owned here.

## Dependencies

- Independent of other proposed hardening tasks; can land first.
- Pairs naturally with Task 47 (recall/cost-model gates) which catches the
  *end-to-end* impact of a SIMD regression, while this task catches the *unit*
  divergence.
