# Task 167 checkpoint: physical mid-insert fixture hook

The multinode physical-generation fixture now invokes the adapted one-owner
mid-insert fault drill before its physical benchmark phase. The drill enables
`ec_distann.debug_fail_insert`, attempts a physical insert, and checks that the
source-row count and physical record count remain unchanged after the injected
failure. A failed drill aborts the fixture and is included in its summary.

Validation:

- `cargo check -p ecaz-cli` — passed at `ff9b9ae8e` (pre-existing unused-field
  warning in `commands/corpus/load.rs` only).
- `cargo check --no-default-features --features pg18` — passed at
  `ff9b9ae8e`.
- `git diff --check` — passed before commit.

This is not a closeout request. The live PG18 multinode drill, concurrent
insert/query coverage, FR-083-AC-4 parity, insert-throughput A/B, and required
10k/50k/100k recall/latency/storage suite artifacts remain outstanding. This
host has no installed `ecaz` binary or staged corpus, so no runtime result is
claimed here.
