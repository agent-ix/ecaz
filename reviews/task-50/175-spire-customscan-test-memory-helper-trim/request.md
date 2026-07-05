# Review Request: SPIRE CustomScan Test Memory Helper Trim

## Scope

This packet reviews commit `292082f4d6f1d9d3055803eb3de844cc91583666` (`Trim SPIRE CustomScan test memory unsafe helpers`).

The slice narrows SPIRE CustomScan test-only memory instrumentation in `src/am/ec_spire/custom_scan/begin_exec.rs`.

## Unsafe Burndown

- Converted test-only note/no-op functions from `unsafe fn` to safe private helpers:
  - `custom_scan_note_memory_baseline_for_test`
  - `custom_scan_note_memory_after_end_for_test`
- Converted `custom_scan_memory_context_used_bytes_for_test` from `unsafe fn` to a private safe helper that retains the single internal unsafe FFI call to `MemoryContextMemConsumed`.
- The remaining unsafe is now the actual PostgreSQL memory-context FFI boundary, with the executor-captured MemoryContext invariant documented at the call site.

Unsafe ledger movement:

- previous packet 174 ledger: `1852`
- packet 175 ledger: `1850`
- net reduction: `2`

High-signal file count from `make unsafe-block-count`:

- `src/am/ec_spire/custom_scan/begin_exec.rs`: `22 -> 20`

## Validation

Packet-local artifacts are under `reviews/task-50/175-spire-customscan-test-memory-helper-trim/artifacts/`.

Passed:

- `cargo-check-pg18-bench.log`
- `cargo-check-pg18-pg-test.log`
- `git-diff-check.log`
- `unsafe-block-count.log`
- `unsafe-ledger-generate.log`
- `unsafe-ledger-check.log`

## Reviewer Focus

Please check that the only remaining memory-context unsafe in this slice is the irreducible test instrumentation boundary around PostgreSQL `MemoryContextMemConsumed`, and that making the note helpers safe does not expand production behavior.
