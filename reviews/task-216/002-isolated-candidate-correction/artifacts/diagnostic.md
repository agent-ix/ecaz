# Task 216 over-materialization diagnostic

This is a source-and-log audit of the existing Task 216 attribution run; it is
not a new benchmark.

## Arm mode

The packet-local suite command contains:

```text
--benchmark-seed-variant production-control:persisted_head:32:32:rabitq:0:off:4:100:off
```

The sixth field is the materialization batch size. The runner's parser accepts
zero as the explicit eager arm (`crates/ecaz-cli/src/commands/dev/distann_multicluster.rs:1881-1892`),
and the runner appends the corresponding session GUC
(`crates/ecaz-cli/src/commands/dev/distann_multicluster.rs:1536-1545`). The
production constant is lazy-10, while the benchmark GUC is feature-gated
(`src/am/ec_distann/options.rs:100-106,749-756`).

The captured summary independently reports:

```text
materialization_batch_size=0
remote_candidates_requested mean_per_scan=31.340000
remote_payloads_installed mean_per_scan=31.340000
executor_remote_rows_consumed mean_per_scan=6.660000
```

Therefore the 31.34-versus-6.66 gap is the eager-control behavior, not a
measurement of lazy-10 over-materialization.

## Stage and build mode

The captured log reports `extension_build_profile=release`, but it also emits
`distann-stage-counters`; the counter-producing scan paths are guarded by
`distann-head-attribution-benchmark` (for example,
`src/am/ec_distann/custom_scan.rs:1105-1155`). This is a cargo release-profile
feature build, not the featureless normal-release latency configuration.

The same log reports:

```text
materialize_coordinator_decode mean_ms=0.074668
materialize_owner_payload_sql_work mean_ms=39.380697
```

The accepted isolated control uses the rounded 0.076 ms / 40.60 ms values for
the maximum-win screen: `0.076 / 40.60 = 0.19%`. The owner-side SQL region is
not a coordinator decode ceiling and remains a separate candidate surface.
