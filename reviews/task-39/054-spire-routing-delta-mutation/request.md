# Task 39 / 054 — SPIRE routing_delta.rs mutation campaign (full verification)

## Goal

Ninth slice of the reviewer-prescribed SPIRE storage mutation
cascade. Drive every mutation in
`src/am/ec_spire/storage/routing_delta.rs` to **0 missed /
0 timeouts**.

## Result

**58 mutations enumerated → 58 KILLED, 0 MISSED, 0 timeouts.**

Full per-mutation verification under
`CARGO_TARGET_DIR=$(pwd)/target-mutants` (per reviewer direction).
The two boundary mutations on line 208:23 (`< -> ==` and `< -> <=`
on the `tail.len() < ROUTING_OBJECT_BODY_PREFIX_BYTES` check) were
initially MISSED — the existing test surface had no test that
exercised the length-mismatch error path at the exact prefix-only
tail boundary. Both are killed after adding one new test
(`routing_partition_object_decode_rejects_prefix_only_tail_with_length_mismatch`).

## Methodology

Full per-mutation apply/test/revert via
`/tmp/run_spire_mutations_v2.py` with
`CARGO_TARGET_DIR=$(pwd)/target-mutants` build isolation.

## Code change

- `src/am/ec_spire/storage/tests/vec_and_routing.rs`: added one
  new boundary-killing test.
- `src/am/ec_spire/storage/tests.rs`: imported
  `SPIRE_PARTITION_OBJECT_HEADER_BYTES`.
- `hardening/careful/src/spire.rs`: mirrored the import in the
  careful crate's `include!` block.

Source `routing_delta.rs` unchanged.

## Validation

Artifacts under `reviews/task-39/054-spire-routing-delta-mutation/artifacts/`:

- `routing-delta-mutants-enumerated.txt` — full 58 enumeration.
- `manual-verification.log` — 58/58 per-mutation verdicts.
- `post-verification-tests.log` — clean re-run after revert.

`triage.md` documents the killing-test rationale and the mutant
re-verification after the test was added.
