# Review Request: C1 ADR-030 V2 Task-15 Landing Proof

## Context

Task 15 asks for one concrete landing claim before this branch line can merge to
`main`:

1. `CREATE INDEX ... WITH (storage_format='turboquant')` and
   `...WITH (storage_format='pq_fastscan')` both pass the `50k` real-corpus
   recall harness
2. insert + vacuum round-trip is proven on both formats
3. the old build gate and grouped-v2 unsupported rejects are gone from runtime
   code

Packets `378` through `404` did the implementation work in narrow slices. This
packet is the explicit proof rollup at the current head.

Current head for this proof packet:

- `215db8ce27f989bfa45e4fea1a0983d6893d0e5c`

## Problem

Without one rollup packet, the branch had all the ingredients but not one
reviewable statement that said:

- both first-class formats clear the canonical `50k` harness
- both first-class formats already have reloption-driven insert/vacuum
  round-trip proof
- the experimental build gate / unsupported-format rejects are no longer in the
  runtime code tree

That is a proof packaging gap, not an implementation gap.

## Scope

No code change.

This packet only gathers the already-landed proof surfaces at `HEAD`.

## Proof

### 1. Explicit `turboquant` passes the canonical `50k` gate

Live-cluster run on the current branch head:

- command:
  - `TQV_PG_SOCKET_DIR=/home/peter/.pgrx ./scripts/run_real_corpus_recall_scratch.sh gate --prefix tqhnsw_real_50k --storage-format turboquant --queries-table tqhnsw_real_50k_queries`
- artifact:
  - `tmp/real_corpus_runs/20260417T004126Z_gate_tqhnsw_real_50k_turboquant_tqhnsw_real_50k_queries.tsv`

Results:

| m | ef_search | Recall@10 | gate | passes |
|---|-----------|-----------|------|--------|
| 8 | 40  | 0.8301 | —    | true |
| 8 | 128 | 0.8927 | 0.89 | true |
| 8 | 200 | 0.9011 | —    | true |
| 16 | 200 | 0.9376 | —   | true |

### 2. Explicit `pq_fastscan` passes the canonical `50k` gate

Live-cluster run on the current branch head after packet `404` aligned the
default source-backed rerank lane:

- runtime settings:
  - `grouped_scan_window = 64`
  - `grouped_scan_score_mode = binary`
  - `grouped_scan_rerank_mode = heap_f32`
  - `grouped_scan_rerank_source_column = build_source_column`
- command:
  - `TQV_PG_SOCKET_DIR=/home/peter/.pgrx ./scripts/run_real_corpus_recall_scratch.sh gate --prefix tqhnsw_real_50k --storage-format pq_fastscan --queries-table tqhnsw_real_50k_queries`
- artifact:
  - `tmp/real_corpus_runs/20260417T002339Z_gate_tqhnsw_real_50k_pq_fastscan_tqhnsw_real_50k_queries.tsv`

Results:

| m | ef_search | Recall@10 | gate | passes |
|---|-----------|-----------|------|--------|
| 8 | 40  | 0.8231 | —    | true |
| 8 | 128 | 0.9078 | 0.89 | true |
| 8 | 200 | 0.9174 | —    | true |
| 16 | 200 | 0.9671 | —   | true |

### 3. Insert + vacuum round-trip is already proven on both formats

Packet `393` added the reloption-driven round-trip proof in `src/lib.rs`:

- packet:
  - [review/393-c1-adr030-v2-storage-format-round-trip-proof/request.md](/home/peter/dev/tqvector/review/393-c1-adr030-v2-storage-format-round-trip-proof/request.md)
- tests:
  - `test_tqhnsw_turboquant_reloption_round_trip`
  - `test_tqhnsw_pq_fastscan_reloption_round_trip`

That proof surface covers, on both formats selected through the reloption:

1. build
2. live insert
3. ordered scan
4. delete
5. vacuum
6. ordered scan again with the deleted row absent

### 4. Runtime code no longer contains the old build gate or unsupported rejects

I checked the live source tree at this head with:

```bash
rg -n "TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD|ADR030_GROUPED_V2_.*UNSUPPORTED|GROUPED_V2_.*UNSUPPORTED" src scripts
```

That returns no matches.

So, at this head:

- no `TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD` references remain in runtime code
- no `ADR030_GROUPED_V2_*_UNSUPPORTED` runtime rejects remain in runtime code

## Validation

This packet adds no code, so it relies on the most recent validated code
checkpoint: packet `404`.

Packet `404` validation:

- `cargo check --tests`
- `cargo check --tests --no-default-features --features 'pg17 pg_test'`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- `bash -n scripts/restart_adr030_scratch.sh`

Required full-test commands still fail on this workstation at the same known
PostgreSQL linker boundary:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed unresolved PostgreSQL symbols remain in the same family:

- `CurrentMemoryContext`
- `PG_exception_stack`
- `error_context_stack`
- `CopyErrorData`
- `errstart`

## Outcome

At `215db8ce27f989bfa45e4fea1a0983d6893d0e5c`, task 15's technical landing bar
is satisfied in the runtime code and live `50k` harness evidence:

1. explicit `turboquant` passes the canonical `50k` gate
2. explicit `pq_fastscan` passes the canonical `50k` gate
3. both formats already have reloption-driven insert/vacuum round-trip proof
4. the old experimental build gate and grouped-v2 unsupported rejects are gone
   from runtime code

The remaining work is no longer a format-parity implementation problem. It is
merge/readiness work:

- reviewer signoff on the final proof
- any desired doc/task cleanup outside `src/` / `scripts/`
- branch-to-`main` integration mechanics

## Next Slice

Unless reviewer feedback uncovers a real defect, the next slice should be the
merge-preparation pass rather than more `pq_fastscan` implementation work.
