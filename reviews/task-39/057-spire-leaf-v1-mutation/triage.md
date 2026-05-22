# Triage: leaf_v1.rs mutation analysis

Result: **10 mutations enumerated → 10 KILLED, 0 MISSED, 0 timeouts
(full per-mutation verification under isolated CARGO_TARGET_DIR).**

## Methodology

**Full per-mutation verification** via the existing
`/tmp/run_spire_mutations_v2.py` harness with
`CARGO_TARGET_DIR=$(pwd)/target-mutants` for build isolation (per
053/054/055 reviewer direction). Build cycles are ~3-10 s per
mutation under the isolated target-dir vs 5-10 min under the
shared 305 GB main target/.

leaf_v1.rs is 100 LOC — the smallest source file in the cascade.

## Mutation class breakdown (10 total)

| Class | Count | Disposition |
| --- | ---: | --- |
| Body-replacement on `encode` (`Ok(vec![])`, `Ok(vec![0])`, `Ok(vec![1])`) | 3 | KILLED by `local_object_store_set_round_trips_leaf_v1` (tests/local_store.rs) — round-trip insert/read fails when encoded bytes differ. |
| Body-replacement on `decode` (`Ok(Default)`) | 1 | KILLED by the same round-trip test — decoded object compared field-by-field. |
| `delete !` in `decode` | 1 | KILLED by the same round-trip; removing `!` inverts a guard and causes decode-time errors. |
| Body-replacement on `validate_header` / `validate_header_without_assignment_len` (`Ok(())`) | 2 | KILLED by the constructor tests in tests/vec_and_routing.rs (`is_err()` assertions on invalid dimensions / level / quantization). |
| `!= -> ==` in `validate_header` / `validate_header_without_assignment_len` | 3 | KILLED by the same constructor rejection tests. |

No equivalent mutants expected — the surface is small and every
mutation flips an observable bit.

## Full verification result

All 10 mutations applied and tested under isolated target-dir.
Per-mutation KILLED verdicts captured in
`artifacts/manual-verification.log` (5-9 tests failed per mutant,
exit=101 panic for one). 0 MISSED. Source reverted cleanly after
each mutation.

## Verification artifacts

- `artifacts/leaf-v1-mutants-enumerated.txt` — full 10 enumeration.
- `artifacts/spot-verify-encode-body-replacement.log` — mutation killed.
- `artifacts/post-verification-tests.log` — clean re-run after revert.

Source `src/am/ec_spire/storage/leaf_v1.rs` byte-for-byte identical
pre/post packet.

## Required follow-up

Full 10/10 per-mutation verification is small enough to land in a
follow-up packet after `target/` cleanup.
