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
completed suite reproduced 0.9625 distinct recall (95% CI 0.9532--0.9700).
Warm latency was 38.10 ms mean, 37.30 ms p50, 48.80 ms p95, 53.70 ms p99,
and 55.80 ms max.

The coordinator decomposition is:

| Stage | Mean/query | Boundary |
| --- | ---: | --- |
| request wait | 25.369 ms | concurrent requests dispatched until all rows available |
| coordinator decode | 0.060 ms | PostgreSQL rows to owned Rust payloads |
| final output association | 0.040 ms | ranked slots associated with payload state |
| request preparation | 0.007 ms | partitioning and request construction |
| connection/statement readiness | 0.002 ms | pooled connection plus prepared statement readiness |
| result-map insertion | 0.002 ms | decoded payload insertion |

The following are explicitly nested work timers and are not added to that
table. The maximum owner endpoint critical path was 22.728 ms/query. Across
both requested owners, endpoint work summed to 39.981 ms/query, comprising
32.277 ms payload SQL / tuple encoding work, 6.683 ms open and schema
validation, and 1.010 ms graph-directory lookup. Summed owner request round
trips were 44.620 ms/query. The request-wait minus maximum owner endpoint
residual is 2.641 ms/query and includes server result encoding after endpoint
return, wire transfer, and client scheduling; this instrumentation does not
mislabel that residual as network-only.

Work attribution shows 40 ranked/associated output slots per query, but the
executor and client consume only 10. Of 26.84 remote payloads eagerly requested
per query, only 6.64 are consumed. Thus 20.20 remote payloads/query (75.3%) are
unused. The path returns 496,003 logical payload bytes/query (binary values plus
null flags, excluding PostgreSQL protocol framing), four columns per remote
row, with zero tombstones or missing payloads.

## Pre-registered isolated candidate

This evidence selects exactly one candidate from MAT-01/MAT-04: **bounded
global-ranked-window incremental payload materialization**. It does not select
copy elimination, connection changes, projection pruning, statement rewrites,
or owner caching.

- Baseline is the unchanged eager path (`batch_size=0`). Candidate uses a
  benchmark-feature-only, default-off batch size of 10 global ranked slots.
- Ranked remote slots initially retain only immutable `vec_id` identity. On the
  first remote slot in each deterministic `[n*10, (n+1)*10)` window, all pending
  remote slots in that window are fetched concurrently by owner using the
  existing endpoint, projection attnums, schema fingerprint, epoch fingerprint,
  pooled connections, order checks, and failure propagation.
- Ten is a request granularity, not a correctness or work cutoff. Qual rejection
  causes the executor to request the next ranked slot and therefore the next
  window. Existing search deepening and its fixed cap remain authoritative.
- Windows never skip an executor-requested ranked slot and never fetch outside
  the finite current ranked output set. Across stable-prefix deepening, already
  consumed windows are not fetched again. Maximum payload work is bounded by
  the same finite candidate/deepening cap as the eager path.
- A tombstone or missing payload is handled with the current semantics; any
  owner, fencing, decode, or transport failure aborts the query rather than
  returning the materialized prefix as complete.

Packet 003 will first extend the suite runner with a same-generation eager/lazy
arm, then implement only this candidate. Correctness evidence must include
unfiltered output identity; filters rejecting the first and multiple batches;
toasted/varlena values; nulls; mixed local/remote winners; projection/qual
columns; and a post-first-batch remote failure. Only a useful isolated A/B may
proceed to the 10k/50k/100k confirmation in packet 004.

Please review the timer nesting/boundaries, work-counter meanings, fail-closed
telemetry decode, feature isolation, measured attribution, and the single
pre-registered candidate boundary.
