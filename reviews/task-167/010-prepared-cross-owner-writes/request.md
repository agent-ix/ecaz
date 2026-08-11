---
agent: coder
role: coder
model: GPT-5
date: 2026-08-11
seq: 10
---

# Task 167 checkpoint: prepared cross-owner physical writes

Head under review: `c9a06c28b` (`feat(ec-distann): prepare cross-owner physical writes`).

This checkpoint extends the Task 167 physical-generation path with:

- owner-local physical backlink application by stable `vec_id`;
- coordinator routing of remote forward-neighbor backlinks;
- remote owner append/backlink transactions prepared under the originating
  transaction and resolved by commit/abort callbacks;
- physical-generation endpoint registration and owner/epoch/READ COMMITTED
  validation for backlinks;
- retained row-tier identity verification before a replacement is accepted.

Validation is limited to the packet-local compile result in
`artifacts/validation.log`. The focused pgrx test did not complete with a
trustworthy result in this turn. This remains an open implementation
checkpoint, not a closeout request.

Open acceptance work:

- durable prepared-transaction intent/reaper coverage for backend crash
  recovery;
- complete UPDATE classification for vector-changing updates versus duplicate
  INSERTs;
- routed tombstone delete;
- robust-prune backlink behavior and concurrent insert/query validation;
- TC-043 and NFR-020 fault/concurrency drills;
- `ecaz bench suite` A/B recall, latency, storage, and throughput evidence at
  10k/50k/100k.

Please review the implementation and identify correctness blockers before the
benchmark and closeout work proceeds.
