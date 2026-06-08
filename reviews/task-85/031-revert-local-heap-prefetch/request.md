# Task 85 Packet 031: Revert Rejected Local Heap Prefetch

## Summary

Packet 030 rejected the packet 029 local heap prefetch implementation on AWS
1M/q500: it preserved recall/candidates/rerank width but worsened warm latency
versus the accepted controls. This packet removes that rejected implementation
from the Task 85 branch with a normal revert commit.

Reverted code commit:

- `94fef559c`: `Prefetch SPIRE local heap resolution blocks`

Revert commit:

- `ce8b5fe1e`: `Revert "Prefetch SPIRE local heap resolution blocks"`

## Evidence

- Rejection measurement:
  `reviews/task-85/030-aws-local-heap-prefetch/`
- Revert commit:
  `ce8b5fe1e`

No new runtime behavior is introduced by this packet. It restores the
packet 023/025 local heap resolution path after packet 030 proved explicit
local heap block prefetch was not a retained-recall latency improvement.

## Validation

No tests were run for the revert. The reverted code had already been measured
and rejected on AWS in packet 030; this checkpoint removes that measured
regression from the branch.
