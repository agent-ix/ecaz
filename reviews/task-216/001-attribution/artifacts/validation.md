# Task 216 attribution validation

- Suite audit before the run passed: 1 step.
- `ecaz bench suite run` completed the physical step with exit code 0 and
  wrote `run/results.jsonl` and `run/suite-manifest.json`.
- Physical topology gate: passed, three owners, 100,000 source rows, two
  remote owners verified, no non-owned or orphan records.
- Physical serving and traversal reconciliation drills: passed.
- Suite-level NFR-021 assertion: unavailable for this single-scale diagnostic;
  the limitation and its effect on decision eligibility are recorded in
  `attribution-disposition.md`.
- Diagnostic install command:
  `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18,distann-head-attribution-benchmark`
- Normal restore command:
  `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18`
- Normal restored schema: `/home/peter/.pgrx/18.3/pgrx-install/share/postgresql/extension/ecaz--0.1.1.sql`, 348,909 bytes; no attribution-only
  `debug_expand_search` entry found.
- The transient cluster at
  `/home/peter/.ecaz/clusters/task216-attribution-100k` was stopped and
  removed after the cited artifacts were captured.
