# Task 215 production-wiring validation

At code checkpoint `ea51a9c8b`:

- PG18 compile check passed with the installed PG18 `pg_config`.
- Normal PG18 release extension installation passed.
- No `distann-head-attribution-benchmark` feature was enabled.
- `git diff --check` passed for the checkpoint.
- The default GUC test was updated to assert `beam_width=64` and
  `hop_rounds=8`; `candidate_heap_limit` remains 32.

The first check attempt used the host's stale PG17 pgrx config and failed
before compilation because `/home/peter/.pgrx/17.9/.../pg_config` is absent.
The successful rerun selected `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`
explicitly. This is environment provenance, not a source failure.
