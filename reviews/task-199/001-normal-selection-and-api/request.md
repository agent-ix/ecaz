# Review request: normal selection and operator/control API

## Scope

Please review Task 199 commits:

- `ee44f0bd9` — absorb the outside Task 198 findings into the task definition;
- `ebf9950c1` — normalize owner/replica effective search defaults in the suite
  runner;
- `241579dfb` — production replica selection, coherence controls, lifecycle
  API, observability, fault drills, deterministic ordering, and normal-build
  feature isolation.

This is the first implementation checkpoint, not Task 199 closeout.

## Result

A matching `Ready` coordinator traversal replica is now the normal preferred
scan path. There is no production selector GUC. Absent and non-Ready images
take the unchanged owner path; malformed, missing-relation, or traversal-failed
Ready images receive a bounded durable diagnosis, are demoted, and restart
fully through owners.

The checkpoint addresses the Task 198 entry findings in code:

- real AM insert/bulk-delete, participant record-write, debug tombstone, and
  delta-fold mutation front doors call the invalidation guard;
- the replica copy holds a mutation-conflicting control-index fence through
  transaction end;
- invalidation passes a plain OID plus the exact active identity over a
  dedicated extension-owner connection with connect, statement, and lock
  timeouts;
- a server-owner-readable password-file GUC, preflight endpoint, and explicit
  operator recovery endpoint cover deployable authentication and recovery;
- Ready/Stale and superseded replicas can enter Retiring, and reclaim skips
  live fingerprint pins and is idempotent;
- Ready selection requires READ COMMITTED;
- the operator functions remain covered by the extension-wide SECURITY
  DEFINER, fixed-search-path, and PUBLIC-revocation closure;
- normal builds include the accepted operator/catalog surface while benchmark
  selector/fault instrumentation remains isolated;
- failed speculative replica traversal attribution is discarded before the
  owner rerun;
- owner and replica traversal use the same deterministic exact-score path and
  final `(distance, vec_id)` order. The early-exit proof now uses that same
  total-order boundary, preserving iterative-deepening prefix safety;
- suite validation rejects the unsupported exact-neighbor/replica combination
  and owner/replica arms expand with identical effective BW/H defaults.

The local multicluster runner now contains Task 199 drills for normal Ready
identity, isolation rejection, mid-scan full restart, corrupt-image fallback,
real INSERT invalidation/retry, explicit retire/reclaim, removed-image owner
fallback, and owner-outage partial-build cleanup. Those drills have not yet
been run in this packet.

## Validation

See `artifacts/manifest.md`.

- strict normal PG18 clippy: pass;
- focused scan ordering/early-exit unit tests: 11 passed;
- speculative attribution tests with benchmark instrumentation: 2 passed;
- suite implicit-default pairing regression: 1 passed;
- focused PG18 iterative-deepening integration regression: 1 passed.

## Requested review

Please focus on:

1. mutation/control-transaction lock ordering and exact-identity invalidation;
2. SECURITY DEFINER caller checks and extension-owner authentication;
3. Ready failure demotion versus owner availability;
4. retirement/reclaim pin handling and epoch-turnover behavior;
5. the total-order early-exit proof and failed-attempt telemetry accounting;
6. whether any Task 198 F1--F8 or P3 code requirement remains unimplemented
   before packet 002 runtime drills.

## Explicitly still open

- Packet 002 must run the full PG18 operations/lifecycle/isolation drills,
  including real DELETE/participant tombstone, concurrent build/mutation,
  lock-queue, authentication failure/recovery, epoch turnover, restart, and
  crash/error paths.
- Packet 003 must install a normal release build and run the required checked-in
  `ecaz bench suite` A/B matrix at 10k/50k/100k for recall, latency, storage,
  build/WAL/cache, topology, ordered identity, fallback, mutation, and
  lifecycle evidence.
- No promotion, closeout, or merge-as-done is requested by this packet.

