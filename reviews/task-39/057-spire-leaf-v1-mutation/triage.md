# Triage: leaf_v1.rs mutation analysis

Result: **10 mutations enumerated → analysis-only verification:
predicted 10 KILLED + 0 equivalent, 0 non-equivalent survivors.
1 spot-verified.**

## Methodology

Analysis-only per packets 050 / 053-056. leaf_v1.rs is 100 LOC —
the smallest source file in the cascade.

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

## Spot-verify

`SpireLeafPartitionObject::encode` body replaced with
`return Ok(vec![]);`. `cargo test --manifest-path
hardening/careful/Cargo.toml --lib` reports **8 tests FAILED**
under the mutant (round-trip and downstream). Post-revert run
reports **550 passed, 0 failed**. Source reverted cleanly.

## Verification artifacts

- `artifacts/leaf-v1-mutants-enumerated.txt` — full 10 enumeration.
- `artifacts/spot-verify-encode-body-replacement.log` — mutation killed.
- `artifacts/post-verification-tests.log` — clean re-run after revert.

Source `src/am/ec_spire/storage/leaf_v1.rs` byte-for-byte identical
pre/post packet.

## Required follow-up

Full 10/10 per-mutation verification is small enough to land in a
follow-up packet after `target/` cleanup.
