# Review Request: SPIRE non-test unwrap/expect audit

## Summary

This checkpoint closes the Phase 12b.7 non-test `unwrap()` / `expect()` audit.

The code checkpoint removes two avoidable `expect()` hits from non-test scan
paths:

- `src/am/ec_spire/scan/relation.rs`: remote-placement diagnostic fallback now
  returns an explicit internal consistency error instead of panicking.
- `src/am/ec_spire/scan/candidates.rs`: bounded vec-id dedupe eviction now has
  an explicit empty-pop fallback instead of `expect("peeked live worst")`.

Code checkpoint: `d216f142151d4b989c13e6d7b083e844e3d1d0c5`

## Audit Result

Final command:

```text
rg -n '\.unwrap\(\)|\.expect\(' src/am/ec_spire --glob '!**/tests*'
```

Final total: 114 non-test hits.

The remaining hits are classified as accepted category (a) invariants in
`artifacts/classification.md`. The packet records zero remaining category (b)
and zero category (c) hot-path panic risks on remote-supplied data.

## Validation

- `cargo test -p ecaz local_heap_delivery_gate_blocks_remote_placements`
- `cargo test -p ecaz rank_routed_leaf_rows_by_ip_keeps_bounded_best_deduped_candidates`
- `cargo fmt --check`

The focused test logs are packet-local artifacts. `cargo fmt --check` passed
with the repository's existing stable-rustfmt warnings about unstable import
options.

Raw logs and command metadata are in `artifacts/manifest.md`.

## Reviewer Focus

- Confirm the accepted category (a) groups are defensible.
- Confirm the two fixed hits were the only category (b) candidates needed for
  this pass.
- Confirm category (c) remains zero after reviewing the final inventory.
