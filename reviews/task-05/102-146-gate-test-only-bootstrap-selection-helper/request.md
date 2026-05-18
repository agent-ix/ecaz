# Request: Gate Test-Only Bootstrap Selection Helper

Commit: `037a785`

Summary:
- gate `select_next_bootstrap_candidate` in `src/am/scan.rs` to `#[cfg(test)]`
- keep runtime bootstrap selection on the refill-aware helper path only
- shrink one more non-runtime bootstrap helper out of the production scan surface

Please review:
- whether any production path still depends on the non-refill bootstrap selection helper
- whether the refill-aware bootstrap selector now reads as the only real runtime bootstrap-selection contract
- whether this meaningfully tightens the runtime vs test boundary in `src/am/scan.rs`
