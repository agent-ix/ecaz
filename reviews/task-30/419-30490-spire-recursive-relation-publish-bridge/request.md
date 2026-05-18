# Review Request: SPIRE Recursive Relation Publish Bridge

Head SHA: `f6d1c42e`

## Summary

This checkpoint adds the relation-side publish bridge for recursive routing
epoch drafts.

The bridge:

- writes placement directory entries to the index relation;
- rebuilds the object manifest from placement-write evidence so manifest entries
  point at durable placement-entry TIDs rather than object tuple TIDs;
- encodes and writes the manifest bundle; and
- installs root/control state through the existing publish coordinator path.

A pure test covers the critical manifest handoff by building a recursive epoch
draft, substituting fake durable placement-entry TIDs, and verifying the encoded
object manifest/root-control state uses those durable locators.

## Files

- `src/am/ec_spire/build.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test recursive_ -- --nocapture`
  - 26 passed, including `recursive_epoch_relation_publish_input_uses_durable_placement_manifest`.
- `git diff --check`

No PG18 SQL test was run yet. This is the relation publish bridge, but it is not
wired to live `ambuild` behavior in this checkpoint.

## Review Focus

- Confirm the object-manifest rewrite after placement-entry writes mirrors the
  existing single-level relation publish contract.
- Confirm installing root/control state through the shared publish coordinator is
  sufficient before the final live recursive `ambuild` smoke wiring.
