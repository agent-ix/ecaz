# Review request: operations, lifecycle, and isolation

## Scope

Please review the Task 199 operations/lifecycle checkpoint through
`1b3de943c`:

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
- `1b3de943c` — preserve owner fallback when durable demotion fails.

The suite config was committed separately as `be3e75ac4`. This packet is a
runtime checkpoint and response to the outside review at `f19462ecf`, not Task
199 closeout or release promotion.

## Result

The checked-in normal-release PG18 suite completed its one operations step
with `completed=1`, `failed=0`, `missing_artifacts=0`, and `stale=0`. The
extension, CLI, runner, and all three nodes reported exact SHA `1b3de943c`.

The run directly exercises:

- coherent build fencing against an actual blocking INSERT that observes
  `Building`, waits behind `ShareRowExclusiveLock`, resumes after Ready, and is
  rejected by the per-tuple guard with `40001`;
- one durable first-mutation `40001` through the actual INSERT front door;
- one durable `40001` with zero DELETE through the actual DELETE front door;
- one durable `40001` with an unchanged physical graph and zero tombstones
  through the actual participant endpoint;
- REPEATABLE READ and SERIALIZABLE owner fallback with ordered identity, plus
  SQLSTATE `25001` / `EC_TRANSACTION_ISOLATION` for writes at both levels;
- real VACUUM of a pre-build dead tuple, with durable Stale and continued
  maintenance;
- immutable in-flight cursor completion while invalidation commits;
- extension-owner control authentication failure, fail-closed behavior,
  owner fallback when both durable and local demotion are unavailable,
  preflight/recovery, and zero mutation;
- a locked/dropped replica-relation race with nonblocking owner fallback and
  durable diagnosis;
- a queued control-index `AccessExclusiveLock`, proving the plain-OID side
  transaction completes in 155 ms without relation-lock inversion;
- owner restart, owner-outage partial build, corrupt-image fallback, explicit
  retire/reclaim, and removed-image fallback;
- successor epoch publication, automatic `epoch_superseded` retirement,
  idempotent reclaim with zero old-relation residue, successor replica build,
  and fresh coordinator-backend identity;
- normal-build absence of the prototype selector/fault hooks and seven
  materialization semantic cases.

The representative Ready image copied exactly 10,000 records / 131,520,000
bytes, occupied 158,326,784 relation bytes, emitted 137,460,656 WAL bytes, and
built in 4,718 ms. The diagnostic A/B returned identical `0.9900` recall; its
two-sample warm means were 19.70 ms owner and 15.30 ms replica. Those small
sample counts are lifecycle smoke data only, not release-decision evidence.

## Validation

See `artifacts/manifest.md` for commands, hashes, provenance, and cited result
lines.

- normal PG18 release extension install: pass;
- exact-SHA release CLI build: pass, with one pre-existing corpus-loader
  dead-code warning;
- focused PG18 extension and CLI compile checks: pass;
- `ecaz bench suite audit`: pass;
- `ecaz bench suite run`: pass in 328,138 ms;
- `ecaz bench suite status`: completed 1, failed 0, missing 0, stale 0;
- three-node 10k topology: Ready/Published, 10,000 owned rows, zero non-owned
  rows, zero orphans;
- all emitted Task 199 lifecycle and semantic scenarios: `pass=true`.

## Requested review

Please focus on:

1. whether the real INSERT/DELETE/participant/VACUUM front doors now close F1
   and the packet-001 P2 maintenance findings;
2. whether the blocking INSERT proves the exact F2 ordering hazard identified
   as packet-002 P1-A and protects the per-tuple guard from removal;
3. whether stronger-isolation owner fallback and the disclosed `25001` write
   restriction resolve packet-001 P1-1/P1-2;
4. whether the invalidation-backed no-Ready presence cache is structurally
   sound; its required no-replica performance A/B remains assigned to packet
   003;
5. whether failed demotion now correctly warns and restarts through owners,
   including in a read-only transaction;
6. whether the epoch successor sequence proves reachable automatic retirement
   and leak-free reclaim for F4;
7. whether the 155 ms queued-DDL result adequately demonstrates the side
   transaction's plain-OID/no-pgrx-relation-lock property;
8. whether the explicit Task 167 fail-closed retry posture and actionable
   retire/reclaim rebuild guidance are correctly separated from Task 199
   invalidation rather than misrepresented as distributed mutation
   propagation;
9. what additional direct runtime evidence is required before packet 003.

## Explicitly still open

- The phase-3 disk-exhaustion and crash-after-control-commit cases still need
  direct normal-release runtime evidence.
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
