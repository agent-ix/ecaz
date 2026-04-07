# Review Request: EXPLAIN Counter Scaffolding Snapshot

Scope:
- `src/am/explain.rs`
- `src/am/mod.rs`
- `src/am/shared.rs`
- `src/lib.rs`
- `spec/functional/FR-024-custom-explain.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Added a pure EXPLAIN-counter contract in `src/am/explain.rs` that defines the seven intended
  FR-024 counters, their Rust types, and the increment conditions each counter is meant to track.
- Added a read-only SQL/admin surface, `tqhnsw_explain_counter_snapshot()`, that exposes that
  staged counter contract while keeping both `scan_opaque_storage_ready` and
  `runtime_wiring_ready` explicitly false.
- Added Rust and pg coverage for the counter-contract surface without touching `am/scan.rs` or
  claiming any runtime storage exists yet.
- Reconciled FR-024 with implementation reality by replacing the stale claim that PG17 already has
  live `TqScanOpaque` counter fields with the current staged boundary: definitions are visible,
  storage and wiring are still pending.

Review focus:
- Whether a dedicated `tqhnsw_explain_counter_snapshot()` is the right D1 seam for the runtime lane
  to target later, or whether this contract should stay embedded only in FR-024 text
- Whether the current row shape is explicit enough for review/productization work without
  overspecifying scan internals too early
- Whether the FR-024 reconciliation is now honest about current implementation status while still
  preserving the intended long-term EXPLAIN contract
- Whether `scan_opaque_storage_ready` and `runtime_wiring_ready` are the right two readiness bits
  for this staging surface

Questions to answer:
- Should the EXPLAIN counter contract remain a dedicated snapshot once runtime wiring starts, or
  should it fold into a broader diagnostics surface at that point?
- Are the chosen counter names and increment-condition strings durable enough to expose now for
  cross-agent coordination?
- Does this change make the current gap between EXPLAIN counter definition and scan/runtime wiring
  clearer for the other agent?
