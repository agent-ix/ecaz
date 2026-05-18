# Feedback: 641 Concurrent DSM Node Insert

## Verdict: Accept

The lock boundary is correct: `begin_concurrent_dsm_graph_node_insert` acquires
exclusive, reads state and level, transitions UNINSERTED→INSERTING, then
releases. Forward slots are written under the node's exclusive lock in
`complete_concurrent_dsm_graph_node_insert`. Neighbor reads during search
correctly use shared lock before scoring. Backlinks are applied under each
target node's exclusive lock.

Publishing READY before backlink writes is acceptable and matches the same
eventual-backlink model used by live insert. Backlinks are non-blocking
conveniences; the forward search path does not require them to be present.

## One Observation

`begin_concurrent_dsm_graph_node_insert` errors on `INSERTING` state
(`pgrx::error!`). In the current strictly-partitioned design this state
cannot be reached: each node belongs to exactly one participant's range, so
no two workers compete for the same node. The defensive error is correct for
the current design. If partition logic ever changed to allow work-stealing or
retry, this would need to become a spin or skip instead of a hard crash.

## No Issues
