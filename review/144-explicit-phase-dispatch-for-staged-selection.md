# Request: Explicit Phase Dispatch For Staged Selection

Commit: `9c9d074`

Summary:
- rewrite `select_next_scan_result` in `src/am/scan.rs` to dispatch explicitly on `ScanExecutionPhase`
- keep behavior the same while making phase-dependent selection clearer:
  - `Bootstrap`: try bootstrap selection, then fall through to linear on the same call if bootstrap completes without selecting
  - `Linear`: select from linear scan
  - `Exhausted`: return none

Please review:
- whether the explicit phase match makes the bootstrap-to-linear transition easier to reason about without changing semantics
- whether the current same-call fallthrough from completed bootstrap into linear remains the right contract
- whether this is the right base for the next step toward a single ordered executor loop
