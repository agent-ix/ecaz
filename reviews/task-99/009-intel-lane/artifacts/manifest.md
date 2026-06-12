# Manifest — Task 99 packet 009: AWS Intel lane (COMPLETE)

- Lane: **AWS Intel production column** — `10k-intel`, db instance
  `i-02479540bbab734ea` (m7i.2xlarge, Sapphire Rapids), us-west-2,
  restored from `snap-0e9c7743263e61d70`; `ecaz cloud status` cost
  line: ~$0.493/hr running.
- Git ref installed: `63bf4d78c`; installed backend
  `/usr/lib64/pgsql/ecaz.so` sha256 `6554890c…` (bracketed in logs).
- Database: `tqvector_bench` (same snapshot sources as the G4 lane).

## Day-one gate (PASSED, one documented exception)

- `day1-smoke-attempt1.log`: lut32 11 / qjl32_ 11 passed; rabitq32
  4 passed + **2 failed by exactly 1 ULP** (see finding).
- `day1-smoke2.log`: rabitq32 remainder green with the strict pair
  skipped; grouped_pq 34 / hamming32 3 / int8_approx32 4 /
  candidate_batch 19 / quant::isa 8 — all passed with
  `--skip pg_test_`; backend sha unchanged.

## Finding — rabitq32 strict bit-equality is not host-portable (1 ULP)

`simd_block32_is_bit_equal_with_production_batch` and
`partial_dispatch_matches_anchor_and_production_batch` assert
`to_bits()` equality between the rabitq32 kernel and the production
batch path ("same order by construction"). On m7i (Sapphire Rapids,
AVX-512-capable) with the repo's `target-cpu=native` build
(`.cargo/config.toml`), the two inline contexts codegen differently and
diverge by **exactly 1 ULP** (e.g. `1088422957` vs `1088422956`).
The family's **binding** gates both pass on this host:
`production_dispatch_is_within_phase2_tolerance` and
`dispatched_block32_matches_anchor_within_tolerance` (the 1e-5
envelope / forced-scalar anchor contract, ADR-076 tolerance lane —
1 ULP ≈ 1e-7 relative, ~100× inside the envelope). The same tests are
bit-equal on the local Intel desktop, the M5, and Graviton 4 — the
divergence is specific to this microarchitecture × target-cpu=native.

**Disposition**: lane proceeds under documented exception (the failing
assertions are test-encoded claims, not production behavior; no kernel
code differs). Post-trip test-only follow-up: weaken the two strict
assertions to the family envelope (or gate the strictness on a
codegen-stable configuration), reviewed as its own slice. Recall
parity on this lane's rabitq cells is evaluated under the ADR-076
tolerance-family rule and reported explicitly either way.

## Lane execution (complete)

- Catalog refresh applied (same stale-snapshot fix as the G4 lane:
  241 function defs replayed, 0 errors — `catalog-refresh.log`).
- Fixtures: `fixtures.log`, 11 indexes, sources 100,000/1,000 rows,
  rc=0 (`t99-fixture-sources-aws.sql`).
- Main profile run (`profile-run/`): **91/91 succeeded, 34/34 recall
  on/off pairs byte-equal** (including the rabitq cells — the 1-ULP
  codegen divergence did not disturb recall), `isa=avx2` kernel
  attribution at every kernel cell, structure identical to the local
  Intel column (packet 003).
- Post-lane snapshot: **`snap-0dc395f4f6458c37b`**; stack destroyed
  after snapshot.

Selected kernel rates (avx2, ns/candidate, citable Intel column):
lut32 IVF/SPIRE/HNSW/DiskANN = 170/180/374/213; rabitq 71–92;
int8_approx 96; qjl32 215–235 @1024d; grouped-pq 132–138;
binary = scalar POPCNT (skip decision upheld).
