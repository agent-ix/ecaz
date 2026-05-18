# Review Request: ReadStream Callback Signature Scaffolding

Scope:
- `src/am/stream.rs`
- `spec/functional/FR-019-async-io-read-stream.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Added pure `GraphPrefetchState` and `LinearPrefetchState` types in `src/am/stream.rs` to model
  the intended PG18 ReadStream callback state without touching `scan.rs` or wiring any runtime I/O.
- Added pure callback-signature helpers for the intended graph and linear callbacks, including the
  stream modes, access patterns, state-carrier types, and `InvalidBlockNumber` end-of-stream
  sentinel.
- Added unit tests that verify both callback signatures and the sequential/exhaustion behavior of
  the two state carriers.
- Updated FR-019, the test matrix, and Task 11 notes to record this as reviewer-directed D1
  callback scaffolding rather than another SQL snapshot surface.

Review focus:
- Whether these pure `am/stream.rs` types are the right remaining D1 seam before any PG18 binding
  or scan wiring exists
- Whether `GraphPrefetchState` and `LinearPrefetchState` capture the intended callback contract
  cleanly without overspecifying the eventual runtime implementation
- Whether this pivot away from expanding `tqhnsw_read_stream_snapshot()` addresses the reviewer's
  snapshot-proliferation concern while still making real progress on FR-019
- Whether the current state-carrier and callback-signature APIs are suitable for the other agent to
  embed later when the runtime lane is ready

Questions to answer:
- Is `Option<u32>` the right pure representation for callback exhaustion at this stage, with
  `InvalidBlockNumber` kept as a documented PG18 binding detail?
- Are there any missing fields that the eventual PG18 callback bindings will clearly need in the
  state carriers?
- Does this slice strike the right balance between tangible D1 progress and avoiding more
  low-value SQL scaffolding?
