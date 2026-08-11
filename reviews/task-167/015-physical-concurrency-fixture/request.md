# Task 167 checkpoint: physical concurrent insert/query fixture

The physical multinode fixture now runs a concurrent insert/query drill against
the published `dm_idx` generation. Four scan workers repeatedly execute a
physical top-k query while a background worker performs 12 coordinator-routed
inserts using the physical source-row shape (`id`, `source_id`, `source`, and
`embedding`). Each scan must complete and return the expected top-k cardinality;
the fixture fails if any worker errors or returns a wrong count.

Validation:

- `cargo check -p ecaz-cli` — passed at `d38abfa44` (pre-existing unused-field
  warning in `commands/corpus/load.rs` only).
- `cargo check --no-default-features --features pg18` — passed at
  `d38abfa44`.
- `git diff --check` — passed before commit.

This is not a closeout request. The live PG18 fixture has not run on this host;
FR-083-AC-4 fresh-rebuild parity, insert-throughput A/B, and the required
10k/50k/100k suite artifacts remain outstanding. No runtime result is claimed.
