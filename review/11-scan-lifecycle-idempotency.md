# Review Request: Scan Lifecycle Idempotency

Scope:
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- Scan scaffolding is in place for `ambeginscan`, `amrescan`, `amgettuple`, and `amendscan`.
- Review comments left one small remaining testability gap around lifecycle idempotency:
  - repeated `amendscan` on the same descriptor
  - whether any additional scan-lifecycle cases should be marked not needed at the current capability boundary

Review focus:
- Whether the current cleanup and guard logic is already sufficient without widening scan execution
- Whether an idempotency test would provide meaningful confidence or just restate defensive null checks
- Whether the remaining scan review comments should now be closed as not needed

Questions to answer:
- Is there any actionable lifecycle bug left in the current scan scaffolding?
- Is repeated-`amendscan` coverage the next smallest worthwhile slice?
- Should the remaining `amgettuple` repeated-rescan note stay deferred until real tuple production exists?

Status at `41cfdfa`:
- Addressed by adding repeated-`amendscan` regression coverage.
- Remaining `amgettuple` repeated-rescan coverage is not needed for now because scan execution still fails at the capability boundary.
