# Task 185 fixed-cap screen manifest

- Task bucket / packet: `reviews/task-185/003-fixed-cap-screen/`
- Status: preregistration plus isolated-attribution implementation; no screen
  benchmark artifacts captured
- Implementation head: `917995972c53f02285cf4683dcb435a5c5b69bcd`
- Planned first scale: 100k (`ec_real_100k`)
- Planned topology: three-owner physical sharded generation, one index per
  arm, run directories under `/home/peter/.ecaz/clusters/`
- Planned controls: cap 4096, exact head scoring, 32 seeds, graph degree 32,
  BW4/H100, RaBitQ traversal, exact final ranking
- Planned split: training rows 201--400; held-out evaluation rows 1--200
- Required runner: `ecaz bench suite` with a packet-local SuiteConfig

The implementation and candidate-level attribution contract must be reviewed
and committed before any run is added here. Corpus TSVs, truth caches, cluster
directories, and operational logs are not packet evidence and must not be
committed.

## Validation artifacts

- `pg18-feature-check.log` — feature build with the benchmark endpoint enabled;
  SHA-256 `5936d6c6573e92774a2b726ce8d06377e875d0d25ba115cee1fc27a365473f26`
- `pg18-featureless-check.log` — production-shaped featureless build;
  SHA-256 `2f011af5f7c379f486c6e8d990edc82a9d2b8255691e115763bde7c9eb299b3e`

The isolated endpoint currently covers one member at a time from the control's
returned seed list. It is not an alternate-head selector and no benchmark
result is claimed from it.
