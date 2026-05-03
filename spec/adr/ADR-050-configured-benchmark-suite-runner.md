---
id: ADR-050
title: Configured benchmark suite runner for indexes and architectures
status: PROPOSED
impact: HIGH for US-015, US-016, FR-037, NFR-007, NFR-009, StR-006
date: 2026-05-03
---
# ADR-050: Configured benchmark suite runner for indexes and architectures

## Context

The Task 31 M5 IVF work produced a long sequence of useful benchmark packets:

- load fixed-size real corpora at 10k, 25k, 50k, and 100k
- preserve packet-local raw logs, truth caches, storage output, and EXPLAIN/counter SQL
- sweep `nprobe` and rerank width on the loaded 100k surface
- repeat candidate points before recommending a balanced and quality-biased setting

That workflow worked, but it was manually orchestrated. Manual orchestration is too expensive and too easy to drift when:

- onboarding a new index access method
- testing a new architecture such as M-series, x86, or RDS Graviton
- comparing local development hardware with cloud instances
- rerunning a known suite after an implementation change
- running longer overnight or weekend benchmark lanes
- narrowing to a specific index, size, or parameter range during tuning

Existing `ecaz` subcommands already own the right primitive operations:
`corpus load`, `bench recall`, `bench latency`, `bench storage`, and `dev sql`.
The missing piece is a declarative suite runner that coordinates those primitives, records provenance, and can run unattended.

## Decision

Add an `ecaz bench suite` command that consumes a versioned suite configuration and executes benchmark steps as first-class `ecaz` operations.

The suite runner SHALL be an orchestration layer over existing CLI surfaces, not a separate benchmark implementation. A suite step expands to ordinary `ecaz` commands so results remain comparable with packet-local manual runs.

The suite configuration format SHALL be structured JSON initially. JSON is already available through `serde_json`, keeps the first implementation dependency-free, and is easy for automation to generate. A future ADR may add TOML/YAML aliases if human editing becomes painful.

## Suite Model

A suite file SHALL include:

- suite name and schema version
- target metadata: intended hardware/architecture, database target, cache policy, and run purpose
- artifact root
- defaults: profile, query limit, iterations, memory sampling, and force-index behavior
- surfaces: named corpora and index builds
- measurements: recall, latency, storage, EXPLAIN/counters, optional stress/churn checks
- selection/repeat policy: repeat count, candidate points, and pass/fail thresholds when applicable

The suite runner SHALL support these step types in the first implementation:

- `load`: wraps `ecaz corpus load`
- `recall`: wraps `ecaz bench recall`
- `latency`: wraps `ecaz bench latency`
- `storage`: wraps `ecaz bench storage`
- `explain`: generates packet-local SQL and wraps `ecaz dev sql`
- `raw`: explicit escape hatch for an existing `ecaz` command not yet modeled

The suite runner SHOULD write a machine-readable suite manifest containing:

- suite config path and config hash
- `ecaz` version and git SHA when available
- host/OS/architecture metadata when available
- connection target with password redacted
- expanded command for every step
- start/end timestamps, duration, exit status
- artifact paths produced by each step
- skipped/dry-run/failure status

The suite runner SHOULD also emit a Markdown summary report with concise tables for the common benchmark outputs. Raw logs remain the source of truth.

## Required Benchmark Coverage

A full index/architecture onboarding suite SHOULD cover the following lanes.

### Preflight

- `ecaz corpus list` or equivalent environment status
- extension version and PostgreSQL version
- host/architecture metadata
- relevant GUCs and preload state
- corpus manifest verification and hashes

### Build and Storage

For each index configuration:

- load/build timing
- index reloptions
- table and index storage
- per-row index bytes
- build warnings and manifest mismatch status

Build surfaces SHOULD be isolated one-index-per-table by default so benchmark results are not contaminated by planner or shared-table effects. Shared-table suites are allowed, but must declare that surface mode explicitly.

### Quality

At minimum:

- recall@10 with a truth cache
- NDCG@10

For quality-sensitive or high-scale points:

- recall@100
- NDCG@100
- reuse exact truth caches between comparable sweeps

Truth caches SHALL be packet/suite-local artifacts or explicitly referenced immutable shared artifacts.

### Latency

At minimum:

- p50, p95, p99, max
- iteration count
- concurrency
- force-index status
- cache policy
- backend memory sampling status

Long-running suites SHOULD support repeats per candidate point. A candidate should not be called a selected point from one latency run unless the suite explicitly marks it as exploratory.

### Counters

For representative points, collect EXPLAIN/counter SQL including:

