# SPIRE Root Control Scan Cache

## Checkpoint

- Code commit: `62c5a125` (`Cache SPIRE root control in scan opaque`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: reviewer F6 follow-up for repeated `amrescan`

## Summary

This checkpoint caches `SpireRootControlState` in `SpireScanOpaque` for the scan descriptor lifetime. `clear_scan_work` still resets per-query state and candidate output, but it preserves the cached root/control state so repeated rescans on the same descriptor do not re-pin and re-read block 0.

The cache is deliberately narrow:

- first rescan reads root/control and stores it in the opaque,
- later rescans reuse the cached state,
- future snapshot-loading code can invalidate the cache when it observes a newer `active_epoch`.

## Changed Files

- `src/am/ec_spire/scan.rs`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable `imports_granularity` / `group_imports`.
- `cargo test --lib scan_opaque_clear_scan_work_drops_rescan_state --no-default-features --features pg18`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1068 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `188 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## Notes

- This addresses the current empty-index `amrescan` behavior from reviewer F6. Populated snapshot loading can tighten invalidation once active epoch transitions are observable in live scan state.
- No measurement artifacts are included; this checkpoint makes no benchmark or recall claim.
