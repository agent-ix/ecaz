# Review Request: refresh Task 17 status and handoff docs

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `plan/tasks/17-diskann-access-method.md`
- `plan/handoff-task17-isolated.md`

## What this packet is

This is a task-17 signoff/hygiene packet, not a new DiskANN AM code slice.

After packets `11078`, `11081`, `11082`, and `11083`, the branch status had
changed materially:

- real-corpus recall recovered
- the local post-delete vacuum path now completes
- the remaining work is review / merge / faster-machine signoff

But the task file still described the branch as of `2026-04-20`, and the
handoff note still described the old isolated `adr034-diskann-access-method`
branch from before the rebased callback buildout landed. That was stale enough
to misdirect the next session.

## What changed

### `plan/tasks/17-diskann-access-method.md`

Refreshed the top-level status from `2026-04-20` to `2026-04-21` and added a
new recovery/signoff subsection that points at the task-17 packets that now
matter operationally:

- `11078` for real-corpus recall recovery
- `11081` / `11082` for vacuum-runtime fixes
- `11083` for the completed local pg18 post-vacuum smoke

The file now states the current truth explicitly: missing callback work is no
longer the bottleneck; final review and faster-machine benches are.

### `plan/handoff-task17-isolated.md`

Replaced the stale isolated-branch handoff with a current branch-state summary
for `adr034-diskann-rebased`.

The new handoff is intentionally short and focused on what the next session
should actually do:

- treat task 17 as signoff/merge work
- use packets `11078` / `11081` / `11082` / `11083` as the live context
- prioritize faster-machine final benches and review feedback over more
  speculative local code churn

## Why this slice

- It is directly about closing DiskANN task 17 correctly.
- It removes stale guidance that still pointed at a superseded branch state.
- It makes the current “what next?” answer explicit for the next operator or
  agent touching this branch.

## Validation context

Docs only. No code changed in this packet.

The operator/runtime state referenced here is backed by the existing task-17
packets, especially:

- `review/11078-task17-diskann-real-recall-recovery/`
- `review/11081-task17-diskann-vacuum-recall/`
- `review/11082-task17-vacuum-pq-frontier/`
- `review/11083-task17-diskann-post-vacuum-smoke/`

## Follow-ups intentionally not in this packet

- Any new DiskANN AM code changes.
- Any new benchmark claims beyond what the cited packets already captured.
- Any `ecaz-cli` work.
