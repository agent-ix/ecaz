# Artifact manifest

- Runner binary SHA: `519177225a7e8a7ab4d8b85de5edc4a508477d2e`
- Task / packet: `task-179` / `070-system-column-latency-isolation`
- Runner: release `ecaz bench suite`; pre-provenance compatibility at
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
| before | `2af20eb0785e18f9f97504c8cb52740d2de85c28` | 40680-40682 | `before/suite-manifest.json` | 3/3 succeeded; final audit pass |
| after | `754eb7b911bf5aa5e2c6e7d4adb8213d03ff5b06` | 40690-40692 | `after/suite-manifest.json` | 3/3 succeeded; final audit pass |

Each arm's checked-in JSON config expands all commands into its suite
manifest. Final validation uses suite `status`, `audit`, and `report` against
the final manifest/config. Per-scale `distann-multinode-summary.log` files are
the compact raw source for every number cited in `../comparison.md`.

Raw PostgreSQL node logs, regenerable run directories, dry-run manifests, and
duplicate report-normalized outputs are pruned after final sealing.

## Key result lines

- 10k before -> after: recall `1.0000 -> 1.0000`, recall-workload mean
  `546.65 -> 546.54 ms`, warm physical p95 `51.9 -> 51.2 ms`, generation
  bytes `242745344 -> 242761728`, control bytes `24576 -> 24576`.
- 50k before -> after: recall `0.9800 -> 0.9800`, recall-workload mean
  `490.90 -> 570.25 ms`, warm physical p95 `67.6 -> 68.8 ms`, generation
  bytes `1242734592 -> 1242734592`, control bytes `24576 -> 24576`.
- 100k before -> after: recall `0.9500 -> 0.9500`, recall-workload mean
  `825.02 -> 921.95 ms`, warm physical p95 `66.9 -> 66.2 ms`, generation
  bytes `2496659456 -> 2496659456`, control bytes `24576 -> 24576`.
- Both suites completed 3/3 steps with zero failures, missing artifacts, or
  stale steps; both final audits passed all three steps.
