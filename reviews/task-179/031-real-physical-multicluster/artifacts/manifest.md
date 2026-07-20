# Artifact manifest

- Head SHA: `2aabb45f9e3abcc058980c7308666897bc7ba6e8`
- Task bucket: `reviews/task-179/`
- Packet: `031-real-physical-multicluster`
- Lane: local PG18, synthetic physical-generation correctness fixture
- Fixture: one coordinator source table; one hidden generation per physical owner; no shared-table or replicated serving surface
- Storage format: `ec_distann distributed_control=true`, generation descriptor v2, manifest/fingerprint v2, WAL-logged owner graph/row/directory relations
- Rerank mode: physical frozen-row exact materialization through CustomScan
- Timestamp: `2026-07-12T05:00:39-07:00`

## Suite command

```text
target/debug/ecaz bench suite run \
  --config reviews/task-179/031-real-physical-multicluster/artifacts/physical-fixture-suite.json \
  --log-file reviews/task-179/031-real-physical-multicluster/artifacts/suite-run.log
```

The runner embedded the exact clean head SHA above. The checked-in config SHA-256 is
`ff9a790614c21ca5954254508601b7d7eb5314307e7d36b71328c8f44f8a4b67`.

## Artifacts

| Artifact | Purpose | Key result |
| --- | --- | --- |
| `physical-fixture-suite.json` | Canonical three-case `ecaz bench suite` config | one owner; three owners/coordinator in roster; three owners/coordinator outside roster |
| `suite-manifest.json` | Commands, exact runner SHA, status, duration, thresholds | all three steps succeeded; all three topology thresholds passed |
| `results.jsonl` | Normalized suite rows | 45 rows; Ready/Published topology and gate rows |
| `suite-run.log` | Complete suite driver output | exit 0; all physical serving and topology gates pass |
| `physical-one-owner/distann-local-multinode.log` | One-owner fixture log | 30/30 records/rows; zero residue/orphans; 10 rows served |
| `physical-one-owner/distann-multinode-summary.log` | One-owner decision-grade summary | topology gate pass, `remote_verified=0` |
| `physical-three-owner-in-roster/distann-local-multinode.log` | Three-owner in-roster fixture log | owner counts 33/24/33; two remote owners materialized |
| `physical-three-owner-in-roster/distann-multinode-summary.log` | In-roster decision-grade summary | Ready and Published global count 90; zero residue/orphans |
| `physical-three-owner-outside-roster/distann-local-multinode.log` | Four-process, three-owner outside-roster fixture log | no coordinator generation; all three owners materialized remotely |
| `physical-three-owner-outside-roster/distann-multinode-summary.log` | Outside-roster decision-grade summary | owner counts 33/24/33; `remote_verified=3` |
| `validation.log` | Focused compile/parser/audit validation summary | all commands passed |

PostgreSQL server logs were intentionally discarded as operational exhaust. No corpus TSV,
run directory, cache, or polling output is committed.

## Cited result lines

```text
physical-one-owner: Ready/Published records=30 rows=30 non_owned=0 orphans=0
physical-three-owner-in-roster: owner records=33/24/33, global=90, remote_verified=2
physical-three-owner-outside-roster: owner records=33/24/33, global=90, remote_verified=3
one-owner-topology-gate passed=true actual=1
three-owner-in-roster-topology-gate passed=true actual=1
three-owner-outside-roster-topology-gate passed=true actual=1
```

