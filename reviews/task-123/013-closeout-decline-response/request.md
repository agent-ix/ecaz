# Task 123 Review Request: Closeout Decline Response

## Scope

This packet responds to reviewer feedback on packet
`012-revised-core-algorithm-closeout`:

`reviews/task-123/012-revised-core-algorithm-closeout/feedback/2026-06-28-01-reviewer.md`

The packet 012 closeout request is retracted. Task 123 is **not** coder-complete
under the 2026-06-27 reopened mandate.

## Response

The packet 009 32-query latency headline is explicitly retracted. The 200-query
packet 011 data does not support the earlier "multi-instance erases the scan
wall" interpretation:

| Config | Packet 009 32q p50 | Packet 011 200q p50 | Current interpretation |
| --- | ---: | ---: | --- |
| n128 b4/tr50/f8 np96 | 337 ms | 5408.521 ms | no multi-instance latency win; essentially back at the single-instance wall |
| n1024 b2/tr50/f8 np64 | 87 ms | 770.595 ms | better than n128, but not the earlier small-sample result |

The valid interim result is narrower:

- recall is validated at 200 queries on the contained local multi-instance lane;
- `n1024 b2/tr50/f8` remains the better high-recall research candidate than
  `n128 b4/tr50/f8` in packet 011;
- latency and communications are **not** closed.

## Artifact Trace

Packet 013 adds `artifacts/latency-trace.jsonl`, an extracted structured JSONL
index of the packet 011 latency rows cited above. The original packet 011 rows
already use `latency_p50` / `latency_p95`, but this packet makes the trace
explicit and review-local.

Packet 013 also adds `artifacts/projection-failure-diagnosis.md`. The
`id,source` projection failure is now identified from the committed PostgreSQL
log as the remote tuple payload cap:

`remote typed tuple payload payload bytes 12316 exceeds ec_spire.max_remote_payload_bytes_per_row 1024`

That means the next communications measurement must either use a narrower
realistic projection or raise the packet-local payload guard with explicit
evidence. The current packet does not treat this as resolved.

## Remaining Blockers

Task 123 remains open on:

- clean latency rerun on a non-disk-constrained host;
- realistic projection / payload measurement that does not fail
  `remote_payload_too_large`;
- per-worker communications bytes for the realistic payload path;
- PR #43 `ec_spire.pre_materialization_prune` A/B on the selected measurement
  surface;
- final latency + communications verdict for the reopened mandate.

Disk state at response time remains unsuitable for clean latency evidence:
`df -h /tmp/ecaz-task123 /home/peter/dev/ecaz` reported `/dev/sdf` at
`936G/1007G`, `20G` available, `98%` used on 2026-06-28.
