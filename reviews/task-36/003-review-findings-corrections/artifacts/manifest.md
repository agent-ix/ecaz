# Artifact manifest

- Head SHA: `15e3831c13b65b488fea1c0f1ac1da8d46e321f1`
- Task bucket: `reviews/task-36/`
- Packet: `reviews/task-36/003-review-findings-corrections/`
- Date: 2026-07-25
- Host: Apple arm64, NEON + dot-product/SDOT detected
- Scope: corrections prompted by feedback written against older head
  `48fc8ee21`; review is requested against the current Task 36 branch.
- Benchmark applicability: none. The production change restores rejected-input
  preconditions and promotes a query-preparation shape check to a hard assert;
  valid-lane scoring, index, rerank, posting, and storage behavior are
  unchanged.

## Commits under review

- `07aa36712` — restore tiled-LUT query length/lane guards, harden the
  centroid/codebook shape invariant, add three regression tests, and include
  them in `make simd-diff`.
- `6df8d4a30` — require int8 block and partial differential tests to report the
  host-preferred ISA, preventing scalar-vs-scalar vacuity.
- `15e3831c1` — replace stale automatic-PR-gate language with the repository's
  actual manual-dispatch and packet-backed validation policy.

## Artifacts

### `make-simd-diff.log`

- Command: `make simd-diff`
- Status: PASS
- Key results:
  - public SIMD harness: 10 passed
  - RaBitQ arithmetic: 2 passed
  - `rabitq32`: 7 passed
  - `qjl32`: 10 passed
  - `lut32`: 9 passed
  - grouped PQ: 8 passed
  - int8/SDOT with host-ISA assertions: 5 passed
  - tiled-LUT safety guards: 3 passed
  - `hamming32`: 3 passed
  - DistANN composition: 2 passed
- Host inventory line:
  `arch=aarch64 backend=neon neon=true dotprod=true sve=false sve2=false`.

### `quant-lib-tests.log`

- Command:
  `cargo test --lib --features bench quant:: -- --test-threads=1 --nocapture`
- Status: PASS
- Key result: 188 passed, 0 failed, 3 ignored.
- This is the broader slice whose earlier run exposed
  `tiled_lut_query_prep_rejects_qjl_active_lane`.

### `int8-scalar-cap-negative-control.log`

- Command:
  `env ECAZ_ISA_CAP=scalar cargo test --lib --features bench
  quant::int8_approx32::tests::block32_and_partial_are_bit_equal_with_scalar_reference
  -- --test-threads=1 --nocapture`
- Status: EXPECTED FAILURE
- Key result: the assertion reports `left: Scalar`, `right: Neon`, proving an
  environment cap can no longer turn this Apple-host differential into a
  vacuous scalar-vs-scalar pass.

### `cargo-fmt-check.log`

- Command: `cargo fmt --all -- --check`
- Status: FAIL on inherited repository-wide formatting drift.
- See `quality-checks.md` for the bounded interpretation and the separate
  clippy result.

## Finding-by-finding disposition

1. **Current block kernels absent from the lane:** already corrected by
   `c373eb51f`; the current `make simd-diff` executes all six named families.
   The green log above proves the current lane, not the older reviewed head.
2. **Stale PR-visible claim:** corrected without changing the deliberate
   manual-only CI policy. Task and hardening documentation now name packet-
   backed local execution as the pre-merge evidence.
3. **Tiled-LUT guard regression:** corrected in `07aa36712`; lane, length, and
   codebook/centroid shape failures are hard assertions covered by tests.
4. **Vacuous int8 dispatch tests:** corrected in `6df8d4a30`; every block and
   partial dispatch result is checked against uncapped host feature selection.
   The scalar-cap negative control fails at that assertion as intended.
5. **SVE unvalidated:** already explicit in the current task and hardening
   inventory. This Apple result lists SVE/SVE2 as unavailable; Graviton
   execution remains a later hardware run and is not claimed here.
