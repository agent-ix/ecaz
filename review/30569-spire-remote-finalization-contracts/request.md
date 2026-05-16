# Review Request: SPIRE remote finalization contracts

## Summary

Code checkpoint: `d71371ac` (`Expose SPIRE remote finalization contracts`)

This slice adds the last diagnostic boundary before a future libpq fanout executor returns heap rows:

- `ec_spire_remote_search_row_locator_contract()` documents the row-locator contract as origin-node scoped, coordinator-opaque bytes, validated as nonempty on receive, and unresolved until origin-node heap lookup exists.
- `ec_spire_remote_search_finalization_summary(...)` projects whether merged candidates are locally finalizable, blocked by missing remote descriptors, or blocked by deferred remote heap resolution.
- Feedback follow-ups from the 30556-30568 review stack:
  - validate published snapshots before the empty fanout-plan return
  - render local/remote fanout placement state from the validated placement lookup instead of hardcoding `"available"`
  - derive libpq result-column count from `remote_search_libpq_result_contract_rows().len()`
  - fix the merge-input `tie_breaker` diagnostic string to match the comparator ordering
  - move the global vec-id merge precondition rustdoc back onto the merge helper and give the receive validator its own contract docs
  - add stale-placement fail-closed coverage for degraded remote search
  - add a combined local/remote/skipped target-readiness precedence fixture
  - document the intentional coordinator-local top-k and remote-version diagnostic asymmetries

## Files

- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/hierarchy_snapshots.rs`
- `src/am/ec_spire/root/snapshots.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib test_ec_spire_remote_search_target_readiness_mixed_precedence --no-default-features --features pg18`
- `cargo test --lib remote_search --no-default-features --features pg18`
  - 34 passed; 0 failed; 1399 filtered out
- `git diff --check`

## Notes

No measurement artifacts are included; this packet makes only contract and validation claims.
