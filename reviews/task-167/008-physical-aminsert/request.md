# Task 167 packet 008 — physical-generation aminsert checkpoint

Coder checkpoint for commit `64d940f87` (`feat(ec-distann): route physical generation inserts`).

## Delivered in this checkpoint

- Distributed-control `aminsert` now resolves a Published physical generation
  and routes by the stable source-identity `vec_id` to its hash owner.
- Owner-local writes append a generation-schema row-tier tuple and physical
  graph record in the callback transaction, with bounded traversal candidates,
  exact robust-prune selection, and local backlink amendments.
- Remote owners receive a frozen row payload, source vector, identity payload,
  and epoch fingerprint; the endpoint validates generation and placement before
  decoding or writing.
- Graph storage retains prior records using `record_version` and `is_current`;
  readers and handoff diagnostics select current records only.
- Remote search candidates are materialized through the existing generation
  payload contract instead of treating owner TIDs as local TIDs.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.log`.

- PG18 compile: pass.
- PG18 physical-generation lifecycle/replay/privilege test: pass.

## Review scope and remaining blockers

This is an implementation checkpoint, not a Task 167 closeout request. The
following acceptance work remains intentionally visible for the next slice:

1. Cross-owner backlink publication needs a pending/idempotent protocol so a
   remote write cannot leave a dangling backlink if the coordinator transaction
   aborts.
2. The live-path distinction between an insert-time `vec_id` collision and an
   UPDATE replacement with the same stable identity must be wired through the
   callback/owner endpoint.
3. TC-043 fault/concurrency drills, the bounded-work counter evidence, and the
   required `ecaz bench suite` 10k/50k/100k recall/latency/storage A/B evidence
   have not yet been produced.

Task status remains `partial`; no closeout bookkeeping is changed by this
packet.
