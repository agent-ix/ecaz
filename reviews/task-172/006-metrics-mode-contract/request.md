---
task: 172
packet: 006-metrics-mode-contract
role: coder
status: review-requested
head: 854c6be176c0d4cd0dddac14b3ba035867c4c148
date: 2026-07-29
---

# Review request: DistANN metrics-mode contract

## Requested decision

Please review commits `8942413d6` and `854c6be17`, which make Task 172's
benchmark-versus-full-metrics distinction a first-class
`distann-local-multinode` suite contract.

This is a runner-capability checkpoint. It does not run or promote the final
physical matrix and does not claim Task 172 complete.

## Scope

The DistANN multinode suite step accepts:

```json
{"metrics_mode": "benchmark"}
```

or:

```json
{"metrics_mode": "full_metrics"}
```

The modes behave as follows:

| Mode | Stage attribution | Backend RSS/HWM sampling | Gate use |
| --- | --- | --- | --- |
| `benchmark` | off | off | primary recall/latency/throughput rows |
| `full_metrics` | on | on, 25 ms default interval | diagnostic attribution and scaling |

An explicit benchmark step fails configuration validation if it also enables
heavy instrumentation. An explicit full-metrics step enables both existing
first-class instrumentation surfaces.

For compatibility, configs without `metrics_mode` retain their exact execution
flags. Their effective label is derived as `full_metrics` when any existing
heavy-instrumentation flag is active and `benchmark` otherwise. Legacy stage
counter configs are not silently changed to add memory sampling.

## Evidence labeling

Every DistANN multinode `StepRecord` in `suite-manifest.json` receives a
computed `metrics_mode=<mode>` tag. The result-context normalizer copies that
label into every parsed `results.jsonl` row for the step.

User-provided `metrics_mode=` tags are discarded and replaced by the computed
value, so a tag cannot mislabel the actual execution flags.

## Task 172 coverage

This checkpoint provides:

- an explicit lean gate mode;
- an explicit instrumentation-heavy diagnostic mode;
- fail-closed rejection of a benchmark row carrying heavy flags;
- durable manifest and normalized-row labels; and
- backward-compatible classification of previously checked-in suite configs.

The required representative benchmark/full overhead A/B remains a measurement
deliverable for the final Task 172 packet.

## Validation

See `artifacts/validation.md` and `artifacts/manifest.md`.

- Two focused suite tests pass.
- The previously blocked Task 172 throughput unit test also passes after the
  Task 205 compile fix was integrated.
- Targeted formatting and the relevant Clippy lint pass.
- The repository-wide lint target reaches unrelated pre-existing
  `ec_ivf/quantizer.rs` and stops on `clippy::manual_checked_ops`.

No benchmark or cluster was run.

## Reviewer focus

1. Confirm the instrumentation split is strong enough to prevent diagnostic
   latency from being mistaken for gate latency.
2. Confirm deriving the legacy label without changing legacy execution is the
   correct compatibility behavior.
3. Confirm the manifest-tag/result-context path makes every normalized metric
   row mechanically attributable to one mode.