- execution time
- buffer hit/read blocks
- selected lists or graph search budget
- posting pages read or AM-equivalent scan pages
- postings/candidates visited
- postings/candidates scored
- postings/candidates pruned
- candidates inserted
- rerank rows
- duplicate filters

Counter capture is required for bottleneck selection, because wall time alone does not distinguish scan volume, scoring cost, rerank cost, heap access, or client/protocol overhead.

### Stress or Churn

Index-specific suites MAY include:

- insert/churn workload
- vacuum consistency
- restart/recovery safety
- planner/shared-table behavior
- concurrent read/write checks

These should be separate lanes from pure recall/latency unless the suite explicitly targets mixed workload behavior.

## Flexibility Requirements

Suites SHALL be decomposable by target:

- run all steps
- run only named steps
- run by tag, such as `load`, `quality`, `latency`, `storage`, `counter`, `candidate`, or `stress`
- skip expensive steps such as fresh load/build or recall@100
- resume after a failed step
- dry-run expanded commands

Suites SHALL let operators vary:

- access method/profile
- corpus size and corpus path
- index reloptions
- sweep axis values, such as `nprobe`, `ef_search`, or DiskANN list size
- rerank width
- recall `k`
- query limit
- latency iterations and concurrency
- cache policy
- output artifact root

This flexibility is required for both broad onboarding and narrow tuning. A Graviton onboarding suite might run the full matrix overnight, while an implementation-tuning suite might run only `100k n128 p80/p96 rerank_width=500/750/1000`.

## Reporting

Each suite run SHALL preserve raw artifacts and SHOULD produce derived reports:

- `suite-manifest.json`: machine-readable provenance and step status
- `summary.md`: human-readable tables and interpretation stubs
- optional `results.jsonl`: normalized result rows for plotting/comparison

The runner SHOULD avoid making product claims automatically. It can identify candidate points and threshold pass/fail states, but final interpretation remains a review-packet or operator decision.

The report SHOULD distinguish:

- local development evidence
- cloud architecture evidence
- product-grade benchmark claims

For RDS Graviton, the suite should record instance class, storage class, database parameter group details, PostgreSQL version, extension SHA/version, region/AZ if acceptable for the environment, and whether the run used cold, warm, or mixed cache.

## Failure Behavior

Default behavior:

- stop on first failed step
- write the suite manifest before exiting
- keep all artifacts already produced

Optional behavior:

- `--continue-on-error` records failures and continues independent later steps
- `--resume` skips successful steps with matching artifact metadata
- `--allow-missing-optional` lets exploratory suites omit declared optional lanes

Failures in manifest verification should default to hard failure. `--allow-manifest-mismatch` remains explicit per load step and must be visible in the suite report.

## Security and Operational Boundaries

The suite runner SHALL not require shell scripts or `/tmp` workarounds. It should use normal repository paths and normal user tool layouts.

Connection passwords SHALL not be written into command manifests. If a password is supplied through `PGPASSWORD`, the manifest records only that password auth was configured.

The suite runner SHOULD support long-running unattended operation, but it should not hide destructive behavior. Drop/recreate steps, table deletion, and restart operations must be explicit step types or raw commands with prominent names.

## Consequences

### Benefits

- Reproducible benchmark onboarding for new indexes and architectures.
- Less manual drift in artifact names, truth cache paths, and reloptions.
- Easier overnight and cloud runs.
- Better comparison between local M-series, x86, and RDS Graviton results.
- Clear separation between raw artifacts and derived interpretation.

### Tradeoffs

- The config schema becomes another operator API and needs compatibility discipline.
- Long-running suites can consume large disk/database resources if configs are careless.
- Report parsing may be brittle if it depends on human table formatting; normalized JSON result rows should follow.
- `raw` escape hatches are pragmatic but can weaken schema validation if overused.

## Implementation Plan

1. Add `ecaz bench suite --config <path> --dry-run`.
2. Support `load`, `recall`, `latency`, `storage`, `explain`, and `raw` steps.
3. Expand steps to ordinary `ecaz` commands and record them in `suite-manifest.json`.
4. Add a checked-in Task 31 M5 IVF suite config based on packets `30169` through `30176`.
5. Add `--only <step>` and `--continue-on-error`.
6. Add tags and `--only-tag <tag>`.
7. Add normalized `results.jsonl` extraction for recall/latency/storage tables.
8. Add resume support keyed by config hash, expanded command, and artifact existence.
9. Add a Graviton/RDS template config with cloud metadata fields.

## Follow-Up

- Decide whether JSON remains sufficient or whether a human-facing TOML/YAML layer is justified.
- Add review-packet generation helpers only after suite manifests are stable.
- Add explicit cold-cache/restart policy support as a separate ADR if cloud benchmark claims need it.
