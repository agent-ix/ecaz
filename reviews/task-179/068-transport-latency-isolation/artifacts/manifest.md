# Artifact manifest

- Runner head SHA: `519177225a7e8a7ab4d8b85de5edc4a508477d2e`
- Task / packet: `task-179` / `068-transport-latency-isolation`
- Runner: release `ecaz bench suite`; pre-provenance compatibility code at
  `f51d512cb`
- Host lane: local Intel, PG18
- Fixture: staged real corpus `ec_real_{10k,50k,100k}` under
  `/home/peter/dev/ecaz/data/staged-current`
- Format / rerank: physical DistANN generation, persisted-head search, no
  separate rerank variant
- Isolation: one index per physical owner table plus a single-index control;
  no shared-table measurement surface
- Common shape: 3 owners, degree 32, head cap 4096, BW4/H100, top-k 10, 20
  queries, 200 recall trials, 10 warmups, 30 measured latency iterations
- Timestamp: suite-generated Unix-millisecond timestamps are embedded in each
  final suite manifest; runs occurred 2026-07-14 PDT
- Complete artifact digests: `checksums.sha256`

## Arms

| Arm | Installed extension | Ports | Final manifest | Result |
| --- | --- | --- | --- | --- |
| before | `9a0f21f0824c675d06e9e87747eb36a70859611f` | 40660-40662 | `before/suite-manifest.json` | 3/3 succeeded; 6/6 thresholds pass |
| after | `ceb15f73ac69fcd98896457c9578fadae2ff0c09` | 40670-40672 | `after/suite-manifest.json` | 3/3 succeeded; 6/6 thresholds pass |

Each arm's checked-in JSON config expands all commands into its suite
manifest. Final validation uses:

```text
target/release/ecaz bench suite status --manifest <manifest> --log-file <status>
target/release/ecaz bench suite audit --config <config> --log-file <audit>
target/release/ecaz bench suite report --manifest <manifest> --results-output <normalized> --log-file <report>
```

The report-generated normalized copies are pruned after comparison because
the canonical `results.jsonl`, suite report, and per-scale
`distann-multinode-summary.log` files retain the same parsed evidence. Raw
PostgreSQL node logs and regenerable run directories are also pruned under the
repository packet rules.

## Before key results

- Recall: `1.0000 / 0.9800 / 0.9500` at 10k/50k/100k.
- Recall-workload mean: `747.71 / 558.10 / 816.38` ms.
- Warm physical p95: `92.5 / 120.8 / 116.8` ms.
- Physical generation bytes:
  `242,794,496 / 1,242,734,592 / 2,496,659,456`.
- Aggregate control index bytes: `24,576` at every scale.
- Topology: zero non-owned rows and zero orphans; two remote owners engaged at
  every scale.

## After key results

- Recall: `1.0000 / 0.9800 / 0.9500` at 10k/50k/100k.
- Recall-workload mean: `588.77 / 588.97 / 936.25` ms.
- Warm physical p95: `84.8 / 115.8 / 111.4` ms.
- Physical generation bytes:
  `242,761,728 / 1,242,734,592 / 2,496,659,456`.
- Aggregate control index bytes: `24,576` at every scale.
- Topology: zero non-owned rows and zero orphans; two remote owners engaged at
  every scale.

The comparison and its deliberately bounded timing interpretation are in
`../comparison.md`.
