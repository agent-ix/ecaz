# Task 39 / 052 — SPIRE local_store_set.rs mutation campaign

## Goal

Seventh slice of the reviewer-prescribed SPIRE storage mutation
cascade. Drive every mutation in
`src/am/ec_spire/storage/local_store_set.rs` to **0 missed / 0
timeouts**.

## Result

**19 mutations enumerated → 19 KILLED, 0 missed, 0 timed-out, 0
equivalent.**

No new tests required. The cumulative careful suite from packets
028-051 (especially the seven `local_object_store_set_*` tests
shipped across 031, 036, and 044) already discriminates every
operator swap and body replacement on this file.

## Code change

None to test code or production code. One bug fix landed in the
verification helper at `/tmp/run_spire_mutations_v2.py` (also copied
to `reviews/task-39/049-spire-helpers-mutation/artifacts/run-spire-mutations.py`
when shipped) so the cascade can parse `<impl Trait for Type>::method`
mutation lines correctly — see below.

## Methodology fix

`BODY_REPLACE_RE` used `\S+` for the function-name capture, which
stops at the first space. cargo-mutants emits trait-impl body
mutations as
`replace <impl Trait for Type>::method -> Result<...> with ...`,
where spaces inside the `<...>` cause the regex to fail and the
mutation to be silently skipped.

Fix: change to `(.+?)` (non-greedy, anchored at the first ` -> `
separator). All 19 mutations parsed and verified after the fix.

The previous packets in the cascade (047-051) didn't surface this
because the affected files had no trait-impl methods directly in
the source file (or their `impl` bodies were on inherent methods,
not on `impl Trait for Type`). `local_store_set.rs` has two such
impl blocks (`impl SpireObjectReader for SpireLocalObjectStoreSet`
and `impl SpireObjectReader for SpireLocalObjectStore`); 12 of its
19 mutations are on those blocks.

## Validation

Artifacts under `reviews/task-39/052-spire-local-store-set-mutation/artifacts/`:

- `local-store-set-mutants-enumerated.txt` — full 19-mutation
  enumeration.
- `manual-verification.log` — **19 KILLED, 0 MISSED.**
- `post-verification-tests.log` — `cargo test --manifest-path
  hardening/careful/Cargo.toml --lib`: **550 passed, 0 failed**
  (unchanged from packet 051; no new tests this packet).

Source `src/am/ec_spire/storage/local_store_set.rs` byte-for-byte
identical pre/post packet.
