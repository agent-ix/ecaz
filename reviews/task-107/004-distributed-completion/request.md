# Task 107 Packet 004 Benchmark Completion

## Summary

This packet records the completed Task 107 benchmark checklist after removing
out-of-scope single-node/single-disk and four-disk rows. The checklist now
contains only:

- single node with 2 disks;
- multinode with 1 controller and 2 nodes.

All required cells were run or cited from the already-completed packet-003
evidence. No Task 106 single-node/single-disk evidence was rerun.

## Final State

- Phase 1 single node with 2 disks:
  - `phase1-rabitq-100k-l2`: completed.
  - `phase1-rabitq-1m-l2`: completed.
  - `phase1-turboquant-100k-l2`: completed.
  - `phase1-turboquant-1m-l2`: completed.
- Phase 2 multinode with 1 controller and 2 nodes:
  - `phase2-rabitq-100k-l1`: completed in packet 003; cited, not rerun.
  - `phase2-rabitq-1m-l1`: completed.
  - `phase2-turboquant-100k-l1`: completed after commit
    `92254bca929c949f2de3715efefec6d4c53e4568`.
  - `phase2-turboquant-1m-l1`: completed all suite steps; suite returned
    nonzero because two recall thresholds missed by 0.001.

## Key Result Notes

- TurboQuant distributed remote endpoint support was fixed in
  `92254bca929c949f2de3715efefec6d4c53e4568`.
- Local smoke tests before AWS passed:
  - `cargo test remote_tuple_transport_tests --no-default-features --features pg18`
  - `cargo pgrx test pg18 test_ec_spire_remote_search_endpoint_identity`
- `phase2-turboquant-1m-l1` suite result:
  - all 9 suite steps succeeded;
  - `suite thresholds failed: 2`;
  - k10 nprobe64 recall actual `0.9490`, expected `>=0.9500`;
  - production-read k10 nprobe64 recall actual `0.9490`, expected `>=0.9500`.
- `phase2-turboquant-1m-l1` production remote-read evidence:
  - k10 and k100 both `status=ready`;
  - `result_source=remote_heap_candidates`;
  - `selected_pid_sum=6400`, `remote_pid_sum=6400`, `dispatch_sum=200`;
  - timeout, cancel, and degraded-skip sums were all zero.
- Cleanup completed successfully on the coordinator and both remotes after the
  final cell. EC2 instances were initially left running for operator review;
  they were later destroyed and verified in
  `artifacts/aws-teardown/teardown-summary.md`.

## Evidence

- Checklist source of truth:
  `reviews/task-107/004-distributed-completion/run-checklist.md`
- Packet manifest:
  `reviews/task-107/004-distributed-completion/artifacts/manifest.md`
- TurboQuant 100k distributed artifacts:
  `reviews/task-107/004-distributed-completion/artifacts/phase2-turboquant-100k-l1/direct-ssm-distributed/`
- TurboQuant 1m distributed artifacts:
  `reviews/task-107/004-distributed-completion/artifacts/phase2-turboquant-1m-l1/direct-ssm-distributed/`
