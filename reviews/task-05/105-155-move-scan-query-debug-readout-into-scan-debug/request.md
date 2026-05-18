# Request: Move Scan Query Debug Readout Into Scan Debug

Commit: `4d24efe`

Summary:
- move the query readout helper from `src/am/scan.rs` into `src/am/scan_debug.rs`
- keep the same debug behavior by reading `TqScanOpaque` query state directly in the debug module
- remove one more debug-only helper from the production scan module surface

Please review:
- whether any non-debug path still depends on the old `read_scan_query` helper from `src/am/scan.rs`
- whether reading the query state directly in `src/am/scan_debug.rs` is the right boundary for this debug-only behavior
- whether this is a meaningful production-surface reduction in `src/am/scan.rs`
