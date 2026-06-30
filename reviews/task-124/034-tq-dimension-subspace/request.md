# Review Request: Task 124 / 034 TQ Dimension/Subspace

## Summary

This slice addresses required Task 124 lever 4: **dimension/subspace reduction**.

Change:

- Added an ignored TQ2 dimension/subspace scorer profiler.
- Swept the same TQ2 QJL batch scorer across 1536, 1280, 1024, 768, 512, 384, and 256 dimensions.
- Reported scorer `ns/candidate` and compact payload code bytes at width 32 and width 100.

This is a TurboQuant scorer-path measurement of fewer dimensions. It does not introduce a production reduced-dimension index format or reloption; recall/product behavior is unchanged.

## TQ-Internal Result

Primary width-100 rows from `artifacts/tq2-dimension-profile.log` on local arm64/NEON:

| Dimension | Code bytes | TQ2 scorer ns/candidate | Delta vs 1536 |
| ---: | ---: | ---: | ---: |
| 1536 | 384 | 330.2 | baseline |
| 1280 | 320 | 276.2 | -16.4% |
| 1024 | 256 | 218.4 | -33.9% |
| 768 | 192 | 163.1 | -50.6% |
| 512 | 128 | 111.7 | -66.2% |
| 384 | 96 | 82.4 | -75.0% |
| 256 | 64 | 56.8 | -82.8% |

Width-32 rows show the same monotonic scorer scaling: 1536 dims `412.3 ns/candidate`, 768 dims `143.2`, and 512 dims `96.0`.

Conclusion: the scorer-speed lever is real and roughly tracks reduced payload/code dimensions. It is not enabled in production in this slice because a reduced-dimension TQ sidecar needs a separate recall/format contract.

## Validation

Passed:

- `cargo fmt --check`
- `ECAZ_TQ2_DIM_PROFILE_LOG=reviews/task-124/034-tq-dimension-subspace/artifacts/tq2-dimension-profile.log cargo test --release --lib --features bench task124_profile_tq2_dimension_sweep -- --ignored --nocapture`

Artifact details are in `artifacts/manifest.md`.

## Task Status

This completes the dimension/subspace-reduction scorer measurement for lever 4. The seven reopened Task 124 scorer levers have now all been touched with TQ-internal scorer/query-prep evidence in packets 028-034, but this packet does not claim product closeout or merge readiness.
