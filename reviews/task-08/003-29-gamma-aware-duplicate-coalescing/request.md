# Review Request: Gamma-Aware Duplicate Coalescing

Scope:
- `src/am/mod.rs`
- `src/lib.rs`
- `spec/adr/ADR-010-gamma-aware-duplicate-coalescing.md`

What changed:
- Build-time duplicate coalescing now matches on `(gamma, code_bytes)` instead of code bytes alone.
- Live `aminsert` duplicate coalescing now, on a same-code match, reads the representative heap row and compares its persisted `gamma` before deciding to coalesce.
- Regression coverage now verifies that same-code tqvectors with distinct persisted `gamma` values stay separate during both build and live insert.

Review focus:
- Whether matching duplicates on `gamma` plus code bytes is the right correctness boundary for persisted tqvectors
- Whether the representative heap-row fetch in live `aminsert` is safe and acceptable as an interim boundary
- Whether the new regression coverage is sufficient for the corrected duplicate semantics

Questions to answer:
- Is using the first stored heap TID as the representative `gamma` source the right current contract for a coalesced element?
- Is `SnapshotAny` acceptable for this duplicate check, or should the heap fetch use a different visibility boundary?
- Should the next storage slice persist `gamma` in element tuples now that duplicate semantics depend on it?
