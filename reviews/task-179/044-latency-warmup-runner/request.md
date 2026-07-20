# Review request: canonical latency warmup support

## Scope

Please review runner commit `f11ffcafc`, landed separately before the physical
fanout A/B measurement.

The existing DistANN physical suite labeled latency `cache=warm` but measured
the first query on a new worker backend. Backend-local head/transport cache
construction therefore contaminated the sample and prevented a fully warmed
latency conclusion.

This checkpoint:

- adds `ecaz bench latency --warmup-iterations N`;
- runs N untimed queries on every worker connection after session/index setup
  and statement preparation but before timers and optional counters;
- preserves historical behavior at the default N=0;
- exposes `--benchmark-warmup-iterations` on the DistANN multi-instance fixture;
- passes that value to both physical and same-data latency arms; and
- exposes the field through `ecaz bench suite`, so the checked-in SuiteConfig
  and suite manifest retain the exact warmup count.

The physical summary line now records `warmup_iterations=N` alongside the
latency statistics.

## Validation

See `artifacts/manifest.md`. Focused latency tests and the DistANN suite command
expansion test pass, and the release `ecaz` runner builds successfully at the
exact implementation SHA.

The attempted repository-wide CLI clippy lane is also recorded: it is blocked
by pre-existing unrelated warnings in `ecaz-cloud`, corpus, SPIRE, and other CLI
modules. This checkpoint does not alter or suppress those findings.

## Requested decision

Please confirm the warmup occurs on the same worker connections used for timed
queries and is represented in suite provenance. The following A/B packet will
use this runner with a nonzero warmup count.
