---
head_sha: 6c5b416e9262f6304acf52a98ab4fbb35aaf2a03
task: task-123
packet: reviews/task-123/020-post-ab-closeout-request
date: 2026-06-29
---

# Manifest

This is a synthesis-only closeout request after the reopened Task 123
multi-instance efficiency loop. It adds no new benchmark run. The evidence
source is packet 019 plus the accepted packet 017 communications datapoint and
packet 018 code review.

## Evidence Chain

- Packet 017:
  `reviews/task-123/017-multinode-communications-prune-ab/`
  - Reviewer accepted it as the communications datapoint in
    `feedback/2026-06-29-02-reviewer.md`.
  - It quantified that `id,source` ships about 73.9 MB over 200 queries across
    the three remotes vs about 48 KB for `id`, while latency stayed flat within
    that packet.
  - It also established that packet 017 prune on/off rows were inert under
    `VecIdDedupeEnabled`.

- Packet 018:
  `reviews/task-123/018-dedupe-prune-threshold/`
  - Code commit `d2ffbdaa9` makes the pre-materialization threshold engage for
    bounded dedupe by delegating to `min_ip_to_keep()`.
  - Reviewer marked the code LGTM in
    `feedback/2026-06-29-01-reviewer.md`, with required next step: b2/b4
    prune on/off recall + latency A/B.

- Packet 019:
  `reviews/task-123/019-dedupe-prune-multinode-ab/`
  - Provides the requested engaged b2/b4 A/B on local PG18 multi-instance
    production-read workloads.
  - n1024/b2/nprobe64 and n128/b4/nprobe96 both preserve recall at 1.0000
    across prune on/off, source/id projection, and rowcap/default variants.
  - Prune on/off is flat in latency. The corrected threshold is therefore
    correctness-needed to make the lever real, but not a demonstrated latency
    win in this representative matrix.
  - Communications counters remain healthy: remote heap dispatch succeeds, no
    degraded skips, and per-worker payload bytes distinguish `id,source`
    (24,632,000 bytes) from `id` (16,000 bytes).

## Closeout Position

Task 123 should close as a no-promote / re-scope result for the reopened
multi-instance core algorithm scope:

- The prior 32-query latency optimism stays retracted.
- Multi-instance recall is stable in the tested representative cells.
- The communications dimension is measured and is not the dominant local
  latency driver in the accepted packet 017 relative comparison.
- The dedupe-aware prune fix is recall-safe in unit coverage and recall-neutral
  in packet 019's b2/b4 multi-instance A/B, but it does not produce a meaningful
  latency improvement.
- Absolute latency remains poor, especially n128 at about 5.1s to 5.2s p50,
  and n1024 remains around 0.73s to 0.78s p50 in packet 019.

No task status file is flipped in this packet. The request is for reviewer
decision on whether packets 017/018/019 are sufficient to close the reopened
Task 123 core-algorithm mandate.
