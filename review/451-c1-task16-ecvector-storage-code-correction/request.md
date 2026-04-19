# Review Request: C1 Task16 Ecvector Storage-Code Correction

Current head at execution: `bb362c0`

## Context

ADR-044 and the task-16 plan were carrying one important factual mistake:

- they treated packet `446` / packet `447`'s default `ecvector` surface as
  `EXTENDED`
- they therefore listed an `EXTERNAL` cell as the highest-value remaining
  must-measure item

While preparing that `EXTERNAL` run, I verified the storage codes on the live
pg17 scratch cluster and found the mapping was the opposite of what the docs
were assuming.

## What was verified

Two checks mattered.

### 1. Current `ecvector` type and table state

Scratch query:

```sql
select t.typname, t.typstorage
from pg_type t
where t.typname in ('ecvector', 'tqvector')
order by t.typname;

select c.relname, a.attstorage
from pg_class c
join pg_attribute a on a.attrelid = c.oid
where c.relname in (
  'tqhnsw_real_50k_ecvector_default_corpus',
  'tqhnsw_real_50k_ecvector_inline_corpus',
  'tqhnsw_real_50k_ecvector_external_corpus'
)
and a.attname = 'embedding'
order by c.relname;
```

Result:

- `ecvector` typstorage = `e`
- default corpus `embedding` attstorage = `e`
- inline corpus `embedding` attstorage = `p`
- the attempted "external" corpus also came back `e`

That already suggested the "new EXTERNAL cell" I was setting up was just a
duplicate of the already-measured default surface.

### 2. Empirical storage-code mapping on pg17

Scratch probe:

```sql
CREATE TABLE tqstorage_probe (v text);
SELECT 'default', attstorage ...;
ALTER TABLE tqstorage_probe ALTER COLUMN v SET STORAGE EXTERNAL;
SELECT 'external', attstorage ...;
ALTER TABLE tqstorage_probe ALTER COLUMN v SET STORAGE EXTENDED;
SELECT 'extended', attstorage ...;
ALTER TABLE tqstorage_probe ALTER COLUMN v SET STORAGE MAIN;
SELECT 'main', attstorage ...;
ALTER TABLE tqstorage_probe ALTER COLUMN v SET STORAGE PLAIN;
SELECT 'plain', attstorage ...;
DROP TABLE tqstorage_probe;
```

Result:

- default text: `x`
- `SET STORAGE EXTERNAL`: `e`
- `SET STORAGE EXTENDED`: `x`
- `SET STORAGE MAIN`: `m`
- `SET STORAGE PLAIN`: `p`

So on the server we are actually measuring:

- `e` = `EXTERNAL`
- `x` = `EXTENDED`
- `m` = `MAIN`
- `p` = `PLAIN`

## What this slice changes

Updated:

- `spec/adr/ADR-043-native-ecvector-raw-f32-column-type.md`
- `spec/adr/ADR-044-ecvector-rerank-source-location-and-storage-policy.md`
- `plan/tasks/16-turboquant-iteration.md`

Key corrections:

1. packet `446` / packet `447` are now described correctly as measuring
   `EXTERNAL` (current-head default) vs `PLAIN`
2. the remaining must-measure heap-storage cell is now `EXTENDED`, not
   `EXTERNAL`
3. ADR-044's option catalog and decision rules now treat:
   - A1 = `EXTERNAL` (measured current default)
   - A2 = `EXTENDED` (unmeasured compressed-TOAST alternative)
   - A3 = `MAIN`
   - A4 = `PLAIN`
4. the open questions now ask the right thing:
   - how much latency tax does `EXTENDED` add over `EXTERNAL`?
   - does `EXTENDED` buy enough TOAST-footprint reduction to matter?

## Important measurement consequence

No trustworthy new q200 or WAL/HOT result is claimed here.

I stopped before recording a new packet because the attempted `EXTERNAL` setup
was not a distinct surface; it was duplicating the already-measured default
`ecvector` state. This slice corrects the matrix first so the next timed run is
the real missing cell.

## Validation

Docs / plan only.

The factual basis for the correction was verified against the live pg17 scratch
cluster with the approved args-only psql wrapper; no Rust or SQL code changed,
so the cargo / pgrx / clippy checkpoint trio was not rerun for this slice.

## Review focus

1. Does the packet make the storage-code correction unambiguous enough that the
   next measurement slice cannot accidentally re-run the already-measured
   default surface?
2. Is the corrected ADR-044 framing right:
   - current-head default = `EXTERNAL`
   - highest-value remaining heap-storage cell = `EXTENDED`
   - `PLAIN` and C1 remain the latency-focused alternatives
3. Is it reasonable to treat the aborted `EXTERNAL` setup as a no-result and
   move directly to the corrected `EXTENDED` and `MAIN` cells?
