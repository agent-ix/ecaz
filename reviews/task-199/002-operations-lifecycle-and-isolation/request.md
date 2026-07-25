# Review request: operations, lifecycle, and isolation

## Scope

Please review the Task 199 operations/lifecycle checkpoint through
`7c27a9916`:

- `e9f17c644` — decode the normal transport row shape;
- `9984bbca0` — recover replica lifecycle after owner outage;
- `b32786449` — diagnose replica isolation status;
- `75369c0c9` — attest fail-closed real INSERT retry posture;
- `d98cf0d93` — isolate normal materialization checks;
- `6890b1088` — add real lifecycle, race, authentication, epoch, and reconnect
  drills;
- `96855f3eb` — use the CustomScan KNN query shape in race drills;
- `2f9dc2b77` — establish a deterministic queued-DDL lock;
- `c9e64a9de` — commit the epoch publish decision before recovery;
- `1bb6fe81c` — invalidate replicas before source-table DML;
- `f0f58e869` — exercise real DELETE and participant-tombstone front doors;
- `a3932f6e4` — inspect the physical participant tombstone generation;
- `b242c748a` — preserve stronger-isolation reads and add the blocking fence
  drill;
- `6a7c1cfed` — add the invalidation-backed no-Ready cache, VACUUM disposition,
  fallback/rebuild drills, and operator runbook;
- `1b3de943c` — preserve owner fallback when durable demotion fails;
- `d08838473` — add the post-control-commit backend-crash drill;
- `5465f7693` — add armed fault-provider suite control;
- `2dd24b6dc` — preserve writes across stronger isolation and bound failed
  control retries per backend/build;
- `6c73649be` / `85174f1be` — add and enable the ENOSPC replica-build drill;
- `6bf805bdb` — re-enable replica selection after operator control recovery;
- `fcfc529e5` — make the Ready-presence probe safe on an empty catalog;
- `a4a8c3aa2` — intercept PG18 vectored writes in the fault provider;
- `f67562904` / `7c27a9916` — target tablespace relation creates and use an
  absolute arm-file path.

The suite config was committed separately as `be3e75ac4`. This packet is a
runtime checkpoint and response to the outside review at `f19462ecf`, not Task
199 closeout or release promotion.

## Result

The checked-in normal-release PG18 suite completed its one operations step
with `completed=1`, `failed=0`, `missing_artifacts=0`, and `stale=0`. The
extension, CLI, runner, and all three nodes reported exact SHA `7c27a9916`.

The run directly exercises:

- coherent build fencing against an actual blocking INSERT that observes
  `Building`, waits behind `ShareRowExclusiveLock`, resumes after Ready, and is
  rejected by the per-tuple guard with `40001`;
- one durable first-mutation `40001` through the actual INSERT front door;
- one durable `40001` with zero DELETE through the actual DELETE front door;
- one durable `40001` with an unchanged physical graph and zero tombstones
  through the actual participant endpoint;
- READ UNCOMMITTED replica selection, REPEATABLE READ and SERIALIZABLE owner
  fallback with ordered identity, plus actual writes at both stronger levels
  fenced by `40001 EC_REPLICA_INVALIDATED` with a rebuild between cases;
- a backend terminated after the durable control commit but before outer
  mutation retry, leaving Stale, zero inserted rows, and fresh-backend owner
  identity;
- armed PG18 ENOSPC at hidden-relation creation, with SQLSTATE `53100`, one
  provider `errno=28` event, zero catalog/relation residue, healthy owner
  fallback, and a successful recovery build;
- real VACUUM of a pre-build dead tuple, with durable Stale and continued
  maintenance;
- immutable in-flight cursor completion while invalidation commits;
- extension-owner control authentication failure, fail-closed behavior,
  owner fallback when both durable and local demotion are unavailable,
  backend/build suppression of repeated control attempts, operator
  preflight/recovery, and zero mutation;
- a locked/dropped replica-relation race with nonblocking owner fallback and
  durable diagnosis;
- a queued control-index `AccessExclusiveLock`, proving the plain-OID side
  transaction completes in 45 ms without relation-lock inversion;
- owner restart, owner-outage partial build, corrupt-image fallback, explicit
  retire/reclaim, and removed-image fallback;
- successor epoch publication, automatic `epoch_superseded` retirement,
  idempotent reclaim with zero old-relation residue, successor replica build,
  and fresh coordinator-backend identity;
- normal-build absence of the prototype selector/fault hooks and seven
  materialization semantic cases.

The representative Ready image copied exactly 10,000 records / 131,520,000
bytes, occupied 158,326,784 relation bytes, emitted 137,659,336 WAL bytes, and
built in 5,067 ms. The diagnostic A/B returned identical `0.9900` recall; its
two-sample warm means were 19.50 ms owner and 16.10 ms replica. Those small
sample counts are lifecycle smoke data only, not release-decision evidence.

## Validation

See `artifacts/manifest.md` for commands, hashes, provenance, and cited result
lines.

- normal PG18 release extension install: pass;
- exact-SHA release CLI build: pass, with one pre-existing corpus-loader
  dead-code warning;
- focused PG18 extension and CLI compile checks: pass;
- `ecaz bench suite audit`: pass;
- `ecaz bench suite run`: pass in 357,885 ms;
- `ecaz bench suite status`: completed 1, failed 0, missing 0, stale 0;
- three-node 10k topology: Ready/Published, 10,000 owned rows, zero non-owned
  rows, zero orphans;
- all emitted Task 199 lifecycle and semantic scenarios: `pass=true`.

## Requested review

Please focus on:

1. whether the fresh post-lock snapshot preserves stronger-isolation writes
   while retaining ordered owner fallback for stronger-isolation reads;
2. whether exact-build backend suppression bounds broken-control retries and
   correctly clears on operator recovery/replacement publication;
3. whether the post-control-commit backend termination proves the durable
   invalidation fence survives process loss with zero mutation;
4. whether the armed PG18 `53100` / `errno=28` run, zero residue, owner
   fallback, and recovery build close packet-002 P2-C;
5. whether the empty-catalog Ready-presence fix and catalog-specific relcache
   invalidation preserve the no-Ready cache invariant;
6. what additional direct runtime evidence is required before packet 003.

## Explicitly still open

- Heterogeneous-ISA ordered identity cannot be established by this local
  three-node same-host run; packet 001 contains the deterministic ordering
  code/tests, and final cross-host evidence remains open.
- Packet 003 must run the checked-in normal-release A/B matrix at
  10k/50k/100k with 200 recall queries / 2,000 trials, 10 warmups / 50 timed
  samples, storage, build/WAL/cache, topology, ordered identity, fallback,
  mutation, and operator lifecycle evidence. Per outside review it must also
  compare no-replica read latency and insert/load throughput before versus
  after the Task 199 boundary.
- Packet 001 and this packet remain open until the outside reviewer accepts
  these fixes. No promotion, closeout, or merge-as-done is requested.
