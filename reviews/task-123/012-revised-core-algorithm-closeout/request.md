# Task 123 Review Request: Revised Core-Algorithm Closeout

## Scope

This packet requests closeout under the revised intent: complete the contained
multi-instance **core routing/recall** validation for Tasks 121 and 123 without
claiming true cross-network performance or realistic-payload transport.

It responds to the reviewer feedback on packets 009/010:

- The 32-query gap is closed by packet 011's 200-query rerun.
- The realistic projection gap is recorded, not hidden: `id,source` failed with
  `remote_heap_resolution_failed`.
- PR #43 `ec_spire.pre_materialization_prune` and full communications
  attribution are transport/materialization follow-ups, outside this narrowed
  core-algorithm closeout.

## Evidence

Primary evidence is packet `011-multi-instance-100k-timeline-rerun`.

| Config | nprobe | Queries | p50 | p95 | recall@10 | Coordinator index |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| n128 b4/tr50/f8 | 8 | 200 | 662.821 ms | 923.969 ms | 0.9900 | 392.2 MiB |
| n128 b4/tr50/f8 | 96 | 200 | 5408.521 ms | 5815.967 ms | 1.0000 | 392.2 MiB |
| n1024 b2/tr50/f8 | 8 | 200 | 555.397 ms | 581.701 ms | 0.9290 | 246.1 MiB |
| n1024 b2/tr50/f8 | 64 | 200 | 770.595 ms | 860.296 ms | 1.0000 | 246.1 MiB |

The completed evidence uses id-only projection and therefore answers the
route/core-executor question, not realistic transport. Under that scoped claim,
`n1024 b2/tr50/f8` remains the better high-recall local multi-instance research
candidate than `n128 b4/tr50/f8`: it reaches recall 1.0 with lower high-nprobe
latency and lower coordinator-index storage.

## Completion Claim

Task 123 should be considered coder-complete for the revised core-algorithm
scope:

- contained local multi-instance substrate used;
- requested 100k cells rerun at 200 queries;
- route/recall behavior measured on the real distributed executor;
- non-claims and transport follow-ups explicitly recorded;
- no default promotion made.

This closeout does not ask the reviewer to accept a communications or
cross-network cost verdict.
