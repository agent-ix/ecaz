---
id: US-017
title: Run Configured Benchmark Suites
type: user-story
artifact_type: US
status: APPROVED
relationships:
  - target: "ix://agent-ix/ecaz/StR-006"
    type: "derives_from"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-038"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/NFR-007"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/NFR-009"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/NFR-015"
    type: "derives_into"
    cardinality: "1:N"
---
# US-017: Run Configured Benchmark Suites

**As** a reviewer, extension developer, or platform engineer,
**I want** long benchmark sequences to run from a checked-in configuration,
**So that** AM onboarding, tuning sweeps, and RDS/Graviton measurements produce repeatable packet-local evidence without hand-running each command.

## Acceptance Criteria

### US-017-AC-1

Operators can define load, recall, latency, storage, EXPLAIN, and raw command steps in one suite configuration.

### US-017-AC-2

Operators can dry-run, execute, audit, and inspect suite status without rewriting the suite as shell scripts.

### US-017-AC-3

Suite runs write a manifest that records config identity, selected steps, expanded commands, execution status, timing, and artifact paths.

### US-017-AC-4

Suite documentation explains how to target specific benches by step name and how to use the runner during tuning sweeps.

### US-017-AC-5

Suite-generated reports and normalized rows preserve the candidate identity and
metric fields required by the benchmark reporting standard.
