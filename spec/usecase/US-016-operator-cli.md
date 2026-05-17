---
id: US-016
title: Operate Ecaz Through One CLI
type: user-story
artifact_type: US
status: IMPLEMENTED
relationships:
  - target: "ix://agent-ix/ecaz/StR-006"
    type: "derives_from"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-037"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/NFR-009"
    type: "derives_into"
    cardinality: "1:N"
---
# US-016: Operate Ecaz Through One CLI

**As** a reviewer, platform engineer, or extension developer,
**I want** one profile-aware `ecaz` command surface for corpora, benchmarks, comparisons, stress harnesses, and local development,
**So that** evidence-producing workflows are reproducible and do not drift across one-off scripts.

## Acceptance Criteria

### US-016-AC-1

The main README and user docs point operators to the `ecaz` CLI for corpus loading, benchmarks, comparisons, stress harnesses, and development helpers.

### US-016-AC-2

The CLI documents every implemented top-level command group and subcommand.

### US-016-AC-3

Benchmark and diagnostic workflows support packet-local artifact logging for review evidence.
