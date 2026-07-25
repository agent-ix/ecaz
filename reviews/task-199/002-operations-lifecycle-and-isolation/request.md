# Review request: operations, lifecycle, and isolation

## Scope

Please review the Task 199 operations/lifecycle checkpoint through
`c9e64a9de`:

- `e9f17c644` — decode the normal transport row shape;
- `9984bbca0` — recover replica lifecycle after owner outage;
- `b32786449` — diagnose replica isolation status;
- `75369c0c9` — attest fail-closed real INSERT retry posture;
- `d98cf0d93` — isolate normal materialization checks;
- `6890b1088` — add real lifecycle, race, authentication, epoch, and reconnect
  drills;
- `96855f3eb` — use the CustomScan KNN query shape in race drills;
- `2f9dc2b77` — establish a deterministic queued-DDL lock;
- `c9e64a9de` — commit the epoch publish decision before recovery.

The suite config was committed separately as `be3e75ac4`. This packet is a
runtime checkpoint, not Task 199 closeout or release promotion.

## Result

The checked-in normal-release PG18 suite completed its one operations step
with `completed=1`, `failed=0`, `missing_artifacts=0`, and `stale=0`. The
extension, CLI, and all three nodes reported exact SHA `c9e64a9de`.

The run directly exercises:

- coherent build fencing against an actual concurrent INSERT;
- one durable first-mutation `40001` through the actual INSERT front door;
- immutable in-flight cursor completion while invalidation commits;
- extension-owner control authentication failure, fail-closed behavior,
  preflight/recovery, and zero mutation;
- a locked/dropped replica-relation race with nonblocking owner fallback and
  durable diagnosis;
- a queued control-index `AccessExclusiveLock`, proving the plain-OID side
  transaction completes in 44 ms without relation-lock inversion;
- repeatable-read rejection without image demotion;
- owner restart, owner-outage partial build, corrupt-image fallback, explicit
  retire/reclaim, and removed-image fallback;
- successor epoch publication, automatic `epoch_superseded` retirement,
  idempotent reclaim with zero old-relation residue, successor replica build,
  and fresh coordinator-backend identity;
- normal-build absence of the prototype selector/fault hooks and seven
  materialization semantic cases.

The representative Ready image copied exactly 10,000 records / 131,520,000
bytes, occupied 158,326,784 relation bytes, emitted 137,659,336 WAL bytes, and
built in 4,963 ms. The diagnostic A/B returned identical `0.9900` recall; its
two-sample warm means were 19.00 ms owner and 14.80 ms replica. Those small
sample counts are lifecycle smoke data only, not release-decision evidence.

## Validation

See `artifacts/manifest.md` for commands, hashes, provenance, and cited result
lines.

- normal PG18 release extension install: pass;
- exact-SHA release CLI build: pass, with one pre-existing corpus-loader
  dead-code warning;
- `ecaz bench suite audit`: pass;
- `ecaz bench suite run`: pass in 299,123 ms;
- `ecaz bench suite status`: completed 1, failed 0, missing 0, stale 0;
- three-node 10k topology: Ready/Published, 10,000 owned rows, zero non-owned
  rows, zero orphans;
- all emitted Task 199 lifecycle and semantic scenarios: `pass=true`.

## Requested review

Please focus on:

1. whether the real INSERT, build-fence, in-flight cursor, auth, relation-race,
   and queued-DDL drills faithfully exercise F1--F3, F5--F7, and the relevant
   P3 findings;
2. whether the epoch successor sequence proves reachable automatic retirement
   and leak-free reclaim for F4;
3. whether the 44 ms queued-DDL result adequately demonstrates the side
   transaction's plain-OID/no-pgrx-relation-lock property;
4. whether the explicit Task 167 fail-closed retry posture is correctly
   separated from Task 199 invalidation rather than misrepresented as
   distributed mutation propagation;
5. what additional direct runtime evidence is required before packet 003.

## Explicitly still open

- F1 is not fully closed: this packet proves the actual INSERT front door, but
  still needs actual DELETE and participant tombstone front-door evidence.
- The phase-3 disk-exhaustion and crash-after-control-commit cases still need
  direct normal-release runtime evidence.
- Heterogeneous-ISA ordered identity cannot be established by this local
  three-node same-host run; packet 001 contains the deterministic ordering
  code/tests, and final cross-host evidence remains open.
- Packet 003 must run the checked-in normal-release A/B matrix at
  10k/50k/100k with 200 recall queries / 2,000 trials, 10 warmups / 50 timed
  samples, storage, build/WAL/cache, topology, ordered identity, fallback,
  mutation, and operator lifecycle evidence.
- Packet 001 and this packet remain open until outside reviewer feedback is
  written to their feedback directories. No promotion, closeout, or
  merge-as-done is requested.
