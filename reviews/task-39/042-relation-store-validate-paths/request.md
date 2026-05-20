# Task 39 / 042 — Relation Store Validate-Path Coverage

## Goal

Add the two relation-store reader error branches that don't need a
crafted-bytes test fixture: the non-local `node_id` rejection in
`validate_local_available_placement` and the length-mismatch branch
inside `with_single_tuple_object_bytes`. Each is reachable by
inserting a real object and then handing the reader a mutated
`SpirePlacementEntry`.

## Code Change

`hardening/careful/src/spire.rs` — 2 new tests in `storage::tests`:

- `relation_store_with_single_tuple_object_bytes_rejects_length_mismatch`
  — inserts a routing object, mutates `placement.object_bytes`, and
  asserts the reader rejects the mismatch.
- `relation_store_validate_placement_rejects_non_local_node_id` —
  asserts the non-local node_id path in
  `validate_local_available_placement`.

## Baseline Ratchet

| File | Pre-packet | This packet |
| --- | ---: | ---: |
| `am/ec_spire/storage/relation_store.rs` | 58.10 | **58.52** |

The remaining gap to the 80% target is dominated by raw-byte-corruption
branches (chain segment-no/byte-base mismatch, V2 chain meta missing,
trailing-locator) and the PG18-only read-stream loop in
`prefetch_relation_blocks_with_read_stream`. Those are not reachable
without either an emulator extension that writes raw tuple bytes
inconsistent with placement metadata or a richer PG18 read-stream
shim; both are tracked as named follow-ups.

## Validation

Artifacts under
`reviews/task-39/042-relation-store-validate-paths/artifacts/`:

- `validate-paths-focused-tests.log` — **513 passed**.
- `coverage/summary.txt` + JSON.
- `coverage-delta-check.log` — relation_store green at new baseline.
- `coverage-baseline-check.log` — 42 critical paths complete.
