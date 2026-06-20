# Artifact Manifest

Packet: `reviews/task-111h/030-counter-fixture-closeout-audit`

Task bucket: `reviews/task-111h`

Head SHA before packet commit: `230348e7c790c6a6ebe88682c1a55ee802e7f0aa`

Created: `2026-06-20`

## Scope

This is a read-only audit packet for Task 111h checklist rows:

- `plan/tasks/111h-ivf-persisted-rerank-format-sweep.md:205`
- `plan/tasks/111h-ivf-persisted-rerank-format-sweep.md:207`
- `plan/tasks/111h-ivf-persisted-rerank-format-sweep.md:209`

No benchmark suite was run and no new test log was produced.

## Commands

The audit was assembled from source and packet inspection using:

```sh
git rev-parse HEAD
git status --short --branch
git log --oneline -8
ls reviews/task-111h
rg -n "snapshot|MVCC|visible|visibility|xmin|xmax|delete|vacuum|partial_final|mixed_fallback|ec_ivf_index_placement" src/tests/ec_ivf.rs reviews/task-111h plan/tasks/111h-ivf-persisted-rerank-format-sweep.md
nl -ba plan/tasks/111h-ivf-persisted-rerank-format-sweep.md | sed -n '190,225p'
nl -ba src/am/common/explain.rs | sed -n '455,690p'
nl -ba src/am/ec_ivf/scan.rs | sed -n '2550,2755p'
nl -ba src/am/ec_ivf/scan.rs | sed -n '3868,3985p'
nl -ba src/tests/ec_ivf.rs | sed -n '1228,1295p'
nl -ba src/tests/ec_ivf.rs | sed -n '1416,1535p'
nl -ba src/tests/ec_ivf.rs | sed -n '1530,1595p'
nl -ba src/tests/ec_ivf.rs | sed -n '1588,1815p'
nl -ba src/tests/ec_ivf.rs | sed -n '2218,2305p'
```

The resulting audit is recorded in
`artifacts/counter-fixture-closeout-audit.md`.

## Key Result Lines

- EXPLAIN exposes Task 111h rerank placement/format, decode time, score time,
  source bytes, group pages, metadata bytes, payload bytes, and slab-copy bytes.
- The PG18 debug counter snapshot exposes the same relevant counters for
  fixture assertions.
- Existing PG18 fixtures cover create/build, insert, delete/vacuum, mixed
  fallback, partial final groups, persisted payload bytes, and no source-vector
  reads during compact index-side rerank.
- Update-path payload refresh and explicit MVCC/snapshot-visible rerank payload
  semantics remain unproven by the audited fixtures.
- Batched compact scoring still copies survivor payload bytes into
  `payload_slab`; copy-cost cleanup remains open.
