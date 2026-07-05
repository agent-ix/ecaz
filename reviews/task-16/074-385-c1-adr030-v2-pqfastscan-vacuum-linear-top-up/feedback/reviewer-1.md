## Feedback: PqFastScan Vacuum Linear Top-Up

Read `LinearRepairPlanner` at `src/am/vacuum.rs:248`,
`collect_linear_repair_candidates_on_page` at `:1182`, and
`load_grouped_rerank_payload_for_linear_repair_candidate` at `:1284`.

### What's right

- **Closes the 383 asymmetry cleanly.**
  `plan_repair_replacement(...)` now always builds a
  `LinearRepairPlanner`, so grouped and scalar repair fall back the
  same way when graph search yields too few candidates. This is
  the half of vacuum parity that 383 left on the table.
- **Same-block rerank reads avoid relock.** The helper at `:1284`
  decodes the rerank tuple in-place when `rerank_tid.block_number
  == block_number`. That's the right optimization — linear top-up
  already holds the buffer, and re-entering the buffer manager for
  a tuple on the same page would be pointless overhead and a
  potential self-deadlock risk.
- **Fallback to the cross-block helper is explicit.** When the
  rerank tuple is on another block the code falls through to
  `graph::load_grouped_rerank_payload(...)`. That's the clean
  split: one path for "I already hold the page," one path for the
  general case.
- **Planner now carries `GraphStorageDescriptor`, not `code_len`.**
  That's consistent with the 381 seam and makes the linear path
  self-consistent with graph-search repair.

### Concerns

1. **`lp_flags == 0` on same-page rerank tuple is a hard error.**
   At `:1301-1307` an unused rerank slot is a `pgrx::error!`. Under
   vacuum's page-level exclusive lock this shouldn't race, but it
   couples the linear-scan path tightly to the in-memory page being
   internally consistent. If a prior vacuum finalized the rerank
   tuple without finalizing the hot tuple (shouldn't happen, but
   worth asserting), this becomes a crash path. A one-line
   invariant comment near the error would help.

2. **No upper-layer grouped replacement coverage.** The pg test
   covers layer-0 replacement only. Upper layers have smaller
   candidate pools and are more likely to hit the linear fallback,
   so this is the case most in need of explicit coverage. Packet
   lists it as a followup — worth not sliding that before merge.

3. **Double-decode on grouped linear candidates.** Every grouped
   candidate now decodes the hot tuple *and* loads the rerank
   payload just to check if the candidate is live/at-the-right-
   level. For a linear scan over many pages this is 2x the work
   of the scalar path. The early-continue checks at `:1272` do
   filter post-construction, but the rerank load happens first.
   Worth a future optimization: lift the liveness check into the
   hot tuple decode path and only load rerank when the candidate
   will actually be scored.

4. **Linker gap.** The new grouped layer-0 repair pg test is the
   load-bearing proof. It did not run locally.

### Observation

This packet is where grouped vacuum reaches actual scalar parity.
383 without 385 is under-built. With both, grouped vacuum has the
same repair-quality properties as scalar vacuum modulo the
upper-layer coverage gap.
