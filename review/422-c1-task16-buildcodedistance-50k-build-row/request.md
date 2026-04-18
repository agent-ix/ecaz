# Review Request: C1 Task16 BuildCodeDistance 50k Build Row

Current head at execution: `6a18dfc`

Comparison head for the pre-`418` side: `a4ccba9` (`4f94ad3^`)

## Context

Task 16's opening move is the deferred build-time measurement called out in the
task-15 final review on packet `421`:

> record one `50k` before/after row for the landed
> `BuildCodeDistance::new(...)` change from packet `418`

That row was still missing on `main`.

There is one important measurement wrinkle on current head:

- the canonical real-corpus loader builds tqhnsw indexes with
  `build_source_column = 'source'`
- that source-backed path uses `build_hnsw_graph_from_source(...)`
- it does **not** exercise `BuildCodeDistance::new(...)`

So measuring the normal real-corpus loader lane would not answer the actual
review question. The row below therefore uses a stripped scalar-only bench table
derived from the already-loaded `tqhnsw_real_50k_corpus` data:

- table: `tqhnsw_build418_50k_corpus`
- columns: `(id, embedding)` only
- no `source` column
- index build reloptions:
  - `m = 16`
  - `ef_construction = 128`
  - `storage_format = 'turboquant'`

That forces the scalar code-distance HNSW build path and directly measures the
`BuildCodeDistance::new(...)` change.

## Environment

Shared lane for both sides:

- scratch pg17 cluster at `/tmp/tqvector_pgrx_home`
- corpus source table: `tqhnsw_real_50k_corpus`
- bench table: `tqhnsw_build418_50k_corpus`
- relation shape: 50,000 rows, `1536` dims, `4` bits
- index DDL:

```sql
CREATE INDEX tqhnsw_build418_50k_m16_idx
ON tqhnsw_build418_50k_corpus
USING tqhnsw (embedding tqvector_ip_ops)
WITH (
  m = 16,
  ef_construction = 128,
  storage_format = 'turboquant'
);
```

Preparation once on current head:

1. loaded canonical `tqhnsw_real_50k_corpus` / `tqhnsw_real_50k_queries` into
   scratch
2. materialized `tqhnsw_build418_50k_corpus` from
   `SELECT id, embedding FROM tqhnsw_real_50k_corpus ORDER BY id`

Per-side measurement method:

1. install target head into the same scratch lane
2. restart scratch postmaster so the backend loads that head's `tqvector.so`
3. `DROP INDEX IF EXISTS tqhnsw_build418_50k_m16_idx`
4. record `clock_timestamp()` before and after the `CREATE INDEX`

Captured artifacts:

- `tmp/task16-418-before-a4ccba9.log`
- `tmp/task16-418-after-6a18dfc.log`

## Results

| head | meaning | elapsed |
|---|---|---:|
| `a4ccba9` | pre-`418` (`4f94ad3^`) | `984.63555s` |
| `6a18dfc` | current head with landed `418` change | `1049.14350s` |

Delta:

- `+64.50795s`
- `+6.55%`

## Readout

### 1. The `418` cost is real on the scalar no-source `50k` build lane

The landed `BuildCodeDistance::new(dimensions, bits, seed, tuples)` shape adds
an upfront O(N) max-self-score pass over the build tuples, and on this
`50k, m=16` scalar lane it is measurable:

- before: `16m24.636s`
- after: `17m29.144s`

### 2. This row is intentionally **not** the source-backed loader lane

That is by design, not a mismatch:

- the task-15 review question was specifically about the hot build-path change
  in `BuildCodeDistance::new(...)`
- the source-backed `build_source_column` lane bypasses that helper entirely

So the stripped bench table is the honest measurement surface for this note.

### 3. The result is worth carrying into task 16, but it is not a blocker

The regression is not small, but it also lives on a narrower lane than the
source-backed explicit-format runtime story that task 15 landed. The right
follow-on is:

- keep this row with the task-16 packet set
- do **not** silently attribute current 50k source-backed runtime behavior to
  the `418` change, because that lane does not use it

