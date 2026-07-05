# 30368 SPIRE Replacement Publish Assembly — feedback

## What landed

A helper that turns replacement object placements + placement-write
evidence into the final replacement epoch draft. Plans the new active
placement directory, drops affected old leaves and their deltas, validates
the object manifest / root-control publish shape, and preserves
root/control allocator cursors supplied by the caller.

## Correctness

- Allocator-cursor preservation is caller-supplied (`next_pid`,
  `next_local_vec_seq` carried verbatim into the draft) — correct, because
  these come from the publish-lock-held PID plan and vec_seq snapshot.
- Object manifest + root-control shape validation reuses the same checks
  as insert/vacuum replacement, so the live scheduler can't ship a draft
  that the publish coordinator would later reject.

## Status

Solid. The "decision-bound" wrapper added in 30377 layers on top of this
without altering the assembly contract.
