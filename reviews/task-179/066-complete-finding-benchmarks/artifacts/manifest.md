# Artifact manifest

- Head SHA: `45f9f0f980d9548083aa965659df4de7089b6e18`
- Task / packet: `task-179` / `066-complete-finding-benchmarks`
- Runner: release `ecaz bench suite` at `45f9f0f98`
- Host lane: local Intel, PG18
- Fixture: staged real corpus `ec_real_{10k,50k,100k}` under
  `/home/peter/dev/ecaz/data/staged-current`
- Format / rerank: physical DistANN generation, persisted-head search, no
  separate rerank variant
- Isolation: one index per physical owner table plus a single-index control;
  no shared-table measurement surface
- Common shape: 3 owners, degree 32, head cap 4096, top-k 10, 20 queries, 200
  recall trials, 10 warmups, 30 measured latency iterations
- Timestamp: suite-generated Unix-millisecond timestamps are embedded in each
  suite manifest; runs occurred 2026-07-13 through 2026-07-14 PDT
- Complete artifact digests: `checksums.sha256`

## Arms

| Arm | Installed extension | Topology | Search shape | Final manifest | Result |
| --- | --- | --- | --- | --- | --- |
| perf baseline | `0b2d4fbabedb4caa59535b875b81359b4dd6f91c` | coordinator is owner 1; 2 remote owners | BW4/H100 | `perf-baseline/suite-manifest.json` | 3/3 succeeded; 9/9 thresholds pass |
| candidate default | `45f9f0f980d9548083aa965659df4de7089b6e18` | coordinator is owner 1; 2 remote owners | BW4/H100 | `candidate-default/suite-manifest.json` | 3/3 succeeded; 9/9 thresholds pass |
| candidate shape | `45f9f0f980d9548083aa965659df4de7089b6e18` | coordinator is owner 1; 2 remote owners | BW16/H25 | `candidate-bw16h25/suite-manifest-resume.json` | 3/3 succeeded after manifest resume; 9/9 thresholds pass |
| outside roster | `45f9f0f980d9548083aa965659df4de7089b6e18` | separate coordinator; 3 remote owners | BW4/H100 | `outside-roster/suite-manifest.json` | 3/3 succeeded; 9/9 thresholds pass |

Each arm's JSON config is checked in at the artifact root. The exact expanded
commands are recorded in its suite manifest and status log. Final validation
used, per arm:

```text
target/release/ecaz bench suite status --manifest <manifest> --log-file <status>
target/release/ecaz bench suite audit --config <config> --log-file <audit>
target/release/ecaz bench suite report --manifest <manifest> --results-output <normalized> --log-file <report>
```

The report-generated normalized copy was pruned because the canonical
`results.jsonl` (or `results-resume.jsonl`) and markdown report retain the same
parsed rows. Per-scale `distann-multinode-summary.log` files are the compact
raw source for every number cited in `comparison.md`.

## Key cited results

- Baseline -> candidate BW4/H100 physical p95:
  10k `53.6 -> 45.9`, 50k `67.8 -> 59.0`, 100k `68.5 -> 58.3` ms.
- Baseline -> candidate recall-workload mean:
  10k `604.22 -> 53.21`, 50k `612.36 -> 65.41`,
  100k `1075.80 -> 60.45` ms; physical recall is unchanged at
  `1.0000/0.9800/0.9500`.
- Candidate BW4/H100 -> BW16/H25 p95:
  10k `45.9 -> 64.5`, 50k `59.0 -> 113.0`,
  100k `58.3 -> 92.1` ms; recall changes to
  `1.0000/0.9950/0.9600`.
- Outside-roster engagement is `3/3/3` remote owners with recall
  `1.0000/0.9800/0.9500` and p95 `45.3/65.3/58.7` ms.
- Aggregate `control_index_bytes` is `24,576` for every arm and scale.

## Failure and cleanup provenance

The initial BW16/H25 100k step failed after Published topology with PostgreSQL
`No space left on device` while constructing the single-index control. The
initial `suite-manifest.json`, `results.jsonl`, and suite-run log preserve that
attempt; `candidate-bw16h25/failure-disposition.md` records the exact retained
error. After deleting only regenerable Task 179 PostgreSQL run directories,
the runner reused the successful 10k/50k records with `--resume-from` and
reran 100k. `suite-manifest-resume.json`, `results-resume.jsonl`, and the
resume log are the accepted final source.
