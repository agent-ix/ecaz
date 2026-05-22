# Triage: assignment.rs mutation analysis (back-filled)

Result: **54 mutations enumerated → 52 KILLED + 2 equivalent
(capacity-hint), 0 non-equivalent survivors. Full per-mutation
verification under isolated CARGO_TARGET_DIR, plus three new
boundary-killing tests.**

This back-fills the partial verification originally shipped in
this packet (9/54 verified). The new run was authorized by
reviewer feedback across 050/053/054/055 calling for
`CARGO_TARGET_DIR=$(pwd)/target-mutants` isolation.

## Methodology

Full per-mutation verification via `/tmp/run_spire_mutations_v2.py`
with `CARGO_TARGET_DIR=$(pwd)/target-mutants` build isolation.
Per-mutation cycles run in 3-10 s under the isolated target-dir
instead of 5-10 min under the shared 305 GB main target/.

## Per-mutation verdicts (54 total)

47 mutations KILLED by the existing test surface (round-trip +
validate-rejects-* tests). 5 mutations initially MISSED on
`decode_prefix_ref` boundary checks, killed after adding 3 new
tests. 2 mutations remain MISSED — both on
`encoded_len_after_validation` (capacity-hint equivalents).

## New killing tests (added to tests/assignment.rs)

| Test | Kills | Mechanism |
| --- | --- | --- |
| `miri_assignment_row_decode_rejects_zero_vec_id_len_at_min_prefix_boundary` | `assignment.rs:106:24 < -> ==`, `< -> <=`, and `116:28 \|\| -> &&` | All-zero buffer of exact length `SPIRE_ASSIGNMENT_ROW_FIXED_PREFIX_BYTES + SPIRE_ASSIGNMENT_ROW_FIXED_TAIL_BYTES`. Original passes line 106 then errors with `"vec_id length 0 is invalid"`. Mutants `==`/`<=` error at line 106 with `"too short"`. Mutant `\|\| -> &&` allows past the `vec_id_len == 0` guard (both conditions can never hold simultaneously) and panics on slice. The message-contains check distinguishes all three. |
| `miri_assignment_row_decode_rejects_vec_id_len_above_max` | `assignment.rs:116:42 > -> ==` | Buffer where `input[2] = SPIRE_VEC_ID_MAX_BYTES + 1`. Original rejects (`MAX+1 > MAX`); mutant `==` doesn't (`MAX+1 != MAX`). |
| `miri_assignment_row_round_trips_with_vec_id_at_max_bytes` | `assignment.rs:116:42 > -> >=` | Valid round-trip with `vec_id_len == SPIRE_VEC_ID_MAX_BYTES` exactly (global vec_id with 31-byte payload). Original accepts (`MAX > MAX` false); mutant `>=` rejects. |

Mutant re-verification after the tests were added:

| Mutant | Original | Mutant |
| --- | --- | --- |
| `< -> ==` line 106 | 556 passed | **1 failed** |
| `< -> <=` line 106 | 556 passed | **1 failed** |
| `\|\| -> &&` line 116 | 556 passed | **1 failed** |
| `> -> ==` line 116 | 556 passed | **1 failed** |
| `> -> >=` line 116 | 556 passed | **1 failed** |

## Equivalent mutants — capacity-hint only (2)

Both on line 59 (`encoded_len_after_validation -> Ok(0)` and
`-> Ok(1)`). The function's return value is consumed only as a
`Vec::with_capacity` hint via `encode_after_validation`; the actual
encoding writes fields at fixed offsets independent of the hinted
capacity. Changing the hint cannot affect any observable encoded-
byte sequence or decoded value. Same equivalence class as 053
(vec_id constant arithmetic) and 056 (reachability-restricted) —
accepted by the reviewer.

## Verification artifacts

- `artifacts/assignment-mutants-enumerated.txt` — full 54 enumeration.
- `artifacts/manual-verification.log` — 54/54 per-mutation verdicts
  (52 KILLED + 2 MISSED equivalent).
- `artifacts/post-verification-tests.log` — clean re-run after revert.

Source `src/am/ec_spire/storage/assignment.rs` byte-for-byte
identical pre/post packet (only the test file gained 3 new tests).
