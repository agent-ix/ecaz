# Triage: routing_delta.rs mutation analysis

Result: **58 mutations enumerated → 58 KILLED, 0 MISSED, 0 timeouts
(full per-mutation verification under isolated CARGO_TARGET_DIR,
plus one new boundary-killing test).**

## Methodology

**Full per-mutation verification** via
`/tmp/run_spire_mutations_v2.py` with
`CARGO_TARGET_DIR=$(pwd)/target-mutants` for build isolation (per
053/054/055 reviewer direction). Per-mutation cycles run in 3-10 s
under the isolated target-dir instead of 5-10 min under the shared
305 GB main target/.

## Per-mutation verdicts (58 total)

56 mutations KILLED by the existing cascade test surface
(round-trip + validate-rejects-*). 2 mutations on line 208:23
boundary check (`if tail.len() < ROUTING_OBJECT_BODY_PREFIX_BYTES`)
were initially MISSED because no existing test probed the
length-mismatch error path at the exact prefix boundary. Both
killed after adding one new test.

## New killing test

`routing_partition_object_decode_rejects_prefix_only_tail_with_length_mismatch`
in `src/am/ec_spire/storage/tests/vec_and_routing.rs` (and mirrored
in the careful crate via the existing `include!` chain). The test:

1. Encodes a valid `SpireRoutingPartitionObject::root` (2 children,
   2-dim centroids).
2. Truncates the encoded bytes to exactly
   `SPIRE_PARTITION_OBJECT_HEADER_BYTES + 4`, leaving the tail at
   exactly the prefix size (4 bytes: dimensions + reserved only).
3. Decodes and asserts the resulting error contains
   `"length mismatch"`.

Under the original code, the line-208 check passes (4 < 4 is false)
and the decoder reaches the `tail.len() != expected_tail_len` check
at line 233 which errors with `"length mismatch"`. Under either
mutant (`< -> ==` or `< -> <=`), the line-208 check fires early
and the error message is `"body too short"`, which fails the
`contains("length mismatch")` assertion.

Mutation cycles re-verified manually after the test was added:

| Mutant | Original | Mutant result |
| --- | --- | --- |
| `< -> ==` | 550 passed | **1 failed** (the new test) |
| `< -> <=` | 550 passed | **1 failed** (the new test) |

## Verification artifacts

- `artifacts/routing-delta-mutants-enumerated.txt` — full 58 enumeration.
- `artifacts/manual-verification.log` — 58/58 per-mutation verdicts.
- `artifacts/spot-verify-encode-body-replacement.log` — legacy
  spot-verify kept for chronology.
- `artifacts/post-verification-tests.log` — clean re-run after revert.

Source `src/am/ec_spire/storage/routing_delta.rs` byte-for-byte
identical pre/post packet (only the test file gained a new test).
