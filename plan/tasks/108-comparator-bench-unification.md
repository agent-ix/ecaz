# Task 108: Comparator Bench Unification

Status: proposed (2026-06-16)
Owner: unassigned
Priority: 2 (benchmark-workflow consolidation)

## Why

Standalone competitor (comparator) measurement is currently impossible through
`ecaz` without also re-measuring ecaz. The only comparator surface is
`ecaz compare {pgvector,vectorscale}` (`crates/ecaz-cli/src/commands/compare/`),
which is *head-to-head*: every invocation re-measures ecaz **and** the
comparator together. That couples comparator measurement to an ecaz
re-measurement, directly violating the no-re-run policy — comparators are
re-run only on a competitor-version or hardware change, never for an ecaz code
change.

To work around that, a one-off bash comparator path was added under
`scripts/comparators/` (`sweep.sh`, `compute_recall.py`, `run_all.sh`,
`_bench_lib.sh`, `_common.sh`, per-engine `load`/`bench`). It is the only thing
that can capture a single competitor's standalone Pareto (p50/p95/p99 latency +
recall@10 across a query-GUC sweep), but it (a) duplicates recall + tuning the
suite already does, and (b) forks the workflow into bash against the FR-038
"`ecaz bench suite` only" rule. It produced
`benchmarks/comparators-50k-100k-1m/`, but the **vchord probe sweep was
explicitly deferred** there — vchord has only a single `default` cell while
pgvector/pgvectorscale have full sweeps.

## Scope

1. **New standalone, engine-generic comparator command.** Add
   `crates/ecaz-cli/src/commands/bench/comparator.rs`, wired as
   `BenchCommand::Comparator(ComparatorArgs)` in
   `crates/ecaz-cli/src/commands/bench/mod.rs`. Shape it on `bench/recall.rs` +
   `bench/latency.rs` (NOT on `compare/*`); it measures **one** external engine
   standalone — no ecaz engine in the loop.
   - `--engine {vchord | pgvector-hnsw | pgvector-ivfflat | pgvectorscale}`
     (required; lantern is out of scope).
   - `--prefix`, `--k` (default 10), `--queries-limit`.
   - `--sweep` = the engine's query GUC values (the tuning axis): vchord
     `vchordrq.probes`; pgvector-hnsw `hnsw.ef_search`; pgvector-ivfflat
     `ivfflat.probes`; pgvectorscale `diskann.query_search_list_size`.
   - build knobs per engine: IVF-family `--lists`
     (default `ceil(sqrt(row_count))`, pinnable to 224/320/1024); hnsw `--m`,
     `--ef-construction`; pgvectorscale `--num-neighbors`,
     `--build-search-list-size`, `--max-alpha`, `--storage-layout`.
   - `--maintenance-work-mem` (default `4GB`), `--rebuild`, `--log-output`.
   - Per sweep value (single pass, both metrics): ensure extension; build
     sidecar table `{prefix}_corpus_<engine>` from `{prefix}_corpus.source` and
     the engine index idempotently (`vector_ip_ops`, `<#>`); brute-force ground
     truth via `recall::brute_force_top_k` + `recall::fetch_sources_public`;
     `SET <query_guc>=<value>`, run the query set once capturing per-query
     top-k ids + durations; emit recall@k (`recall::recall_summary_at_k`),
     latency percentiles (`latency::summarize`), and a `pg_relation_size`
     storage line — one Pareto row per sweep value, in the **same output
     format** as recall/latency/storage so the suite parser reads it unchanged.
   - **Salvage** the sidecar/populate/index-build SQL builders out of
     `compare/pgvector.rs` + `compare/vectorscale.rs` into `comparator.rs`;
     discard the head-to-head ecaz-vs-X loop, `measure_engine`, `ComparisonRow`,
     `print_comparison`. Add new vchord builders
     (`USING vchordrq (embedding vector_ip_ops) WITH (options = $vco$ ...
     residual_quantization = true ... lists = [N] ... $vco$)`).
   - New suite step `Comparator(ComparatorStep)` (kebab kind `"comparator"`) in
     `suite.rs`, modeled on `RecallStep`/`LatencyStep`; `expand_comparator()`
     emits `bench comparator --engine ... --sweep ...`; parse arm reuses the
     recall/latency/storage row parsers.

2. **Drop the head-to-head `compare` surface.** Delete
   `crates/ecaz-cli/src/commands/compare/` entirely; strip references in
   `commands/mod.rs`, `cli.rs`, and the compare step machinery in `suite.rs`
   (`ComparePgvector`/`CompareVectorscale` variants, `*Step` structs,
   `expand_compare_*`, all method arms, `parse_compare_*` if unused). Migrate
   the 3 committed configs
   (`profile-cross-engine-real10k`, `profile-hnsw-100k`, `profile-ivf-100k`) to
   the `comparator` step. Update `crates/ecaz-cli/README.md`.

3. **Retire the bash measurement scripts.** Remove `sweep.sh`,
   `compute_recall.py`, `run_all.sh`, `_bench_lib.sh`, `_common.sh`, per-engine
   `load.sh`/`bench.sh`, and the entire `lantern/` directory from
   `scripts/comparators/`. **Keep** the `install*.sh` scripts
   (pgvector/pgvectorscale/vchord — extension install + `shared_preload_libraries`
   + PG restart is an operator prerequisite the CLI can't do) and `README.md`,
   trimmed to "install prerequisite, then use `ecaz bench comparator`".

4. **Run the missing vchord comparisons (vchord-only).** On AWS Graviton 4
   (m8g) restored from corpus base snapshot `snap-0e9c7743263e61d70` (matching
   the recorded pgvector/pgvectorscale host; follow snapshot/no-recreate rules).
   Install vchord via the kept `install.sh`. Run
   `ecaz bench comparator --engine vchord` at 50k/100k/1m, `--lists`
   224/320/1024, `--sweep 1,4,16,64`, `k=10`, via an `ecaz bench suite` config
   with a `comparator` step (FR-038 compliant), logging to the packet's
   `artifacts/`. Complete `benchmarks/comparators-50k-100k-1m/` — add the vchord
   `p{1,4,16,64}` cells, regenerate `_pareto.tsv`, update `manifest.md`. Trust
   existing pgv/pgvscale data (not re-run).

## Acceptance criteria

- `ecaz bench comparator --engine <e>` measures any one of the four engines
  standalone, emitting recall@k + latency percentiles + storage per sweep value
  in suite-parseable format.
- `compare` command, compare suite steps, and the bash measurement scripts are
  removed; `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  and `cargo build` are clean.
- Unit coverage for the per-engine SQL builders (incl. vchord DDL),
  `expand_comparator`, and the `--lists` default/override.
- `ecaz bench suite --config <vchord-packet>.json --dry-run` expands the
  `comparator` step to the expected argv.
- vchord probe sweep produces a monotone probes→recall curve and the high-probe
  cell sanity-checks against the recorded vchord cell; the
  `benchmarks/comparators-50k-100k-1m/` packet is completed with the new cells,
  regenerated `_pareto.tsv`, and an updated `manifest.md`.

## Coordination

- Pairs with Task 109 (standardized ecaz sweep config). Independent of the
  kernel lanes.
- vchord/pgvectorscale/pgvector **installation** stays a manual operator
  prerequisite (the CLI cannot edit `shared_preload_libraries` + restart PG).
- pgvector + pgvectorscale recorded recall/tuning data is **trusted, not
  re-run** (no-re-run policy).
