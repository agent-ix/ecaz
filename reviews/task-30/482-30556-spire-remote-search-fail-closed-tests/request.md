# Review Request: SPIRE Remote Search Fail-Closed Tests

- Code commit: `0239f5bf` (`Add SPIRE remote search fail-closed tests`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 7 remote search followup
- Agent: coder1

## Summary

This checkpoint addresses the one item intentionally deferred in the 30555
feedback: fail-closed PG18 coverage for `ec_spire_remote_search` before the
next Phase 7 libpq fanout slice.

Changes:

- adds `test_ec_spire_remote_search_mode_mismatch`, which builds a normal
  strict SPIRE index and verifies a remote-search call requesting `degraded`
  mode fails with the active-epoch consistency mismatch error;
- adds `test_ec_spire_remote_search_strict_unavailable_leaf`, which rewrites a
  selected leaf placement to `Unavailable` through a test-only debug helper and
  verifies strict remote search fails closed;
- adds `debug_spire_rewrite_placement_state`, available only under test /
  `pg_test`, to republish active manifest bytes with a modified placement state
  for fail-closed SQL fixtures;
- exports the helper through the test-only AM debug surface.

## Files

- `src/am/ec_spire/root/debug.rs`
- `src/am/mod.rs`
- `src/lib.rs`

## Review Focus

1. Check that the test-only placement rewrite helper remains fenced behind
   `#[cfg(any(test, feature = "pg_test"))]`.
2. Check that the helper intentionally bypasses publish validation only for
   constructing malformed/fail-closed fixtures.
3. Check that the two new PG tests cover the reviewer-requested surfaces:
   consistency-mode mismatch and strict unavailable selected placement.

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote_search --no-default-features --features pg18`
  - Result: passed; 3 tests passed, including the two new should-panic cases.
- `git diff --check`
