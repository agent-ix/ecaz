# Triage: local_store_set.rs mutation campaign

Result: **19 mutations enumerated → 19 KILLED, 0 missed, 0 timed-out.**

No new tests required. The cumulative careful suite from packets
028-051 — particularly the
`local_object_store_set_routes_by_pid_and_reads_back_objects`,
`local_object_store_set_round_trips_non_leaf_object_kinds`,
`local_object_store_set_rejects_unconfigured_placements`,
`local_object_store_set_from_config_rejects_duplicate_local_store_id`,
`local_object_store_set_round_trips_leaf_v1`,
`local_object_store_set_object_reader_trait_routes_through_store_for_placement`,
and `local_object_store_trait_dispatch_covers_all_read_methods`
tests — already discriminates every operator swap and body
replacement in this file.

## Methodology fix

While running this campaign, a second bug was discovered in
`run_spire_mutations_v2.py`: the `BODY_REPLACE_RE` pattern used
`\S+` for the function name, which doesn't match
`<impl Trait for Type>::method` patterns (spaces inside angle
brackets). Changed to `(.+?)` (non-greedy) to allow spaces while
still anchoring at the first ` -> ` separator. After the fix all 19
mutations parsed and all 19 killed.

## Per-mutation map

All 19 mutations are body-replacements on the
`SpireLocalObjectStoreSet` impl and the two `SpireObjectReader`
impl blocks. Each gets killed by the round-trip tests above because:

- Replacing a `from_config` or `insert_*` body with
  `Ok(Default::default())` produces a placement whose fields don't
  match the inserted object; round-trip read fails on metadata
  mismatch.
- Replacing a `store_*` or `read_*` body similarly produces a
  Default value (empty / mismatched), failing the round-trip.

Verdicts in `artifacts/manual-verification.log`.

## Verification artifacts

- `artifacts/local-store-set-mutants-enumerated.txt` — full
  19-mutation enumeration.
- `artifacts/manual-verification.log` — **19 KILLED, 0 MISSED.**
- `artifacts/post-verification-tests.log` — `cargo test`:
  **550 passed, 0 failed** after revert.

Source `src/am/ec_spire/storage/local_store_set.rs` byte-for-byte
identical pre/post packet.
