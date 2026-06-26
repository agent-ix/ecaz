# Task 89 Review Request: IVF TQ+ Insert Drift

## Summary

This packet records the IVF TQ+ streaming-insert drift gate.

The suite shape is:

- Load staged DBPedia 50k as a deterministic source reservoir.
- Derive a 10k live TQ+ IVF index inside Postgres.
- Insert 10%, 25%, and 50% additional rows after the live index is built.
- Derive full-rebuild TQ+ baselines at 11k, 12.5k, and 15k rows.
- Compare recall@10 at `nprobe=48` for live-post-insert versus full rebuild.

The raw SQL files are packet-local under `artifacts/sql/` and the suite is
driven by `ecaz bench suite`.

## Result

The live post-build insert surface stays within the Task 89 drift thresholds.

| insert ratio | live rows | live recall@10 | rebuild recall@10 | live-minus-rebuild | threshold |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10% | 11,000 | 0.9265 | 0.9310 | -0.45 pp | informational |
| 25% | 12,500 | 0.9230 | 0.9235 | -0.05 pp | <= 0.5 pp |
| 50% | 15,000 | 0.9245 | 0.9220 | +0.25 pp | <= 1.0 pp |

At `nprobe=48`, drift is acceptable for the measured DBPedia live-insert
surface. This does not override the latency regressions observed in packets
003 and 004; it only answers the insert-drift gate.

## Validation

Completed in this packet:

- `./target/debug/ecaz bench suite audit --config reviews/task-89/005-ivf-tqplus-insert-drift/suite.json`
  passed: 14 steps.
- `./target/debug/ecaz bench suite run --config reviews/task-89/005-ivf-tqplus-insert-drift/suite.json --dry-run --manifest-output reviews/task-89/005-ivf-tqplus-insert-drift/artifacts/suite-manifest-dry-run.json`
  expanded the load, raw SQL, and recall steps.
- DBPedia insert-drift suite run completed through `ecaz bench suite` and
  wrote `artifacts/suite/results.jsonl`.

## Not Claimed

This packet does not close Task 89. Remaining gates after drift:

- At least one non-DBPedia corpus.
- Public-shape gate and closeout decision.
