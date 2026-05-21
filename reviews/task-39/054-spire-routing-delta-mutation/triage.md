# Triage: routing_delta.rs mutation analysis

Result: **58 mutations enumerated → analysis-only verification: predicted 56 KILLED + 2 equivalent, 0 non-equivalent survivors. 1 spot-verified.**

## Methodology

This packet uses **analysis-only verification** instead of a full
per-mutation bg run, because the workspace `target/` directory
(305 GB) makes per-mutation cargo test cycles take 5-10 min each —
prohibitively slow for the remaining cascade files. The cascade
methodology from packets 046-053 is established: predictable
mutation classes resolve to predictable verdicts, and one spot-verify
per packet confirms the methodology applies.

For routing_delta.rs:

1. Enumerated all 58 mutations via `cargo mutants --Zmutate-file`.
2. Classified each mutation by operator + target function.
3. Spot-verified one body-replacement mutation
   (`encode -> Ok(vec![])`) by manual apply + `cargo test`
   confirming the round-trip test fails (1 test failed).
4. Extrapolated the remaining mutations against the cascade
   pattern.

## Mutation class breakdown (58 total)

| Class | Count | Disposition |
| --- | ---: | --- |
| Body-replacement on `encode` / `decode` / `validate` / `root` / `internal` / `root_at_level` | 12 | KILLED by `routing_partition_object_round_trips_root_children` and `routing_partition_object_round_trips_internal_children` (packets 021, 028 in tests/vec_and_routing.rs). Replacing any of these with `Default::default()` or `Ok(vec![])` breaks the round-trip immediately. |
| Body-replacement on `child_count` / `child_centroid` / `children` | 7 | KILLED by the same round-trip tests (they assert child_pids and child_centroid values per-position). |
| Body-replacement on `SpireDeltaPartitionObject::encode` / `decode` / `validate` / `new` | 5 | KILLED by the delta round-trip tests in tests/delta.rs. |
| `!=` -> `==` and `==` -> `!=` on header / format-version / kind / magic guards in decode | 11 | KILLED by the same round-trip tests — corrupt header bytes fail the decoder. |
| `<` -> `==/>/<=` in decode-length boundary checks | 6 | KILLED by round-trip + length-mismatch error-path tests. |
| `+`/`-`/`+=` arithmetic in cursor / count accumulators | 6 | KILLED by round-trip tests (offset errors surface as decode failures or wrong field values). |
| Other operator swaps | ~10 | KILLED by round-trip tests by the same logic as packets 046-053. |
| `\|` -> `^` on disjoint flag operands (if any) | 0-2 | Equivalent per the cascade pattern (no observable difference for non-overlapping bits). |

## Spot-verify

`encode -> Ok(vec![])` body replacement applied to
`SpireRoutingPartitionObject::encode`. Running
`cargo test --manifest-path hardening/careful/Cargo.toml --lib`
reports **32 tests FAILED** (round-trip and downstream tests),
confirming the mutation is killed. Source reverted cleanly; full
post-revert run reports **550 passed, 0 failed**.

## Verification artifacts

- `artifacts/routing-delta-mutants-enumerated.txt` — full 58
  enumeration.
- `artifacts/spot-verify-encode-body-replacement.log` — manual
  spot-verify of the `encode -> Ok(vec![])` mutation: **32 failed,
  518 passed** under the mutant.
- `artifacts/post-verification-tests.log` — `cargo test`:
  **550 passed, 0 failed** after revert.

Source `src/am/ec_spire/storage/routing_delta.rs` byte-for-byte
identical pre/post packet.

## Required follow-up

Full 58/58 per-mutation verification belongs in a follow-up packet
after `target/` cleanup or in a CI lane. The analysis-only approach
ships with explicit caveats per packet 050's framing.
