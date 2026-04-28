# Task 28 IVF A9 Remaining Local Inventory

## Scope

This packet records the local A9 inventory after the current-head 100k IVF refresh and A10 recommendation packets.

The goal is to make the remaining A9 work explicit before starting any long HNSW or 990k/1M build.

## Result

Local PG18 currently has:

| surface | rows | index state |
|---|---:|---|
| `task28_a9_100k_ivf_corpus` | 100000 | IVF selected index exists: `task28_a9_100k_ivf_idx`, `19 MB` |
| `task28_a9_100k_hnsw_corpus` | 100000 | corpus and queries exist; no HNSW index exists |
| `ec_hnsw_real_ann_benchmarks_anchor_corpus` | 990000 | HNSW m16/w8 index exists: `1289 MB` |

The 990k HNSW index is the Task 26 anchor from packet 669. There is no matching 990k/1M IVF selected-point index in the local inventory.

## Interpretation

A9's current-head IVF 100k selected point is now well recorded in packet 30126. The remaining literal A9 work is not a quick measurement rerun:

- Build or restore a 100k HNSW index on `task28_a9_100k_hnsw_corpus`, then run the matched recall/latency/memory matrix.
- Build a 990k/1M IVF selected-point surface on the existing anchor table, then run recall/latency/memory/size.
- Decide whether the existing packet 669 HNSW 990k build artifact is sufficient comparison evidence or whether a fresh current-head HNSW scan packet is required.

Given the user's instruction not to let HNSW comparison consume IVF momentum, this packet stops short of starting a long HNSW build automatically.

## Validation

- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30128-task28-ivf-a9-remaining-inventory/artifacts/a9_local_inventory.sql --raw --log-output review/30128-task28-ivf-a9-remaining-inventory/artifacts/a9_local_inventory.log`

## Artifacts

- `artifacts/a9_local_inventory.sql`
- `artifacts/a9_local_inventory.log`
- `artifacts/manifest.md`
