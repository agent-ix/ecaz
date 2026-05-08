# SPIRE Scale Packet Artifact Manifest

Head SHA: to be filled by the measurement run.
Packet/topic: `30629-spire-scale-packet-runbook`

This manifest is a scaffold for the controlled AWS/RDS-class SPIRE scale
packet. Do not cite scale claims from this packet until the artifact rows below
are replaced with real run outputs.

| Artifact | Lane | Fixture | Storage format | Rerank mode | Command | Timestamp | Isolated one-index-per-table | Key result lines |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `load.log` | load | real10k or larger configured corpus | SPIRE profile value | profile value | `ecaz bench suite run --config crates/ecaz-cli/suites/task30-spire-real10k.json --only load --log-file review/30629-spire-scale-packet-runbook/artifacts/load.log` | pending | yes | pending |
| `storage.log` | storage | same loaded corpus | SPIRE profile value | profile value | `ecaz bench suite run --config crates/ecaz-cli/suites/task30-spire-real10k.json --only storage --log-file review/30629-spire-scale-packet-runbook/artifacts/storage.log` | pending | yes | pending |
| `explain.log` | planner | same loaded corpus | SPIRE profile value | profile value | `ecaz bench suite run --config crates/ecaz-cli/suites/task30-spire-real10k.json --only explain --log-file review/30629-spire-scale-packet-runbook/artifacts/explain.log` | pending | yes | pending |
| `latency.log` | latency | same loaded corpus | SPIRE profile value | profile value | `ecaz bench suite run --config crates/ecaz-cli/suites/task30-spire-real10k.json --only latency --log-file review/30629-spire-scale-packet-runbook/artifacts/latency.log` | pending | yes | pending |
| `recall.log` | recall | same loaded corpus | SPIRE profile value | profile value | `ecaz bench suite run --config crates/ecaz-cli/suites/task30-spire-real10k.json --only recall --log-file review/30629-spire-scale-packet-runbook/artifacts/recall.log` | pending | yes | pending |

## Required Environment Record

- Instance class:
- Storage class and IOPS:
- PostgreSQL version:
- Extension SHA:
- Dataset and row count:
- Query count:
- Warmup policy:
- Shared buffers:
- Maintenance work mem:
- Effective `ec_spire` reloptions:
- Comparison AMs and reloptions:

## Completion Gate

The request packet can mark the scale item complete only after this manifest
contains packet-local raw logs and key result lines for load, storage, explain,
latency, and recall.
