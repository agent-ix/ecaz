# Task 39 / 053 — SPIRE vec_id.rs mutation campaign (full back-fill)

## Goal

Eighth slice of the SPIRE storage mutation cascade. Drive every
mutation in `src/am/ec_spire/storage/vec_id.rs` to **0 missed /
0 timeouts** (excluding documented equivalents).

## Result

**148 mutations enumerated → 133 KILLED + 15 equivalent,
0 non-equivalent survivors.**

Back-fills the earlier partial verification (31/148) under
`CARGO_TARGET_DIR=$(pwd)/target-mutants` build isolation. All 15
remaining MISSED are in two reviewer-accepted equivalence classes:

- **Disjoint-flag** (5): `|→^` on `SPIRE_ASSIGNMENT_KNOWN_FLAGS`
  combining disjoint flag bits.
- **Encoder/decoder-symmetric constants** (10): byte-layout
  constants on lines 42, 75, 76, 83, 127 that are consumed
  identically by encoder and decoder, so any mutation shifts both
  sides equally and round-trip tests still pass.

## Methodology

Full per-mutation apply/test/revert via
`/tmp/run_spire_mutations_v2.py` with isolated build cache.

## Code change

None — all surviving mutants are documented equivalents.

## Validation

Artifacts under `reviews/task-39/053-spire-vec-id-mutation/artifacts/`:

- `vec-id-mutants-enumerated.txt` — full 148 enumeration.
- `manual-verification.log` — 148/148 per-mutation verdicts.
- `post-verification-tests.log` — clean re-run after revert.

`triage.md` documents each equivalence class with line-level
justification.
