# Task 167 review fixes

Please review code checkpoint `ae19b8ce0`, which addresses the CHANGES
REQUESTED findings in packet 022 and the related evidence defects in packet
020.

Implemented changes:

- P1-1/P2-7/P2-8: restore in-process local read routing; gate the retained-row
  fallback on an actual unreadable local neighbor; lock and re-read graph
  records before backlink/tombstone mutation; make successful amendments,
  already-present edges, and no-room drops separate counters; report measured
  inserted-neighborhood parity separately from the blended physical-vs-fresh
  ANN agreement.
- P1-2/P1-3: record `commit_intended` on the owner in the coordinator's
  `PreCommit` callback, abort the coordinator commit if that fence cannot be
  recorded, and run prepared-xact xid liveness checks through coordinator SPI
  rather than the owner's xid space. The reaper commits `commit_intended` and
  rolls back only undecided intents.
- P1-4/P3-11: preserve `conninfo=None` for local read routes and validate owner
  placement against the immutable generation descriptor, while requiring the
  runtime roster ordering to match it for connection lookup.
- P1-5/P3-10: resolve participant tombstones by the published fingerprint,
  prune successfully processed source-map rows, and add a three-owner routed
  DELETE + VACUUM drill.
- P2-6: add two concurrent writers to the physical fixture's existing reader
  drill, with row locking preventing stale whole-record overwrite or
  tombstone resurrection.
- P3-12/P3-13: keep INVALID source TIDs out of remote source-map writes and
  document the palloc lifetime invariant for binary receive datums.
- New review fixes: allow the physical `commit_intended` state in both schema
  paths, fail VACUUM on routed tombstone errors so source-map rows remain
  retry tokens, retain a resolved local endpoint for empty-roster fallback,
  and make the concurrency drill assert inserted IDs appear in forward
  neighbor lists.
- Remote transport compatibility: expose `payload_offsets` from the
  upgrade-time row-payload wrapper, matching the transport ABI.

Validation and benchmark provenance are recorded in `artifacts/manifest.md`
and the packet-local logs it cites. The exact-head two-owner probe committed a
remote insert (`owner=1`, `remote=true`); the later backlink failure keeps the
packet review-open and does not claim the concurrency drill passed.

head_sha: ae19b8ce0
task_bucket: reviews/task-167
packet: 023-review-fixes
timestamp: 2026-08-12 (America/Los_Angeles)
