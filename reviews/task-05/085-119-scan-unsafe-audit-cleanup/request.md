# Request: Tighten Scan Unsafe Audit Helpers

Commit: `d469c5a`

Summary:
- Adds an explicit `SAFETY` comment to the raw-pointer split-borrow helper that simultaneously reads the visible frontier and mutates the bootstrap beam scheduler.
- Changes `tqhnsw_amendscan` to cast the opaque scan state once before the teardown calls instead of recreating several `&mut TqScanOpaque` references in sequence.

Files:
- `src/am/scan.rs`

Why this matters:
- The pass-3 review correctly identified two remaining unsafe-audit rough edges in the scan module.
- Neither was a runtime bug, but both made the current invariants less obvious than they should be during later ownership transfer work.
- This slice documents the non-aliasing assumption explicitly and makes teardown borrow structure easier to audit.

Review focus:
- Whether the new `SAFETY` comment states the real non-aliasing invariant clearly enough
- Whether the cast-once `amendscan` pattern now makes the teardown lifetime structure unambiguous
- Whether any nearby unsafe scan helpers still need the same kind of audit cleanup
