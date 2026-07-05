# Review Request: IVF Bound-Prune Limit Guard

- task: 51
- packet: `reviews/task-51/004-ivf-bound-prune-limit-guard`
- code commit: `aea43e0ad` (`Guard IVF bound pruning by rerank width`)
- scope: local `ec_ivf` RaBitQ scan pruning

## Summary

This removes the hard-coded `DEFAULT_PRE_PRUNE_K = 200` from the RaBitQ IVF
posting scan. Bound pruning now only materializes a running top-K cutoff when
the scan has a positive heap-f32 rerank frontier width.

That keeps no-rerank RaBitQ scans from applying an arbitrary candidate bound
without access to the SQL `LIMIT`, while preserving bound pruning for the
heap-f32 path where `rerank_width` is the downstream frontier size.

## Files Changed

- `src/am/ec_ivf/scan.rs`
  - uses `pre_rerank_candidate_limit(index_options)` to seed `running_top`.
  - removes the default 200 no-rerank pruning cutoff.
  - adds focused coverage for `pre_rerank_candidate_limit` semantics.

## Local Validation

All validation was local-only. No AWS, vchord, or pgvectorscale runs were used.

- `cargo check --lib --no-default-features --features pg18`: passed
- `cargo test --lib pre_rerank_candidate_limit_requires_heap_f32_positive_width --no-run --no-default-features --features pg18`: passed
- `rustfmt --check src/am/ec_ivf/scan.rs`: passed
- `git diff --check -- src/am/ec_ivf/scan.rs`: passed
- `cargo pgrx install --test --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`: passed
- isolated PG18 smoke: passed with `ec_ivf` RaBitQ only

Key smoke counters:

- no-rerank RaBitQ, `LIMIT 220`:
  - `Postings Pruned By Bound: 0`
  - `Candidates Emitted: 220`
  - `Rerank Rows: 0`
  - `no_rerank_limit_220_count = 220`
- heap_f32 RaBitQ, `rerank_width = 3`, `LIMIT 3`:
  - `Postings Pruned By Bound: 252`
  - `Candidates Emitted: 3`
  - `Rerank Rows: 3`
  - `Heap Blocks Fetched: 1`
  - `heap_f32_limit_3_count = 3`

## Artifacts

See `artifacts/manifest.md` for commands, timestamps, and artifact metadata.
