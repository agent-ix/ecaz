# Task 208 / 001 artifact manifest

- Head SHA: `dece67f96a05e1b549c5cd384426b39bc101d260`
- Task bucket / packet: `reviews/task-208/001-gates/`
- Timestamp: `2026-07-30T11:30:35-07:00`
- Lane: PG18 `distann-local-multinode`, physically sharded owner traversal plus
  the FR-084 coordinator-replica negative fixture
- Corpus scales: staged real DBpedia 10k / 50k / 100k for the conforming owner
  replay; 100k for the known nonconforming replica replay
- Storage format: RabitQ neighbor codes with co-placed full-precision row tier
- Rerank mode: default exact co-located rerank
- Surface isolation: Task 205 source artifacts use isolated three-owner
  generations per step. Task 204's two variants share one immutable physical
  generation; the coordinator-replica variant adds its separately measured
  derived relation.

## Source artifacts

The replay reads committed packet-local summaries; it does not rerun a
benchmark or rely on a local cluster.

| source | SHA-256 | purpose |
|---|---|---|
| `reviews/task-205/003-ab/artifacts/run-candidate-stage2/suite-manifest.json` | `45e7a782dd4edb78108fa0bf1c3c8a37a1fc5c06eed2157bb737542bcd7e1cc0` | three-scale owner control and candidate |
| `reviews/task-204/001-arm-fidelity/artifacts/run-final/suite-manifest.json` | `38c27e950952f60429c8a9a019cec63ea1b2fa84abae1f4a657f772dde7b1d7b` | 100k FR-084 replica negative |

The manifests point to their packet-local
`distann-multinode-summary.log` files. No corpus, PGDATA, cache, tunnel, or
polling artifact is committed here.

## Replay commands

The source manifests predate the registration schema. For replay only, `jq`
added `nfr_021_registrations` to temporary copies under `/private/tmp`, without
altering the source evidence. The Task 205 control/candidate ids were registered
as conforming decision arms. The Task 204 owner and replica were registered as
context; the replica preregistration was nonconforming.

The derived rows were then regenerated with:

```text
target/debug/ecaz bench suite report \
  --manifest /private/tmp/task205-nfr021-manifest.json \
  --results-output /private/tmp/task205-nfr021-results.jsonl

target/debug/ecaz bench suite report \
  --manifest /private/tmp/task204-nfr021-manifest.json \
  --results-output /private/tmp/task204-nfr021-results.jsonl
```

The temporary transformed manifests are not durable evidence. Their inputs,
registration values, source hashes, and resulting decision rows are recorded in
this packet; the underlying measurements remain the cited immutable source
packets.

## Key result lines

- Owner control: `actual_admissibility=conforming`,
  `normalized_bytes_per_owned_record_growth_max=1.094675`,
  `raw_fixed_roster_graph_side_growth_max=11.117647`,
  `max_unsharded_derived_bytes=0`.
- Algorithm 1 candidate: same conformance and growth values.
- Coordinator replica: `actual_admissibility=nonconforming`,
  `max_unsharded_derived_bytes=1659518976`,
  `preregistration_matches=true`.

The exact compact rows are in `artifacts/nfr021-replay-results.jsonl`.
