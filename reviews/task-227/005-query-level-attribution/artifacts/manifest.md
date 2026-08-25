# Task 227 packet 005 artifact manifest

- Implementation head: `43a20a0d233f620ae19a5f02f26715c390fe4eae`
- Implementation commits after packet 004:
  `8ccc635d6`, `9187e8261`, `59b7adbed`, `1a36ac6d6`, `1126c114c`,
  `048e4e58c`, `66cba243e`, and `43a20a0d2`
- Task bucket / packet: `reviews/task-227/005-query-level-attribution/`
- Lane: PG18 release, physical three-owner DistANN, frozen 100k diagnostic
  slice, five diagnostic variants, one reused generation
- Timestamp: 2026-08-24 PDT (America/Los_Angeles)
- Fixture: isolated one-index-per-table physical generation plus its separately
  built monolithic control; no shared-table comparison surface
- Storage / rerank: persisted RaBitQ graph, lazy-10 materialization; RaBitQ and
  benchmark-only exact-neighbor diagnostic arms as registered in the task

## Provenance

- Runner head: `43a20a0d233f620ae19a5f02f26715c390fe4eae`
- Preserved extension head: `9187e82618f228beb9bbf7f9810fb2d6c767a951`
- Extension build: release,
  `distann-head-attribution-benchmark,pg-test,pg18`
- Physical generation identity:
  `0200797e7ccf2576a229d4af60cdb23e1ffed5854a032066f2d37c623436b810c210`
- Corpus prefix / scale: `ec_real_100k`, 100,000 rows
- Query parent: 1,000 rows, SHA-256
  `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- Diagnostic slice: rows 201--400 (`query_offset=200`, `queries=200`),
  SHA-256
  `a12a81111d586e78165a950962aa8667e2f95e700159fe86d83bba2b49a5ece9`
- Fixture run directory:
  `/home/peter/.ecaz/clusters/task227-attribution-diagnostic-100k`; outside the
  repository as required. It was operational state, not evidence, and was
  removed after the cited packet artifacts were captured (8.9 GiB reclaimed).

## Suite artifacts

- `reuse-suite.json`
  - Checked-in `SuiteConfig` for the single diagnostic step and five frozen
    variants.
- `reuse-run-success/suite-manifest.json`
  - Command: `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-227/005-query-level-attribution/artifacts/reuse-suite.json --artifact-dir reviews/task-227/005-query-level-attribution/artifacts/reuse-run-success --log-file reviews/task-227/005-query-level-attribution/artifacts/reuse-suite-run-success.log`
  - Runner head and expanded command are recorded in the manifest. The single
    selected step succeeded with exit code 0 in 4,872,803 ms.
- `reuse-run-success/results.jsonl`
  - Structured suite metrics parsed from the successful step summary.
- `reuse-suite-run-success.log`
  - Suite runner log for the successful run.
- `reuse-attestation/distann-multinode-summary.log`
  - Preserved-fixture build, extension, generation, corpus, and original matrix
    control attestation used before mutation.

The decisive step artifacts are under
`reuse-run/diagnostic-reuse-rows-201-400/`:

- `physical-*-recall.log` and `physical-*-predictions.json`
  - Production BW4/RaBitQ recall 0.9295.
  - BW8/RaBitQ recall 0.9565; 29 query wins, 3 query losses, mean paired delta
    +0.027.
  - Production-seed BW4/exact-neighbor recall 0.9265.
  - Owner-seed BW4/RaBitQ recall 0.9955; owner-seed BW4/exact-neighbor recall
    0.9960.
- `physical-*-query-trace.json`
  - Five variants x 200 traces. Every trace is untruncated and contains exactly
    10 final ids. The production BW4 trace preserves 36--80 exact-rerank ids
    per query (mean 47.2) before final-result truncation.
- `physical-residual-attribution.jsonl`
  - SHA-256
    `a576e7abd907db8c9452fdd5e222fc17ce30de281968946a9b3dbc55e2a584b4`.
  - 141 missed truth-neighbor rows, all classified `budget_frontier`.
- `physical-residual-attribution-summary.json`
  - 2,000 truth neighbors; 141 misses; 0 unknown; reconciliation passed;
    Task 189 same-seed approximate-ordering trigger false.
- `physical-residual-query-features.jsonl`
  - SHA-256
    `2ad616c50922c8e35b6c24ca74da40c47ddbc84c27285bbc8d0a9d55860c5655`.
  - 200 truth-free feature rows; no truncated traces.
- `physical-graph-diagnostic.json`
  - SHA-256
    `57c86fdd80cfdf9b2823254eed3694e113b3feab119fde509e361ee151a6af8a`.
  - Physical and monolithic: 100,000 live nodes, 3,101,447 directed edges,
    one weak component, 12 SCCs, largest SCC 99,989, matching degree shapes,
    and zero invalid/duplicate/self edges or aggregate bridge/articulation
    candidates. The physical persisted head reaches all live nodes.
- `physical-*-latency.log`
  - Instrumented diagnostic timings only. They are not candidate production
    latency evidence and are not used for selection.
- `distann-local-multinode.log`, `distann-multinode-summary.log`, and
  `node*-postgres.log`
  - Compact step/run evidence. The runner's generic post-measurement Task 167
    append/fresh-rebuild probes execute after the Task 227 artifacts and report
    `pass=false` on this reused fixture; they are not Task 227 gates or cited
    findings. That tail mutated the fixture, which was then removed. The
    preregistered `NO RELIABLE SIGNAL` stop independently forbids reusing it for
    the blind slice.

## Frozen rule decision

- `finite-rule-screen.json`
  - Source: `physical-residual-query-features.jsonl` at the SHA above.
  - Thresholds use nearest-rank p25/p75 exactly as preregistered.
  - Simulated recall uses the candidate delta only on activated queries.
  - Bootstrap: 10,000 resamples, fixed u64 xorshift seed
    `0x9e3779b97f4a7c15`, sorted indices 250/9749.
  - All seven rule rows are ineligible; decision `NO RELIABLE SIGNAL`; STOP
    before blind evaluation and runtime implementation.

## Focused validation

- `query-slice-reuse-suite-tests.log`: 6 passed, 0 failed.
- `diagnostic-replay-regression-test.log`: 1 passed, 0 failed.
- `reuse-provenance-parser-test.log`: 1 passed, 0 failed.
- `reuse-matrix-provenance-test.log`: 1 passed, 0 failed.
- `reuse-control-validation-compile-test.log`: 1 passed, 0 failed.
- `test-query-trace-quality-bar.log`: target regression 1 passed, command exit
  0; unrelated targets selected zero tests.
- `test-cli-query-trace-settings.log`: 1 passed, 0 failed.
- `build-release-cli-43a20a0d2.log`: release CLI build passed; one known
  unrelated dead-field warning remains in `corpus/load.rs`.

No corpus/query/truth TSV, truth cache, PGDATA directory, polling snapshot,
raw operational exhaust, or repository-wide formatting output is included.
No adaptive candidate exists, so the preregistered plan forbids blind-slice
evaluation and a 10k/50k/100k candidate matrix.
