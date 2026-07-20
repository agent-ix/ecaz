# Artifact manifest

- Task / packet: `task-179` / `072-final-signoff-remediation`
- Code head and suite runner: `45491d1052ef0369a9f418b055b462663cf5612c`
- Host lane: local Intel desktop, PG18
- Fixture: staged real corpora `ec_real_{10k,50k,100k}` under `/home/peter/dev/ecaz/data/staged-current`
- Storage / rerank: physical DistANN hash-shard generations, persisted-head search, no separate rerank variant
- Isolation: one index per physical owner table plus a distinct single-index control; no shared-table measurement surface
- Common shape: 3 owners, degree 32, head cap 4096, BW4/H100, top-k 10, 20 queries, 200 recall trials, 10 warmups, 30 measured latency iterations
- Run date: 2026-07-14 PDT; exact Unix-millisecond timestamps and durations are embedded in each final suite manifest
- Complete artifact digests: `checksums.sha256`

The checked-in `exact-ab-suite.json` is the sole matrix driver. Every measurement arm was invoked with:

```text
target/release/ecaz bench suite run \
  --config reviews/task-179/072-final-signoff-remediation/artifacts/exact-ab-suite.json \
  --artifact-dir reviews/task-179/072-final-signoff-remediation/artifacts/<arm> \
  --log-file reviews/task-179/072-final-signoff-remediation/artifacts/<arm>-suite.log
```

The runner expands the exact per-scale commands into each arm's `suite-manifest.json`. The retained `results.jsonl` and `suite-report.md` are normalized from the compact summaries, not manually transcribed.

## Exact A/B arms

| Pair | Arm | Installed extension SHA | Artifact root | Result |
| --- | --- | --- | --- | --- |
| refactor isolation | before | `59da26b8e02314f8f10d737b0a101f8e6e1d41e4` | `refactor-before/` | 3/3 succeeded; post-prune 0 missing / 0 stale; audit passed |
| refactor isolation | after | `0043c3e746bef0baf6977dc8ae426006d7a0a887` | `refactor-after/` | 3/3 succeeded; post-prune 0 missing / 0 stale; audit passed |
| final remediation | before | `34b61fb3c55d0333cec2213c6714858dd5b43e68` | `remediation-before/` | 3/3 succeeded; post-prune 0 missing / 0 stale; audit passed |
| final remediation | after | `45491d1052ef0369a9f418b055b462663cf5612c` | `remediation-after/` | Clean rerun 3/3 succeeded; post-prune 0 missing / 0 stale; audit passed |

Each completed clean scale's `physical_benchmark_provenance` row queried the installed extension on the coordinator and all owner ports. Every retained row records the SHA named above, `extension_build_profile=release`, `nodes=3`, and `unanimous=true`. This is independent of the suite manifest's runner SHA. The earlier interrupted remediation-after output is not retained or used; the retained arm is a fresh, non-resumed rerun.

## Artifact inventory and provenance

- `exact-ab-suite.json`: checked-in `SuiteConfig` for all four 10k/50k/100k arms.
- `dry-run/suite-manifest.json` and `suite-dry-run.log`: command-expansion proof produced by `ecaz bench suite run --dry-run` at runner head `45491d105`.
- `<arm>/{10k,50k,100k}/distann-multinode-summary.log`: compact raw source for every topology, extension-provenance, recall, latency, and storage value cited by `comparison.md`.
- `<arm>/suite-manifest.json`: runner SHA, config digest, fully expanded commands, step timestamps/durations, statuses, expected compact artifacts, and threshold outcomes.
- `<arm>/results.jsonl`: normalized metric rows parsed from the retained summaries.
- `<arm>/suite-status.log`: post-prune validation of completion plus missing/stale artifact counts.
- `<arm>/suite-report.md`: post-prune reconstruction of normalized results and thresholds.
- `<arm>/suite-audit-post-prune.log`: config/input-shape audit run after pruning.
- `*-install.log`: exact pgrx release installation output for each installed extension SHA.
- `*-suite.log`: suite-driver output; the per-owner PostgreSQL/run logs it referenced were pruned as regenerable operational exhaust.
- `focused-pg18.log`: real PG18 `test_distann_multi_epoch_publish` run at code head `45491d105`, including the mid-scan ERROR and retained-cache eviction coverage.
- `runner-build.log`: release CLI build used for all structured suite runs.
- `finding-disposition.md`: exhaustive mapping of packet-071 findings to code and evidence.
- `comparison.md`: exact A/B numeric comparison and decision.
- `remediation-after-tainted.md`: interruption record explaining why the first remediation-after attempt is excluded from evidence.

## Compact post-prune semantics

Historical packets' pre-prune `suite-audit-final.log` files cannot prove that their later-pruned expected files still existed; that limitation is not reconstructed. In this packet, `compact_artifacts: true` makes the summary the only expected per-step durable artifact. Post-prune `suite status` establishes 0 missing / 0 stale and `suite report` proves result reconstruction; `suite audit` separately validates the config and its required inputs.

## Key results

- Refactor isolation recall is identical at 10k/50k/100k (`1.0000 / 0.9800 / 0.9500`); physical p95 changes by `-0.4 / -0.7 / +1.1 ms`; physical bytes change by `0 / -16384 / +32768`.
- Final remediation recall is identical at 10k/50k/100k (`1.0000 / 0.9800 / 0.9500`); physical p95 changes by `-0.1 / +1.3 / +0.3 ms`; physical bytes change by `-16384 / +8192 / 0`. These are run/page-level variance, not evidence of a performance regression or gain.
