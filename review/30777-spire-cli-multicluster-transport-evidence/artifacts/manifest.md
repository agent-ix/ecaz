# 30777 Artifact Manifest

- Head SHA: `f2950a3678c7c69cc5e114a22570f79b6167d16f`
- Packet: `30777-spire-cli-multicluster-transport-evidence`
- Timestamp: `2026-05-10T22:33:51Z`
- Lane: Phase 11 Stage E local multi-instance transport-overlap evidence
- Fixture: one coordinator plus two remote PG18 clusters
- Storage format: local SPIRE pg_test fixture rows from the transport-overlap harness
- Rerank mode: not applicable
- Surface style: CLI-owned local multi-instance runtime fixture

## Command

```text
target/debug/ecaz dev spire-multicluster transport-overlap-pg18 --artifact-dir review/30777-spire-cli-multicluster-transport-evidence/artifacts --run-id 30777 --skip-install
```

## Key Results

From `multicluster-transport-overlap.log`:

```text
transport_overlap_row=2,ready,none,0,304,304,3
transport_overlap_row=3,ready,none,0,3,3,3
slow_status=ready
slow_failure_category=none
fast_status=ready
fast_failure_category=none
fast_completed_before_slow=true
SPIRE multicluster PG18 transport overlap passed
```

PostgreSQL logs for coordinator, fast remote, and slow remote are stored in the
same artifact directory.

No full Stage E fault/lifecycle matrix coverage or product performance claim is
made by this packet.
