# Task 94 Packet 027 Artifact Manifest

- head SHA: `4c796439db80b160a841fd749f03d26e2072a9ec`
- task bucket: `reviews/task-94/027-graviton4-closeout-runbook/`
- timestamp: `2026-06-10T06:34:22Z`
- lane: coder-1 LUT lane
- quant: `grouped_pq`
- storage format / AM surface: not applicable; closeout-prep runbook only
- isolated one-index-per-table or shared-table surface: not applicable
- CI: not run
- AWS: not run

## Artifacts

| Artifact | Command | Result |
| --- | --- | --- |
| `graviton4-closeout-runbook.md` | written by inspection from Task 94 feedback, Task 94 packet manifests, `docs/aws-bench-workflow.md`, and `docs/block-kernel-development.md` | future approved-AWS runbook |
| `git-diff-check.log` | `git diff --check` | passed |

## Key Status

This packet does not claim new runtime evidence. It records the exact
approval-gated Graviton 4 evidence pass required to close Task 94:

- restore via `snap-0e9c7743263e61d70`;
- install `task-94-grouped-pq-block-kernel`;
- run grouped-PQ NEON/SVE2 parity hooks on-host;
- measure SVE vector lanes from the runtime helper;
- run `ecaz bench suite` for the Task 94 grouped-PQ/PqFastScan matrix;
- cite direct `[block-kernel-counters]` rows for `quant=grouped_pq`;
- carry forward packet 026's pruning-vs-throughput and GUC-default notes.
