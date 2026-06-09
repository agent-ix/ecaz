# Task 87 Phase 7 Real10k Counter Suite

## Scope

This packet is the first Phase 7 real-corpus counter run after packets 017-019 added direct scoring-share counters and same-backend benchmark capture.

It is not the final Phase 7 closeout. It validates the measurement path on the accepted real10k surfaces and exposes one important call-shape finding before scaling to the full real10k/50k/100k matrix.

Head under test:

- `345a59659` plus packet-local PG18 install from the current branch.

Suite config:

- `phase7-real10k-counter-suite.json`

## Setup

- Rebuilt `target/debug/ecaz` with `cargo build -p ecaz-cli`.
- Installed the PG18 pg_test extension build:
  - `target/debug/ecaz --log-file reviews/task-87/020-phase7-counter-suite/artifacts/install-ecaz-pg-test.log dev install ecaz-pg-test --pg 18`
  - installed backend SHA256: `23b39c72fcfd2071c07db950363f9d30e93940f0d48053c67172568e185261e4`
- Verified the new Task 87 counter SQL functions were initially absent in the existing `postgres` database:
  - `artifacts/counter-function-check.log`
- Registered just the two new counter functions using the packet-local SQL file:
  - `artifacts/register-task87-counter-functions.sql`
  - `artifacts/register-task87-counter-functions.log`
- Verified both functions were registered:
  - `artifacts/counter-function-check-after-register.log`

## Suite Evidence

Audit and dry-run:

- `artifacts/suite-audit.log`
  - `target/debug/ecaz bench suite audit --config reviews/task-87/020-phase7-counter-suite/phase7-real10k-counter-suite.json`
  - Result: audit passed, 10 steps.
- `artifacts/suite-dry-run.log`
  - Confirms `--task87-candidate-batch-counters` expands on IVF latency, SPIRE pipeline, and HNSW latency steps.

Run:

- `artifacts/real10k-run.log`
- `artifacts/real10k-run-manifest.json`
- `artifacts/real10k-results.jsonl`
- Suite status: completed 10, failed 0, skipped 0, missing artifacts 0, stale 0.

## Key Results

### IVF real10k

- Recall byte-equal:
  - off: recall@10 `1.0000`
  - on: recall@10 `1.0000`
- End-to-end latency improved:
  - off: p50 `19.7 ms`, p95 `20.9 ms`, p99 `23.5 ms`
  - on: p50 `16.7 ms`, p95 `18.7 ms`, p99 `23.2 ms`
- Same-backend Task 87 counters:
  - off: IVF counters all zero.
  - on: `flushes=8000`, `candidates=2000000`, `elapsed_ms=2302.509555`, `lut32_flushes=7800`, `lut32_candidates=1996800`.

Interpretation: IVF real10k is hitting the Phase 7 LUT32 path for 7,800/8,000 flushes and 1,996,800/2,000,000 candidates. This is a valid touched-kernel surface.

### SPIRE real10k

- Recall byte-equal:
  - off: recall@10 `1.0000`
  - on: recall@10 `1.0000`
- Coordinator query latency improved:
  - off: p50 `19.091 ms`, p95 `22.515 ms`, p99 `24.009 ms`
  - on: p50 `16.951 ms`, p95 `18.087 ms`, p99 `20.206 ms`
- Same-backend Task 87 counters:
  - off: SPIRE counters all zero.
  - on: `flushes=157548`, `candidates=1551640`, `elapsed_ms=2169.038804`, `lut32_flushes=0`, `lut32_candidates=0`.

Interpretation: SPIRE real10k uses the shared CandidateBatch scorer but does not hit the 32-block LUT32 route in this run. Average candidates per flush are about 9.85, so the current SPIRE call shape does not satisfy the Phase 7 kernel-routing intent even though end-to-end latency still improves.

### HNSW real10k

- HNSW candidate-batch-on latency step completed:
  - p50 `5.19 ms`, p95 `11.2 ms`, p99 `36.4 ms`
- Same-backend Task 87 counters:
  - all surfaces zero.

Interpretation: this HNSW latency surface did not exercise the TurboQuant no-QJL 4-bit CandidateBatch scorer. It is not evidence for routing HNSW through LUT32; it is evidence that this real10k HNSW surface should not be counted as a Phase 7 touched-kernel cell.

## Next Work

- Fix or explicitly stop-condition the SPIRE call shape before final Phase 7 closeout. The current real10k SPIRE path has useful CandidateBatch counters but zero LUT32 blocks.
- Run Phase 7 50k/100k counter suites after resolving the SPIRE call-shape decision.
- Build the superseding aggregate matrix and completion audit only after the full Phase 7 evidence is available.
