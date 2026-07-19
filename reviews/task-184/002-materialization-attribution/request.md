---
task: 184
packet: 002-materialization-attribution
role: coder
status: open
date: 2026-07-19
head: 0f4b1d44c
---

# Review request: remote-materialization attribution

Checkpoint `0f4b1d44c` implements the feature-gated Task 184 attribution
contract from packet 001. It adds non-overlapping coordinator timers for
preparation, connection/statement readiness, concurrent request wait,
coordinator decode, map insertion, and final output association. Explicitly
nested work timers report summed per-owner request round trips and owner
endpoint work, the maximum owner endpoint critical path, and owner-side open /
schema validation, graph-directory lookup, and payload-SQL work.

The same counter snapshot reports ranked candidates; remote candidates and
owners requested; rows, tombstones, logical payload bytes, and columns
returned; installed payloads; associated outputs; executor-consumed local and
remote rows; and client result rows. The last metric is counted independently
by `ecaz bench latency`, so executor filtering is derivable rather than
inferred from `LIMIT`.

The owner endpoint repeats one validated telemetry record on each response row.
Coordinator decoding rejects inconsistent or negative telemetry. Existing
ordered row validation, tombstone behavior, projection attnums, schema and
epoch fingerprints, pooled connections, concurrent per-owner requests, and
error propagation remain unchanged. All instrumentation and the profiling
endpoint compile only under `distann-head-attribution-benchmark`; the normal
PG18 build also compiles.

The checked-in suite config reproduces Task 183's retained 100k policy and
topology exactly: trained exact 4,096-entry head, 32 seeds, BW4/H100, RaBitQ
neighbor traversal, exact final ranking, three owners, 200 held-out recall
queries, and 50 timed latency queries after 10 warmups at concurrency 1. The
profile result and candidate pre-registration will be appended to this packet
after the installed-release suite run. No candidate is selected in advance of
that measurement.

Please review the timer nesting/boundaries, work-counter meanings, fail-closed
telemetry decode, feature isolation, and suite propagation.
