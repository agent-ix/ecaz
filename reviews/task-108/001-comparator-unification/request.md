# Task 108 — Comparator bench unification (CLI + suite step)

Branch: `task-108-109-comparator-unification`
Head SHA at request: `6f3427d96`
Packet: `reviews/task-108/001-comparator-unification/`

## What changed

Replace the head-to-head `ecaz compare {pgvector,vectorscale}` surface with a
standalone, engine-generic competitor-measurement command, and retire the bash
comparator measurement scripts. This decouples comparator measurement from ecaz
re-measurement (the head-to-head path re-measured ecaz on every invocation,
violating the no-re-run policy).

### `ecaz bench comparator` (new)

`crates/ecaz-cli/src/commands/bench/comparator.rs`. Measures **one** external
engine standalone — no ecaz engine in the loop:

- `--engine {vchord | pgvector-hnsw | pgvector-ivfflat | pgvectorscale}`
  (lantern out of scope per the task decision).
- `--sweep` = the engine's query GUC values (vchord `vchordrq.probes`,
  pgvector-hnsw `hnsw.ef_search`, pgvector-ivfflat `ivfflat.probes`,
  pgvectorscale `diskann.query_search_list_size`).
- Per-engine build knobs: IVF-family `--lists` (default `ceil(sqrt(rows))`,
  pinnable), hnsw `--m`/`--ef-construction`, pgvectorscale `--num-neighbors` /
  `--build-search-list-size` / `--max-alpha` / `--storage-layout`;
  `--maintenance-work-mem` (default 4GB), `--rebuild`, `--log-output`.
- Flow per sweep value: ensure extension (vchord surfaces the
  `shared_preload_libraries` + restart prerequisite); build sidecar
  `{prefix}_corpus_<engine>` from `{prefix}_corpus.source` + the engine index
  idempotently (`vector_ip_ops`, `<#>`); brute-force ground truth via the
  shared `recall::brute_force_top_k`/`fetch_sources_public`; run the query set
  once capturing top-k ids + durations; emit recall@k (`recall_summary` path),
  latency percentiles (`latency::summarize`), and a `pg_relation_size` storage
  line. Output is the recall/latency/storage table shape the suite parser
  already reads.
- The sidecar/populate/index-build SQL builders were lifted out of the deleted
  `compare/{pgvector,vectorscale}.rs`; vchord builders are new.

### `comparator` suite step (new)

`suite.rs`: `Comparator(ComparatorStep)`, kebab kind `"comparator"`, modeled on
`RecallStep`/`LatencyStep`. `expand_comparator()` emits
`bench comparator --engine ... --sweep ...`; the `"comparator"` parse arm reuses
table + summary parsing (`comparator`, `comparator_build`,
`comparator_index_size` metrics).

### Removed

- `crates/ecaz-cli/src/commands/compare/` (whole dir) + the `Compare` CLI
  variant + the `ComparePgvector`/`CompareVectorscale` suite steps and their
  expand/parse helpers.
- `scripts/comparators/` bash measurement path: `sweep.sh`, `compute_recall.py`,
  `run_all.sh`, `_bench_lib.sh`, per-engine `load.sh`/`bench.sh`, and the entire
  `lantern/` directory.

### Kept (deliberate)

- `scripts/comparators/{pgvector,pgvectorscale,vchord}/install.sh` — extension
  install + preload + PG restart is an operator prerequisite the CLI can't do.
- `scripts/comparators/_common.sh` — **deviation from the plan's literal
  delete-list**: the kept `install.sh` scripts source it for
  `comparator_log` / `comparator_extension_installed` / default paths, so it
  stays. Its load-table helpers are now unused but harmless.
- Migrated the 3 committed compare-step configs to `comparator` steps.

## Commits

- `ea91ba9f8` replace head-to-head compare with standalone `ecaz bench comparator`
- `6f3427d96` retire bash comparator measurement scripts

## Verification

- `cargo build -p ecaz-cli`: clean.
- `cargo clippy -p ecaz-cli --all-targets`: no new warnings (the
  `large_size_difference` on `SuiteStep` pre-exists — largest variant is
  `SpirePipelineStep`, not the new `ComparatorStep`).
- `cargo test -p ecaz-cli comparator`: 15 passed (see
  `artifacts/comparator-tests.log`). Covers the per-engine SQL builders (incl.
  vchord `USING vchordrq (embedding vector_ip_ops)` / `residual_quantization =
  true` / `lists = [N]` / `$vco$`), KNN `<#>` + `$1::real[]::vector(dim)`,
  `expand_comparator` argv, `default_lists_for_rows` (50000→224, override),
  and the comparator table/summary parser round-trip.
- `expand_comparator` argv is asserted by
  `suite::tests::expands_comparator_with_vchord_engine_and_lists`
  (`bench comparator --engine vchord --prefix real_100k --sweep 1,4,16,64
  --lists 320 --maintenance-work-mem 4GB --queries-limit 200 --log-output ...
  --rebuild`, with no `--profile`).

## Out of scope / follow-up

- Task 108.4 (run the deferred vchord probe sweep on AWS Graviton 4 to complete
  `benchmarks/comparators-50k-100k-1m/`) is the next packet.
- Task 109 (canonical per-lane standard sweep configs) is separate.
