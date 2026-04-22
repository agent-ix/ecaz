# Review Request: codify the Task 17 handoff contract

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `plan/tasks/17-diskann-access-method.md`
- `plan/handoff-task17-isolated.md`
- `review/11084-task17-status-refresh/feedback/2026-04-21-01-reviewer.md`

## What this packet is

This is the docs/signoff response slice for the remaining reviewer feedback on
packet `11084`.

Packet `11085` handled the AM-local unit-norm blocker. This packet closes the
task-file and handoff blockers the reviewer called out:

1. stale task-17 DoD rows still referred to missing `scripts/*.sh` scratch
   artifacts
2. baseline recall/latency numbers only existed in packet prose
3. the handoff note did not tell coder-1 exactly what to measure on the faster
   machine
4. ADR-034 PROPOSED → ACCEPTED was still implicit instead of being called out
   in the remaining signoff checklist

## What changed

### `plan/tasks/17-diskann-access-method.md`

Added a durable baseline block under the current-status section with the exact
real-10k numbers that coder-1 should compare against on the faster machine:

- packet `11078` clean real-10k sweep at
  `graph_degree=32`, `build_list_size=100`, `alpha=1.2`,
  `list_size ∈ {64,128,200,400,800}`
- packet `11083` pre/post-vacuum local smoke at `list_size=128`

The same file now also:

- calls out the ADR-034 ACCEPTED flip explicitly in the remaining signoff list
- marks non-unit corpora as a V0 out-of-scope follow-up, pointing at the new
  build/insert warnings from packet `11085`
- replaces the stale phase-7 / phase-8 scratch-script rows with the concrete
  `pg_test_ec_diskann_*` coverage that now stands in for callback-surface
  insert/vacuum correctness on this branch
- updates the Definition of Done to reference that on-branch pg_test matrix
  instead of non-existent `scripts/diskann_*_scratch.sh` files

### `plan/handoff-task17-isolated.md`

Added an explicit "Measurement contract for coder-1" section that names:

- profile: `ec_diskann`
- reloptions: `graph_degree=32`, `build_list_size=100`, `alpha=1.2`
- sweep grid: `list_size=64,128,200,400,800`
- baseline fixture order: real 10k first, then the remaining 50k `CREATE INDEX`
  smoke if the faster machine is clean
- success criteria: `Recall@10 >= 0.90` floor, `~0.95` preferred, plus latency
  comparisons against `ec_hnsw` on matched tuning
- next closing step: queue the ADR-034 ACCEPTED flip if the faster-machine run
  is clean

That turns the handoff from "keep working in signoff territory" into a concrete
measurement checklist.

### `review/11084-task17-status-refresh/feedback/2026-04-21-01-reviewer.md`

Tracked the reviewer feedback file in-branch so the branch state now contains
the note this packet responds to.

## Why this slice

- It closes the remaining reviewer blockers without inventing more DiskANN AM
  code work.
- It makes the faster-box signoff criteria durable in the task file instead of
  scattering them across packet prose.
- It removes stale references to `scripts/` from the live task-17 closure path,
  which matters on this branch because `ecaz-cli` replaced that surface and
  `scripts/` is being deleted separately.

## Validation context

Docs only. No code changed in this packet.

The measurements and runtime claims now lifted into the task file are backed by
the existing task-17 packets:

- `review/11078-task17-diskann-real-recall-recovery/`
- `review/11081-task17-diskann-vacuum-recall/`
- `review/11083-task17-diskann-post-vacuum-smoke/`
- `review/11084-task17-status-refresh/feedback/2026-04-21-01-reviewer.md`

## Follow-ups intentionally not in this packet

- Any new DiskANN AM code changes.
- Any new benchmark artifacts beyond the numbers already captured in `11078`
  and `11083`.
- The actual faster-machine benchmark run. This packet only defines the
  contract for it.
