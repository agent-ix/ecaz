## Feedback: Storage Format REINDEX Guardrail

Read `src/am/graph.rs` (`from_index_relation` at :25–40,
`matches_storage_format` at :128–134, `storage_format_name` at
:121–126), `src/am/options.rs::StorageFormat::as_str`, and the
routing in `src/am/scan.rs`, `src/am/insert.rs`, `src/am/vacuum.rs`,
`src/am/shared.rs`, `src/am/scan_debug.rs`. Checked the pg test
added in `src/lib.rs`.

### What's right

- **Closes the exact footgun flagged across packets `378` and `387`.**
  Both earlier feedback rounds called out that
  `ALTER INDEX ... SET (storage_format = ...)` without `REINDEX` would
  produce a split-brain index where reloption and metadata disagree
  and runtime silently preferred one. This packet converts that silent
  failure mode into a direct, actionable panic with the fix in the
  error message. That is the right resolution.
- **`from_index_relation` is the right central seam.** It composes
  `from_metadata` (the authoritative on-disk decode) with a live
  reloption read, and the mismatch check is a single small helper
  (`matches_storage_format`) that pattern-matches the descriptor
  against the enum. One place to extend when new formats land.
- **Every runtime open path routed through the helper.** Ordered
  scan, live insert, vacuum, the tuple-counting used by vacuum stats,
  and the grouped debug scan validation all now go through
  `from_index_relation`. That is every path I would expect — a
  mismatch cannot sneak past by opening through some alternate
  entrypoint. Verified that the metadata-only unit tests still use
  `from_metadata` directly where no live relation is available,
  which is correct.
- **Error message tells the operator the fix.** `REINDEX after
  switching formats` is literally the remediation; the operator
  doesn't have to go find the README first. That matches the tone
  the `387` reviewer feedback asked for.
- **Negative pg coverage proves the integration.** The `src/lib.rs`
  test builds `turboquant`, flips only the reloption to
  `pq_fastscan`, forces an ordered scan, and asserts the explicit
  `REINDEX` panic text. That is the load-bearing integration test
  for this seam — without it, the whole guardrail could regress to
  "error path not reachable" without clippy or `cargo check`
  noticing.
- **Correctly scoped.** The packet explicitly does not reject
  `ALTER INDEX ... SET` itself, does not attempt to rewrite on
  format switch, and does not change wire tags. A noisier slice
  that tried to block `ALTER` would have conflated a policy change
  with the guardrail landing.

### Concerns

1. **Runtime cost of reloption read on every open path not
   characterized.** `options::relation_options(index_relation)`
   runs on every scan init, insert adapter resolve, vacuum adapter
   resolve, and tuple-count call. That's a `pg_class` relcache
   touch + reloption decode on the hot path for every tuple-write.
   Likely negligible — it is a relcache lookup, not a heap scan —
   but the packet doesn't quantify it, and ADR030 measurement
   work is latency-sensitive. A single before/after microbench
   on insert / scan setup would close this question.
2. **No coverage of the insert/vacuum mismatch paths.** The pg
   test asserts mismatch-during-ordered-scan. The same mismatch
   reaching insert or vacuum adapter resolution would also panic,
   but that is load-bearing behavior — if someone refactored the
   vacuum or insert open path to bypass the seam, the scan-only
   test would still pass. At minimum a smoke test that attempts
   an insert after a reloption-only flip, and the same for
   vacuum, would lock the contract across all three adapters.
3. **No explicit test for the "reloption matches metadata"
   happy path.** All the routing unit tests exercise paths that
   call `from_index_relation`, so the success path is implicitly
   covered, but a direct assertion that a freshly built
   `pq_fastscan` (and a freshly built `turboquant`) opens
   without error would make the guardrail self-evidently
   symmetric — not just a one-sided rejection test.
4. **Error surfaced via `String` + panic chain.** The pattern
   matches the rest of the AM, but the mismatch error is an
   operator-facing condition that would benefit from a proper
   pgrx `ereport(ERROR)` with a hint field, so it renders
   cleanly in `psql` instead of as a backend crash-style
   message. Out of scope for this slice, worth a follow-up
   packet once the AM converges on an error-reporting pattern.
5. **Linker gap unchanged.** The new pg test in `src/lib.rs` is
   proven locally only by `cargo check --tests` + clippy. The
   load-bearing integration assertion here is exactly the kind
   of test that CI needs to run pgrx-linked; before merge, the
   CI lane's pass/fail on this specific test should be named
   explicitly in the merge packet.

### Observation

This is the single most merge-gating fix on the arc after the
canonical real-corpus runs. Without it, the first time an operator
hits `ALTER INDEX ... SET (storage_format = ...)` on production
they get silent scoring/storage disagreement; with it they get a
one-line error that tells them what to do. The seam shape
(`from_index_relation` composed from `from_metadata` + a
reloption read) is also the right place for future format-level
invariants to land. Ship this.
