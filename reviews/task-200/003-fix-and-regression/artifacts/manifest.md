# Task 200 fix/regression artifacts

- Packet: `reviews/task-200/003-fix-and-regression/`
- Head SHA: `fdcfbcae8`
- Code checkpoint: `9de8b4fa2` (pushed); packet commits: `c1665196f`, `fdcfbcae8`
- Regression source: `../001-reproduction/artifacts/run-latency-rerun/counters-{off,on}-100k/physical-production-latency.memory-series.log`
- Validation source: `../001-reproduction/artifacts/run-latency-rerun/reuse-suite-manifest.json`
- Decision: no production fix; benchmark-only coverage growth is outside the production latency path.
