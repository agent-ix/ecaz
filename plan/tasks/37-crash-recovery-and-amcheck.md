# Task 37: Crash Recovery, WAL Replay, and `pg_amcheck` Integration

Status: **proposed** — owns the PG-extension-specific safety net that Task 34
explicitly does not cover. Without this lane there is no automated proof that
an ECAZ index survives backend crashes, server restarts, or replication
replay.

## Scope

Add a PG18 integration lane that:

1. Drives realistic workloads against ECAZ indexes (HNSW, IVF, DiskANN, SPIRE).
2. Crashes the cluster mid-workload at adversarial points.
3. Restarts and verifies that the index is recoverable and correct using
   `pg_amcheck`, recall probes, and structural invariants.

Coverage targets:

- WAL records emitted by each AM during INSERT, UPDATE, DELETE, VACUUM, and
  background merges (SPIRE).
- Crash points:
  - mid-build (`CREATE INDEX` interrupted between page allocations),
  - mid-insert (between heap WAL and index WAL),
  - mid-VACUUM (between tuple removal and index cleanup),
  - mid-REINDEX CONCURRENTLY,
  - mid-merge for SPIRE (between coordinator state writes).
- Recovery surfaces:
  - cold restart from data directory,
  - PITR replay from base backup + WAL,
  - streaming replica catch-up,
  - logical replication if/when ECAZ becomes logical-decoding-aware.

## Why

ECAZ ships custom access methods. A bug in WAL record encoding, redo
implementation, or buffer eviction order causes silent index corruption that
only surfaces after a real crash — long after the change that introduced it.
Today nothing in the test stack restarts Postgres between writes:

- pgrx integration tests run in a single backend without crash injection.
- Task 34's sanitizer and fuzz lanes are local-process and never touch the
  WAL pipeline.
- `pg_amcheck` is not invoked anywhere.

This is the single largest unmitigated correctness risk for a custom AM. Even
trivial implementations have had data-loss bugs caught only by crash-recovery
harnesses (Postgres core itself uses `recovery_test.pl` and TAP tests for
exactly this reason).

## Approach

1. **Crash harness.** Add `ecaz dev crash-recovery` or a TAP-style test driver
   under `crates/ecaz-cli` that:
   - Spawns a PG18 cluster.
   - Loads a corpus and creates an ECAZ index of the requested AM.
   - Runs a workload (insert / update / delete / scan / vacuum) under a seeded
     RNG.
   - Sends `SIGKILL` (or `pg_ctl stop -m immediate`) at a deterministic point
     selected by a `--crash-after` argument or a `gdb`/`stap`-style probe.
   - Restarts the cluster and waits for recovery to complete.
   - Verifies: `pg_amcheck --all`, scan results match an in-memory ground
     truth, recall ≥ floor, no `WARNING` / `ERROR` in the post-recovery log.
2. **`pg_amcheck` extension.** ECAZ AMs need to implement the `amcheck`
   callback (`amcheckindex` / `bt_index_check`-equivalent for each AM). Where
   `amcheck` is not yet wired, define a packet-owned `ecaz amcheck` that walks
   pages and asserts:
   - graph invariants (no orphan neighbors, no cycles where forbidden),
   - SPIRE partition object invariants (epoch monotonicity, no overlapping
     segments, leaf V2 metadata length matches segment count),
   - DiskANN metadata page checksums and tuple counts,
   - quantizer codebook fingerprint matches the stored codebook.
3. **Probe points.** Generate the list of crash points from instrumentation:
   add `#[cfg(feature = "crash_probes")]` `crash_probe!("name")` macros at
   every WAL boundary; the harness reads the probe list at runtime and
   schedules crashes at each one in turn.
4. **Workload generators.** Reuse the existing corpus generator and add:
   - mixed read/write workloads with configurable conflict rate,
   - VACUUM concurrent with INSERT,
   - REINDEX CONCURRENTLY concurrent with SCAN.
5. **Replica path.** Add a second cluster instance acting as a streaming
   standby; assert recall and `pg_amcheck` parity after promotion.
6. **Make lanes:**
   - `make crash-recovery-smoke` — single crash point per AM, 30s workload.
   - `make crash-recovery-full` — sweep every probe point, nightly.

## Validation

- Smoke lane: at least one crash point per AM passes restart + `pg_amcheck`.
- Full lane: every probe point survives at least 5 seeded reruns.
- A deliberately mutated WAL record (e.g., wrong record length) is caught by
  the harness — that mutation should be part of the validation suite, not the
  test suite, to prove the lane fails when it should.
- Replica path: streaming standby converges and matches the primary's
  `pg_amcheck` output for every AM.

## Exit Criteria

- `make crash-recovery-smoke` runs in CI nightly with green status across all
  four AMs.
- `make crash-recovery-full` documented and runnable; failures land in
  `review/<packet>/artifacts/`.
- Each ECAZ AM exposes an `amcheck`-style entry point.
- A short doc under `docs/crash-recovery.md` describes:
  - the probe-point model,
  - how to add a probe for new code,
  - how to interpret a crash-recovery failure,
  - the known set of probe points and their last-passing SHA.

## Dependencies

- Requires a live PG18 cluster; runs in the same environment as Task 34's
  deferred `sqlsmith-pg18` and `sanitizer-pg18-*` lanes.
- Naturally pairs with Task 38 (PG-level fault injection) — that task generalizes
  the crash mechanism to I/O, OOM, and statement cancellation.
- Should land before any future replication / HA work.
