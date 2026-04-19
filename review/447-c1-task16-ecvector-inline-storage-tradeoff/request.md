# Review Request: C1 Task16 Ecvector Inline-Storage Tradeoff Measurement

Current head at execution: `7fa40d3`

## Context

Packet `446` answered the speed question on the real canonical row model:

- default-storage `ecvector` serious lane stayed in the `5.2ms-5.9ms` range
- inline-storage `ecvector` carried the packet-`441` win onto the real
  product surface

That left one explicit landing-checklist item open in the task plan:

- quantify the **tradeoff** of forcing the canonical raw vector inline

This packet closes that item by measuring:

1. buffer-cache pressure
2. vacuum scan cost
3. WAL / HOT behavior on small row updates
4. fresh TurboQuant build time

The point is not "inline good" or "inline bad". The point is to say exactly
what the `ALTER COLUMN embedding SET STORAGE PLAIN` lever buys and what it
costs on the real `ecvector` surface.

## Measurement surfaces

Reused the same current-head scratch DB and tables from packet `446`:

- database: `task16_ecvector`
- default surface: `tqhnsw_real_50k_ecvector_default_corpus`
- inline surface: `tqhnsw_real_50k_ecvector_inline_corpus`

Cluster settings read back from scratch:

```sql
SHOW shared_buffers;
SHOW block_size;
```

Result:

- `shared_buffers = 128MB`
- `block_size = 8192`
- shared-buffers capacity in pages: `16,384`

## 1. Buffer-cache pressure

Read back heap and TOAST relation sizes:

```sql
SELECT
  c.relname,
  c.relpages AS heap_pages,
  pg_relation_size(c.oid) AS heap_bytes,
  t.relname AS toast_relname,
  t.relpages AS toast_pages,
  pg_relation_size(t.oid) AS toast_bytes
FROM pg_class c
LEFT JOIN pg_class t ON t.oid = c.reltoastrelid
WHERE c.relname IN (
  'tqhnsw_real_50k_ecvector_default_corpus',
  'tqhnsw_real_50k_ecvector_inline_corpus'
)
ORDER BY c.relname;
```

And:

```sql
SELECT
  'tqhnsw_real_50k_ecvector_default_corpus' AS relname,
  avg(pg_column_size(embedding))::numeric(10,1) AS avg_embedding_bytes
FROM tqhnsw_real_50k_ecvector_default_corpus
UNION ALL
SELECT
  'tqhnsw_real_50k_ecvector_inline_corpus' AS relname,
  avg(pg_column_size(embedding))::numeric(10,1) AS avg_embedding_bytes
FROM tqhnsw_real_50k_ecvector_inline_corpus;
```

Results:

| Surface | `attstorage` | Avg `pg_column_size(embedding)` | Heap pages | Heap bytes | TOAST pages | TOAST bytes |
|---------|---------------|---------------------------------|------------|------------|-------------|-------------|
| default `ecvector` | `e` | `6144.0` | `468` | `3,833,856` | `100,000` | `819,200,000` |
| inline `ecvector` | `p` | `6148.0` | `50,000` | `409,600,000` | `50,000` | `409,600,000` |

Readout:

- raw vector bytes are effectively the same either way: `6144` vs `6148`
- the important difference is **where** those bytes live
- total heap+toast relation bytes stay in nearly the same class:
  - default: `823.0MB`
  - inline: `819.2MB`
- but the hot heap footprint changes radically:
  - default heap: `468` pages = `2.86%` of `shared_buffers`
  - inline heap: `50,000` pages = `305.18%` of `shared_buffers`

So the inline win is not "use much less storage". It is "move the raw vector
from a mostly-TOASTed layout into the heap". That gives the rerank hot path
what it wants, but the heap working set grows from "fits trivially in shared
buffers" to "about 3.05x shared_buffers".

## 2. Vacuum scan cost

Timed one `VACUUM (ANALYZE)` on each already-built corpus table.

Commands:

```sql
VACUUM (ANALYZE) tqhnsw_real_50k_ecvector_default_corpus;
VACUUM (ANALYZE) tqhnsw_real_50k_ecvector_inline_corpus;
```

Measured wall time externally with `date +%s%N` around the approved psql
wrapper.

Results:

| Surface | Wall time | Heap+TOAST pages | ms / total page |
|---------|-----------|------------------|-----------------|
| default `ecvector` | `19.121s` | `100,468` | `0.190ms` |
| inline `ecvector` | `19.250s` | `100,000` | `0.193ms` |

Readout:

- vacuum time was essentially flat (`+0.129s / +0.67%` inline)
- the useful normalization is **total pages**, not heap pages alone
- once normalized by total heap+TOAST pages, the two surfaces are nearly
  identical

So inline storage does **not** show up as a large vacuum-scan penalty on this
static 50k seam. The storage bytes mostly moved between heap and TOAST rather
than disappearing or exploding.

## 3. WAL and HOT behavior on small row updates

Created two update-probe tables so the corpus/ANN tables themselves stayed
untouched:

```sql
CREATE TABLE tqhnsw_real_50k_ecvector_default_update_probe AS
SELECT id, 0::integer AS touch, source, embedding
FROM tqhnsw_real_50k_ecvector_default_corpus;

CREATE TABLE tqhnsw_real_50k_ecvector_inline_update_probe (
  id bigint,
  touch integer,
  source real[],
  embedding ecvector
);
ALTER TABLE tqhnsw_real_50k_ecvector_inline_update_probe
  ALTER COLUMN embedding SET STORAGE PLAIN;
INSERT INTO tqhnsw_real_50k_ecvector_inline_update_probe
SELECT id, 0::integer, source, embedding
FROM tqhnsw_real_50k_ecvector_inline_corpus;
```

