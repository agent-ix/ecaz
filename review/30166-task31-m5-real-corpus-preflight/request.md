# Task 31 M5 Real Corpus Preflight

Reviewer: please review this preflight packet before the first real M5 IVF
baseline packet.

## Scope

This packet executes the immediate next action from packet `30165`: verify the
local PG18/extension target and inventory loaded corpora on the M5. It does not
run recall, latency, storage, build-time, memory, or EXPLAIN/counter baselines.

All command outputs are under `artifacts/`, and `artifacts/manifest.md` records
the packet-local metadata and cited result lines.

## Result

PG18 and the installed extension are reachable through the documented `ecaz`
operator path:

- PostgreSQL: `18.3 (Homebrew)` on `aarch64-apple-darwin25.2.0`
- Extension: `ecaz 0.1.1`
- CLI path: `/Users/peter/.cargo/bin/ecaz`
- Database target: `postgres`, socket directory `/Users/peter/.pgrx`, port
  `28818`

The corpus inventory contains only the prior synthetic Task 31 smoke prefix:

| prefix | rows | queries | indexes | profiles |
|---|---:|---|---|---|
| `task31_m5_smoke_pqg8` | 10000 | yes | `btree, ec_ivf` | `ec_ivf` |

No real DBPedia Task 31 10k, 25k, 100k, or 990k prefixes are currently loaded
in `postgres`.

## Interpretation

The M5 operator loop is ready, but Phase A from packet `30165` cannot start
with measurement yet. The next checkpoint must stage and load real corpus
surfaces before running recall or latency.

Because only the synthetic smoke corpus is loaded, do not treat any next recall
or latency run as a Task 31 real baseline unless it first creates or verifies a
real DBPedia-derived prefix with the intended one-index-per-table shape.

## Commands

PG18 / extension status:

```sh
/Users/peter/.cargo/bin/ecaz dev sql --pg 18 --db postgres \
  --socket-dir /Users/peter/.pgrx --port 28818 --raw \
  --sql "select version(); select extname, extversion from pg_extension where extname = 'ecaz';" \
  --log-output review/30166-task31-m5-real-corpus-preflight/artifacts/pg18-ecaz-status.log
```

Corpus inventory:

```sh
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 \
  --log-file review/30166-task31-m5-real-corpus-preflight/artifacts/corpus-list.log \
  corpus list
```

## Next Checkpoint

Create `30167-task31-m5-real-corpus-staging` before the 10k baseline packet.
That checkpoint should either:

- fetch and prepare the canonical DBPedia OpenAI 1536D parquet release through
  `ecaz corpus fetch` and `ecaz corpus prepare`, then derive/load 10k, 25k, and
  100k one-index-per-table prefixes; or
- if staged TSV files already exist outside the database, record their paths,
  hashes, row counts, dimensions, and load commands.

Use explicit IVF reloptions from packet `30165`:

- 10k/25k: `profile=ec_ivf`, `storage_format=pq_fastscan`,
  `pq_group_size=8`, `nlists=64`, `nprobe=48`, `rerank=heap_f32`,
  `rerank_width=750`
- 100k: `profile=ec_ivf`, `storage_format=pq_fastscan`,
  `pq_group_size=8`, `nlists=128`, `nprobe=48`, `rerank=heap_f32`,
  `rerank_width=500`

Do not run long 990k setup in the next checkpoint unless there is already a
local staged corpus and a specific 990k hypothesis to test.

## Validation

No tests or benchmarks were run. This is an inventory-only preflight packet.

## Artifacts

- `artifacts/pg18-ecaz-status.log`
- `artifacts/corpus-list.log`
- `artifacts/manifest.md`
