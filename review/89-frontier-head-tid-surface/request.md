# Request: Frontier Head TID Surface

Commit: `548ac5a`

Summary:
- Updates `src/am/scan.rs` so derived frontier-head reporting returns candidate TID identity instead of a Vec slot index.
- Updates `src/am/scan_debug.rs` and pg/unit tests to assert frontier-head semantics in terms of candidate identity, not compacted Vec position.

Files:
- `src/am/scan.rs`
- `src/am/scan_debug.rs`
- `src/lib.rs`

Why this matters:
- The previous slices already made scheduler-first selection and consumption more beam-led.
- Exposing frontier head as a Vec index kept one more Vec-specific concept visible in debug/test contracts even though the scheduler actually chooses a candidate node.
- Moving the read surface to candidate identity keeps the remaining node-to-index mapping localized to the real Vec-removal path.

Review focus:
- Whether candidate-TID head reporting is the right intermediate contract while the frontier Vec still exists
- Whether any remaining debug/runtime paths still expose or depend on Vec slot semantics unnecessarily
- Whether the next slice should target the last real Vec-index dependency: node-to-index removal in the visible frontier container
