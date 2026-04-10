# Task 12: Real-Corpus Recall Validation

Status: not started — **can start immediately; now required to resolve the A4 / NFR-003 dataset contradiction**

## Scope

Add the real-corpus benchmark path needed to evaluate A4 on a dataset consistent with
`NFR-003` rather than only the current in-repo synthetic generators.

This task is not about weakening the recall target. It is about measuring the existing gate on a
credible dataset:

- DBpedia OpenAI embeddings, or
- a clearly documented equivalent real `1536`-dimensional embedding corpus

## Why This Exists

Recent A4 investigation changed the problem statement:

- the deterministic synthetic fixtures are reproducible, but they are no longer credible gate
  surfaces
- raw reference `hnsw-rs` on the same synthetic workload only reaches:
  - `29.0%` Recall@10 on uniform `10k` at `m=8, ef_search=128`
  - `26.0%` Recall@10 on the current clustered `10k` at `m=8, ef_search=128`
  - `66.5%` Recall@10 on uniform `10k` even at `m=16, ef_search=200`
- `NFR-003` already says recall benchmarks SHALL run against DBpedia OpenAI embeddings or
  equivalent

So A4 is now blocked on benchmark methodology as well as implementation correctness.

## Subtasks

- [ ] **Dataset contract.** Pick the first real-corpus benchmark source and document:
  - dataset name
  - licensing / local availability
  - row count
  - dimensionality
  - query split
  - any preprocessing required to reach the repo's expected `float4[]` shape
- [ ] **Local loader path.** Add a local-file loader that can ingest corpus rows and query rows
  without requiring a network fetch during tests. Prefer a simple, auditable format such as CSV,
  TSV, or line-delimited JSON over a tightly coupled ad hoc format.
- [ ] **External-corpus relation seam.** Reuse or extend the current relation-based recall probes so
  they can operate on:
  - an external corpus table
  - an external query table
  - the built tqhnsw index on that corpus
  while still comparing against brute-force fp32 truth.
- [ ] **Reusable fixture flow.** Keep the new path reusable, like the current fixture-backed gate:
  - one-time load
  - one-time index build
  - repeated report/probe reruns without rebuilding the corpus each time
- [ ] **A4 rerun on real data.** Measure the required A4 configurations:
  - `(m=8, ef=40)`
  - `(m=8, ef=128)`
  - `(m=8, ef=200)`
  - `(m=16, ef=200)`
  against brute-force fp32 truth on the real corpus.
- [ ] **Reporting surface.** Record dataset metadata and results in the same durable style as the
  synthetic A4 work:
  - graph Recall@10
  - exact quantized Recall@10
  - build-code / reference comparisons when useful
  - clear pass/fail statement against the existing gate

## Owns

- `NFR-003` real-dataset methodology for A4 and later benchmark reporting
- The "DBpedia OpenAI embeddings or equivalent" execution lane implied by the spec

## Dependencies

- Task 05 / A4 probe and debug surfaces on `main`
- Task 10 benchmark/report infrastructure

## Unblocks

- A4 decision-making on a credible dataset
- Final `NFR-003` benchmark reporting
- Any project decision about whether the current A4 failure is an implementation defect or a
  synthetic-fixture mismatch

## Deliverables

- A documented local dataset contract
- A loader script / SQL path for corpus and query tables
- Reusable relation-backed recall measurement on external data
- A benchmark report or review packet with the first real-corpus A4 results

## Primary Tests

- Loader smoke test on a tiny external sample
- Stable rerun of the external-corpus recall report without corpus rebuild
- A4 result table on the chosen real corpus

## Notes

- Deterministic synthetic fixtures remain useful for debugging graph/runtime invariants. They are
  not retired by this task.
- Keep large datasets out of the repository. Check in metadata, loader logic, and benchmark
  documentation, not the corpus itself.
- Prefer local reproducibility over cleverness: explicit file contract, explicit query split,
  explicit seeds or row selections where applicable.
- This task should reuse the current SQL/debug probes rather than invent a second benchmark stack.
