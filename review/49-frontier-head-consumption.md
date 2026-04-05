# Review Request: Frontier Head Consumption

Scope:
- `src/am/scan.rs`
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- Added one explicit helper that consumes the current head of the fixed two-slot candidate frontier, clears that slot, and recomputes the next head under the existing ordering rule.
- Added a narrow debug helper that snapshots the frontier before consumption, after one consumption step, and after a second step drains the remaining slot.
- Added regression coverage that consuming the current head either reselects the remaining valid slot or clears the frontier when no valid candidate remains.

Review focus:
- Whether explicit head consumption is the right next traversal-groundwork seam before adding a larger frontier or visited state
- Whether clearing only the consumed slot and lazily recomputing the next head is the right invariant for this fixed-width frontier stage
- Whether the helper and tests stay narrow enough to avoid implying a full traversal loop exists already

Questions to answer:
- Is it correct for head consumption to return the consumed slot while leaving the other slot untouched?
- Is recomputing the head from the remaining valid slot sufficient groundwork before a real queue or heap exists?
- Are there any missing edge cases around consuming an already-empty frontier or double-consumption that should be covered now?
