# Task 144 Packet 002 Review Request: Closure Geometry Simulation

## Summary

This slice extends the Task 144 Phase 0 diagnostic added in packet 001. The
geometry JSONL can now include closure-simulated true-neighbor concentration for
pre-registered epsilon values:

```json
{
  "geometry_output": "${artifact_dir}/geometry.jsonl",
  "geometry_closure_epsilon": [0.05, 0.1, 0.2]
}
```

The emitted rows use `mode="closure_simulated_ip_distance_ratio"` and include
the `closure_epsilon` value, so packet-local suite output can compare current
single-assignment concentration with simulated closure concentration.

## Validation

- `cargo test -p ecaz-cli spire_pipeline --no-default-features`
- Result: `30 passed; 0 failed; 0 ignored; 409 filtered out`
- Log: `artifacts/cargo-test-ecaz-cli-spire-pipeline.log`

## Review Focus

Please review the diagnostic boundary:

- closure simulation is intentionally read-only and does not change index build
  or scan behavior;
- the distance proxy is explicit: `max(0, 1 - dot(vector, centroid))`;
- suite configs can now pre-register epsilon values and keep the resulting
  JSONL as packet-local evidence.

The next Task 144 slice should run the Phase 0 release suite over real cells or
move into gated build/query options after reviewer feedback.
