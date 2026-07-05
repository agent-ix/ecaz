# Request: Exhaustion Owns Result-State Clearing

Commit: `419caef`

Summary:
- move current-result teardown into `mark_scan_exhausted` in `src/am/scan.rs`
- remove the duplicated `result_state.clear()` calls from the linear selector's exhausted return sites
- add regression coverage that exhausting the scan clears both the current-result slot and pending duplicate-drain state

Please review:
- whether exhaustion is now the right single owner of result-state teardown
- whether any non-exhaustion path still relies on the old duplicated clear calls implicitly
- whether this makes the staged executor's phase-transition contract clearer without changing behavior
