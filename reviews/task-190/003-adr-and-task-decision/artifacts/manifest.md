# Task 190 packet 003 artifact manifest

Date: 2026-07-23 (America/Los_Angeles)

This is a design-decision packet with no code or benchmark behavior change.

Decision outputs:

- `spec/adr/ADR-086-ec-distann-coordinator-traversal-replica.md`;
- `plan/tasks/198-ec-distann-coordinator-traversal-replica.md`;
- updated Task 190 status and decision;
- updated DistANN recall/latency roadmap entries for `ARCH-02`, `ARCH-07`,
  and `TRAV-28`; and
- updated task index entries for Tasks 190 and 198.

The ADR selects a rebuildable, fingerprint-bound derived traversal replica,
not payload replication or a new source of truth. It requires active-epoch
pin/revalidation, complete digest/cardinality/owner coverage before Ready,
single-authority mutation invalidation, unchanged owner traversal fallback,
full restart after a mid-scan replica failure, and no partial success. An
in-flight scan that pinned Ready may complete on its immutable image; new scans
observe Stale.

Task 198 separates faithful prototype, lifecycle/fault work, isolated paired
100k A/B, and conditional 10k/50k/100k confirmation. Compact packing, sparse
replication, binary RPC, placement changes, payload replication, and replica
mutation propagation are excluded from the initial causal cell.

Rollback is explicit: invalidate/remove the derived replica and serve via the
existing owner traversal path. Production remains unchanged until Task 198
evidence supports a later promotion decision.
