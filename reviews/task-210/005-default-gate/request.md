# Task 210 review request: the default-config gate (003a blocker 1 rerun)

- Branch: `task-203-ec-distann-conformance`
- Commits under review: `fe5822f46` (sharded head is the shipped default),
  `e0c93c21c` (replica serving reachable; population attests complete
  coverage), `a3dccfd16` + `0b3e688e6` (fixture populates replicas and
  applies the replica GUC independently of the legacy flag), `81e816f32`
  (replica-served head searches counted through the shard cache; bench child
  argv logged).
- Evidence: `artifacts/manifest.md`, `artifacts/run/results.jsonl` (default
  arms), `artifacts/run-replica/results.jsonl` (replica arms).

## What the gate shows

1. **NFR-021 clause 5 holds as shipped.** With no arm flags and no session
   GUCs, a `CREATE INDEX` on a 3-owner roster persists a membership-only
   head: coordinator-resident index state is a constant 53,440 bytes at
   10k/50k/100k (was 25.9 MB), recall matches the 003a A/B at every scale,
   and the `task210-default-gate` registration evaluates conforming with
   `preregistration_matches=true`.
2. **§4.1 replica serving is real, attested, and measured.** With
   `head_replica_count=2`: population distributes all (shard, replica)
   pairs including coordinator-owned shards, routing never clamps
   (`head_replica_fallbacks=0`), and serving is active in the measured
   window — `head_replica_shards_served` 29/33/32 at 10k/50k/100k. The
   100k arm records a +25% single-stream latency cost; the arm is
   registered `context`, and promoting replica routing needs a
   contended-load case, which this packet does not claim.
3. **Three inert-mechanism layers found and fixed** (populate never called;
   GUC gated behind the legacy flag; counter blind to shard-cache hits) —
   each produced a green suite run and was caught only by activation
   counters. The manifest's defect ledger names all three; the reviewer's
   003a finding-3 concern ("a successful population call…") is addressed by
   complete-coverage attestation plus the now-visible serving counter.

## Still open (unchanged from 003a)

- The residual 53,440 bytes of empty-neighbour head-graph rows
  (reviewer question 1: eliminate vs retire as a justified bounded entry).
- The 10k/50k small-scale latency cost of sharding recorded in 003a
  (reviewer question 2).

Request open.
