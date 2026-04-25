# PG18 Contributor Obsolete Output Retirement

## Summary

This packet covers commit `009d95b93d8d4627626677686cda0977c8b6f528`.

Non-elected PG18 contributor workers now retire hidden local-only output when
the hidden row's next heap TID has already been emitted through the shared
coordinator or another worker snapshot. That keeps the contributor loop from
sleeping on an obsolete duplicate row until the diagnostic drain-poll budget
expires.

This is a narrow progress fix. It does not solve rank-aware distinct worker
contribution; the live PG18 diagnostic lane still reports zero useful foreign
handoffs.

## Result

The default elected-emitter lane still passes:

```text
Limit (actual time=13.752..14.817 rows=16.00 loops=1)
Bootstrap Expansions: 17
Elements Scored: 17
Heap TIDs Returned: 16
Parallel Handoffs: Foreign Selected: 0
Parallel Handoffs: Foreign Head: 0
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

The contributor diagnostic lane also still preserves serial equivalence, but
the live fixture remains counter-neutral:

```text
Limit (actual time=36.165..36.856 rows=16.00 loops=1)
Bootstrap Expansions: 17
Elements Scored: 17
Heap TIDs Returned: 16
Parallel Handoffs: Foreign Selected: 0
Parallel Handoffs: Foreign Head: 0
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

Interpretation: the obsolete-hidden-output retirement path is covered by a
unit test and safe in the live PG18 fixture, but useful performance still
requires rank-aware distinct contribution behind the elected visible emitter.

## Artifacts

- `artifacts/pg18-parallel-contributor-retire-default.log`
- `artifacts/pg18-parallel-contributor-retire-diagnostic.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
- `cargo test retire_obsolete_non_emitting_parallel_contributor_output --lib`
- `cargo test contributor --lib`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --features pg_test --no-default-features`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-contributor-retire-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-contributor-retire-diagnostic.log`
- `git diff --check`
