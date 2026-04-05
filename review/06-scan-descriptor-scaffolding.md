# Review Request: Scan Descriptor Scaffolding

Scope:
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- `ambeginscan` now allocates a real PostgreSQL `IndexScanDesc` for `tqhnsw`.
- The AM attaches a small opaque scan-state struct to that descriptor.
- `amendscan` frees that opaque state instead of hard-erroring.
- Actual scan execution is still unsupported; this slice only establishes descriptor lifecycle.

Review focus:
- Scan descriptor ownership and cleanup
- Memory-context safety of the opaque scan state
- Whether the current begin/end behavior matches PostgreSQL AM expectations closely enough for a narrow groundwork slice

Questions to answer:
- Is the descriptor/opaque lifecycle correct under normal executor cleanup?
- Is there any double-free or leak risk in the current `amendscan` plus `IndexScanEnd` split?
- Is there any smaller or safer way to stage scan groundwork here?
