## Feedback: Storage-Format REINDEX Insert/Vacuum Coverage

Read `test_tqhnsw_storage_format_switch_rejects_insert_until_reindex`
and `test_tqhnsw_storage_format_switch_rejects_vacuum_until_reindex`
in `src/lib.rs:8802+` / `:8824+`, and cross-referenced the
guardrail seam `graph::GraphStorageDescriptor::from_index_relation`
at `src/am/graph.rs:25–40`.

### What's right

- **Closes the exact test-coverage gap I raised on packet `403`.**
  Earlier feedback called out that scan had explicit pg coverage
  but insert and vacuum used the same seam without independent
  assertion. This packet adds one targeted test per path, same
  mismatch shape, same expected panic text. Smallest possible
  slice that proves the contract on all three entry paths.
- **Reuses the same mismatch construction across all three
  tests.** Build `turboquant`, `ALTER INDEX ... SET
  (storage_format='pq_fastscan')`, then hit the runtime path.
  Shared construction means if the guardrail message ever drifts,
  all three tests fail consistently — no test will pass through
  accidental inversion of the format pairing.
- **Vacuum test uses the real AM entry point.** The test drives
  vacuum through `am::debug_vacuum_remove_heap_tids(...)`, which
  is the actual vacuum path's open seam. A shallower test that
  called `from_index_relation` directly would prove the helper
  errors but not that vacuum actually reaches it. This test
  proves the routing.
- **No AM logic change.** Correctly scoped as test-only. A
  guardrail-coverage packet that also tweaked the guardrail would
  muddy the merge story.
- **Matches the exact panic text.** Both new tests assert the
  `REINDEX after switching formats` message. That locks the
  operator-facing contract in code: the error text is now a test
  invariant, not just an aspiration.

### Concerns

1. **These tests have never executed.** This is the single most
   important test packet on the entire 378–409 arc — it is the
   load-bearing proof that the storage-format guardrail works on
   every write path — and because no lane runs
   `cargo pgrx test pg17`, the tests are unexecuted source. If
   insert or vacuum has a subtle bug where `from_index_relation`
   is called but its error is swallowed or masked, these tests
   cannot catch it because they have never run. A guardrail is
   as strong as its most recently executed test.
2. **No happy-path companion.** Both tests assert mismatch
   rejection. Neither asserts that a matching reloption/metadata
   pair accepts insert and vacuum cleanly. The `393` round-trip
   tests cover happy-path build+insert+vacuum, but do so without
   touching the reloption surface, so a bug in
   `from_index_relation` that spuriously rejected matching pairs
   would be caught by `393` but not by these tests. Not a gap
   here, but worth naming the division of labor.
3. **Vacuum test deletes one row before the flip.** That
   exercises the "dead-tid removal" vacuum path, which is the
   one `debug_vacuum_remove_heap_tids` names. The other two
   vacuum phases (repair, finalize) also route through the same
   seam per `403`'s description, but are not separately tested.
   Probably fine — the seam is `from_index_relation` at the
   vacuum adapter open, and phases don't reopen — but a one-line
   comment noting "one phase-1 test suffices because all three
   phases share the adapter open" would document the reasoning.
4. **`test_tqhnsw_storage_format_switch_rejects_insert_until_reindex`
   uses a "normal heap insert" — is the AUTOVAC/AUTOANALYZE
   pathway covered?** If an autovacuum worker opens the index
   while the mismatch is present, does it hit the same guardrail
   or a different open? Probably the same — autovacuum and
   manual vacuum share the cleanup path — but the packet does
   not name this explicitly. Worth one sentence confirming the
   guardrail fires on background-worker opens.
5. **Linker gap is load-bearing here.** For most packets the
   "tests don't run locally" caveat is a general CI-discipline
   concern. For this packet it is the specific merge-readiness
   question: packet `405` cited `393` + `403`'s guardrail as
   task-15 landing evidence, and both now depend on tests that
   have never executed. The merge reviewer needs to know this.

### Observation

Right follow-up, wrong environment to prove it. The code changes
here are correct and minimal, the tests are well-shaped, and the
reused mismatch construction is the right call. But this is also
exactly the kind of test that needs to actually run to be worth
anything — "insert panics on metadata drift" is not a claim that
`cargo check` can validate. Of everything on the 378–409 arc,
this is the packet whose tests most need to execute before merge.