Then updated a tiny non-indexed integer field only:

```sql
UPDATE ... SET touch = touch + 1 WHERE id > 1000 AND id <= 2000;
```

WAL bytes were measured by sampling `pg_current_wal_insert_lsn()` before and
after and applying `pg_wal_lsn_diff(...)`.

Why quote the **second** 1k-row batch: the first batch showed the same
direction but was noisier. The second non-overlapping batch is the steadier
comparison cell.

Second-batch results:

| Surface | Rows updated | Wall time | WAL bytes | WAL / row |
|---------|--------------|-----------|-----------|-----------|
| default `ecvector` | `1000` | `9.177s` | `4,013,896` | `4,013.9` |
| inline `ecvector` | `1000` | `9.950s` | `14,297,248` | `14,297.2` |

Delta:

- inline WAL: `+10,283,352` bytes
- inline WAL multiplier: `3.56x`
- inline wall-time delta: `+0.773s / +8.42%`

HOT/readback stats after the two 1k-row batches:

```sql
SELECT relname, n_tup_upd, n_tup_hot_upd, n_dead_tup
FROM pg_stat_user_tables
WHERE relname IN (
  'tqhnsw_real_50k_ecvector_default_update_probe',
  'tqhnsw_real_50k_ecvector_inline_update_probe'
)
ORDER BY relname;
```

Results:

| Probe | `n_tup_upd` | `n_tup_hot_upd` | `n_dead_tup` |
|-------|-------------|-----------------|--------------|
| default | `2001` | `38` | `1000` |
| inline | `2001` | `0` | `2006` |

Readout:

- inline `ecvector` makes tiny row touches materially heavier
- the row-churn cost is not subtle: WAL moved from `4.0MB` to `14.3MB`
  on the steady 1k-row batch
- inline also lost HOT entirely on this seam (`0` vs `38`)

This is the clearest downside of the inline lever: it speeds the serious read
path by keeping the raw vector in the heap tuple, but that same choice makes
row-version churn materially more expensive when a row gets rewritten for any
other reason.

## 4. Fresh TurboQuant build time

Created two clean build-probe tables:

```sql
CREATE TABLE tqhnsw_real_50k_ecvector_default_buildprobe_corpus AS
SELECT id, source, embedding
FROM tqhnsw_real_50k_ecvector_default_corpus;

CREATE TABLE tqhnsw_real_50k_ecvector_inline_buildprobe_corpus (
  id bigint,
  source real[],
  embedding ecvector
);
ALTER TABLE tqhnsw_real_50k_ecvector_inline_buildprobe_corpus
  ALTER COLUMN embedding SET STORAGE PLAIN;
INSERT INTO tqhnsw_real_50k_ecvector_inline_buildprobe_corpus
SELECT id, source, embedding
FROM tqhnsw_real_50k_ecvector_inline_corpus;
```

Built the same TurboQuant index on both:

```sql
CREATE INDEX ... USING tqhnsw (embedding ecvector_ip_ops)
WITH (m = 16, ef_construction = 128, storage_format = 'turboquant');
```

Timed with `date +%s%N` around the approved psql wrapper.

Results:

| Surface | Build time | Index bytes |
|---------|------------|-------------|
| default `ecvector` | `180.774s` | `68,280,320` |
| inline `ecvector` | `173.784s` | `68,280,320` |

Delta:

- inline build: `-6.990s / -3.87%`

Readout:

- inline did **not** penalize build time on this seam
- if anything, the default-storage surface paid a modest extra build cost
  from reading / decoding the mostly-external raw source
- index output size stayed identical

## Overall readout

Inline `ecvector` is a real tradeoff, but not the tradeoff I would have guessed
before measuring it:

- **Serious-lane read latency:** much better
  - packet `446`: `5.248ms -> 3.195ms` for TurboQuant on the confirming q200
    rerun (`-39.12%`)
- **Total storage footprint:** almost unchanged
  - bytes mostly move between heap and TOAST rather than disappear
- **Buffer-cache profile:** much heavier on the heap
  - heap working set grows from `468` pages to `50,000` pages
- **Vacuum scan cost:** roughly flat
  - total pages scanned stay about the same
- **Fresh TurboQuant build time:** slightly better inline
  - `-3.87%`
- **Small row rewrites:** materially worse inline
  - steady 1k-row update batch: `3.56x` WAL, zero HOT updates

So the packet-`441` / packet-`446` inline-storage win is not a "free default".
It is a serious-lane optimization with a real write-path penalty. The clean
product question after this packet is not "does inline help?" — yes, clearly.
The question is whether the product surface should:

1. default to external storage and expose inline as an expert lever for
   read-mostly workloads, or
2. choose inline by default because serious-lane latency matters more than
   row-churn cost for the intended deployment class.

This packet does not make that policy choice. It closes the measurement gap
so the choice can be made with numbers instead of guesswork.

## Practical mitigation guidance

The primary mitigation is structural:

- keep the embedding row as static as possible
- move frequently updated metadata to a separate table
- join when needed

That preserves the inline serious-lane read win without turning unrelated
metadata writes into large row rewrites.

Operationally, the measured guidance now looks like:

- **Read-mostly / append-mostly tables:** inline `ecvector` is justified
  when serious-lane latency matters
- **Churn-heavy rows:** keep `ecvector` mostly external

Lower-level mitigations like `fillfactor` may help at the margin, but they are
not the primary answer. The clean product surface is likely a per-column
storage-policy choice rather than one universal default.
