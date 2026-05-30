# Current Benchmark Lanes

This directory holds the promoted current benchmark state for repeatable
engineering comparisons. These lanes are mutable by design: update them when a
new packeted benchmark run becomes the accepted current result for that host
class.

Immutable evidence still lives in benchmark or review packets. A current lane
must cite the source packet, head SHA, suite config path, suite manifest,
result files, host metadata, cache policy, and claim class before its numbers
are used in docs, task status, or review rationale.

## Lanes

| Lane | Host class | Standard suite |
| --- | --- | --- |
| `m5-local` | Apple Silicon M5 local development host | `crates/ecaz-cli/suites/current/m5-local.json` |
| `intel-local` | Local Intel desktop/workstation | `crates/ecaz-cli/suites/current/intel-local.json` |
| `aws-intel` | AWS Intel x86_64 benchmark host | `crates/ecaz-cli/suites/current/aws-intel.json` |
| `aws-graviton` | AWS Graviton arm64 benchmark host | `crates/ecaz-cli/suites/current/aws-graviton.json` |

## Lane Layout

Each populated lane uses:

- `manifest.md`: human source of truth for promoted current state.
- `suite-manifest.json`: `ecaz bench suite run` provenance.
- `results.jsonl`: normalized rows from the suite run.
- `results-report.jsonl`: normalized rows from `ecaz bench suite report`.
- `artifacts/`: raw per-step logs and generated SQL.

When refreshing a lane, run the standard suite with lane-local outputs:

```sh
ecaz bench suite audit --config crates/ecaz-cli/suites/current/m5-local.json

ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 \
  bench suite run \
  --config crates/ecaz-cli/suites/current/m5-local.json \
  --artifact-dir benchmarks/current/m5-local/artifacts \
  --manifest-output benchmarks/current/m5-local/suite-manifest.json \
  --results-output benchmarks/current/m5-local/results.jsonl

ecaz bench suite report \
  --manifest benchmarks/current/m5-local/suite-manifest.json \
  --results-output benchmarks/current/m5-local/results-report.jsonl
```

Task packets may run the same suite with `--artifact-dir` pointing at the
packet `artifacts/` directory. Promote to `benchmarks/current/<lane>/` only
after the packet result is accepted as the current comparison point.
