# Review request: operations, lifecycle, and isolation

## Scope

Please re-review Task 199 packet 002 through exact release SHA
`2a4a70b23161f556c44d6d1d2c960541fbcb1bdb`.

This checkpoint incorporates the fixes requested in
`feedback/2026-07-25-03-reviewer.md`:

- `8b6a00e9f` makes the post-lock Ready lookup genuinely latest-snapshot and
  adds a stale RR/SERIALIZABLE discriminator;
- `66992cfbe` completes the create-time and mid-copy ENOSPC drills without
  transactional tablespace cleanup;
- `0c4bf83f0` restores the no-replica per-tuple fast path after a measured
  66.7% regression exposed by packet 003's historical baseline;
- `2a4a70b23` prevents the statement-level RR/SERIALIZABLE guard from
  populating that cache through its fixed transaction snapshot.

The final suite ran from a clean detached worktree, so the runner, extension,
and all three PostgreSQL nodes report the same exact SHA with no `-dirty`
suffix.

## Result

The checked-in normal-release PG18 suite completed its operations step with
`completed=1`, `failed=0`, `missing_artifacts=0`, and `stale=0`.

The two seq-03 blockers are now directly discriminated:

- Stronger-isolation mutation starts RR/SERIALIZABLE, assigns an XID and fixes
  the transaction snapshot, lets another session commit Ready, then reaches
  the ordinary mutation front door. Both cases return SQLSTATE `40001` /
  `EC_REPLICA_INVALIDATED`, insert zero rows, and rebuild between cases.
- ENOSPC is injected both at replica relation creation (`op=open`) and after a
  `Building` row exists during copy (`op=pwrite count=2`). Both return `53100`,
  leave zero catalog/relation residue and zero eligible partial images,
  preserve owner fallback, and recover to Ready. The packet-local marker is
  the raw provider witness.

The no-replica guard optimization preserves those semantics while restoring
throughput. The identical five-trial / 2,000-row workload measured
`2481.671 rows/s` at this final SHA versus `2315.234 rows/s` at the
pre-Task-199 parent `ebf9950c1` (packet 003), a 7.2% improvement rather than a
regression.

The remainder of the lifecycle surface also passes: blocking build/mutation,
real INSERT/DELETE/participant tombstone invalidation, VACUUM disposition,
post-control-commit backend termination, in-flight cursor completion,
authentication failure and backend/build suppression, relation-drop and
queued-DDL races, restart/outage/corruption/removed-image fallback,
retire/reclaim, successor epoch turnover, normal-build feature isolation, and
seven materialization semantic cases.

The lifecycle-only recall/latency diagnostic remains deliberately small:
owner and replica recall are both `0.9900`; two-sample warm means are
19.80 ms owner and 15.20 ms replica. Packet 003 owns the decision-grade
200-query / 50-sample matrix.

## Requested review

Please confirm:

1. the raw `SPI_execute_snapshot(..., read_only=true)` lookup plus the
   discriminating stale-snapshot drill closes seq-03 P1-B;
2. the cache fast path remains coherent: only a known negative can bypass
   snapshot registration, while an unknown statement-level state reaches the
   latest-snapshot per-index guard;
3. the create and data-write ENOSPC arms, explicit `53100` assertions, raw
   marker, zero residue, and recovery build close seq-03 P2-G/P3-C/P3-D;
4. packet 002 can be accepted subject only to packet 003's final release
   decision and cross-ISA/no-replica evidence.

See `artifacts/manifest.md` for commands, committed-blob hashes, provenance,
and cited lines.
