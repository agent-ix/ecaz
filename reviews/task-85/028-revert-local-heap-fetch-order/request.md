# Task 85 Packet 028: Revert Rejected Local Heap Fetch Ordering

## Summary

Packet 027 rejected the packet 026 local heap TID fetch-order implementation on
AWS 1M/q500: it preserved recall/candidates/rerank width but worsened warm
latency. This packet removes that rejected implementation from the Task 85
branch with a normal revert commit.

Reverted code commit:

- `4f92108eda6903fb524b6feb068b886622ff0122`:
  `Order SPIRE local heap rerank fetches by TID`

Revert commit:

- `7302c8369`: `Revert "Order SPIRE local heap rerank fetches by TID"`

## Evidence

- Rejection measurement:
  `reviews/task-85/027-aws-local-heap-fetch-order/`
- Revert commit:
  `7302c8369`

No new runtime behavior is introduced by this packet. It restores the
packet 023/025 local heap fetch path after packet 027 proved the TID-ordered
variant was not a retained-recall latency improvement.

## Validation

No tests were run for the revert. The reverted code had already been measured
and rejected on AWS in packet 027; this checkpoint removes that measured
regression from the branch.
