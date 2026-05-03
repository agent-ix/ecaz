---
id: FR-037
title: Ecaz CLI Operator Surface
type: functional-requirement
artifact_type: FR
status: IMPLEMENTED
object_type: interface
relationships:
  - target: "ix://agent-ix/tqvector/US-016"
    type: "implements"
    cardinality: "N:1"
---
# FR-037: Ecaz CLI Operator Surface

## Requirement

Ecaz SHALL provide a single `ecaz` CLI for operator workflows that prepare and load corpora, run benchmarks, compare external engines, exercise stress harnesses, inspect quantizer feasibility, and manage local development helpers.

## Behavior

1. The binary SHALL be named `ecaz` and expose global PostgreSQL connection flags: `--database`, `--host`, `--port`, `--user`, and `--password`.
2. Global connection flags SHALL fall back to `PGDATABASE`, `PGHOST`, `PGPORT`, `PGUSER`, and `PGPASSWORD` when omitted.
3. `--log-file` SHALL mirror command output into a stable artifact file and suppress transient progress bars.
4. Profile-aware workflows SHALL use the `IndexProfile` registry for `ec_hnsw`, `ec_ivf`, and `ec_diskann`.
5. `ecaz corpus` SHALL expose `fetch`, `prepare`, `generate`, `load`, `inspect`, and `list`.
6. `ecaz bench` SHALL expose `recall`, `latency`, `storage`, `overhead`, `diskann-graph`, `diskann-build-probe`, and `suite`.
7. `ecaz compare` SHALL expose `pgvector` and `vectorscale`.
8. `ecaz dev` SHALL expose `install`, `scratch`, `sql`, and `test`; nested commands SHALL cover local ecaz/pgvector install, pgrx scratch restart/sql/debug-helper flows, pgrx SQL execution, pgrx tests, and PG18 preload/pgstat validation.
9. `ecaz quant` SHALL expose `feasibility` for offline quantizer recall and error-bound calibration.
10. `ecaz stress` SHALL expose `vacuum`, `ivf-insert`, and `ivf-vacuum-scale`.
11. Commands that interpolate relation, schema, or option names into SQL SHALL validate identifiers or reloption ownership before execution.
12. Reloption passthrough SHALL accept AM-specific `key=value` pairs while rejecting collisions with native CLI flags.

## Acceptance Criteria

### FR-037-AC-1

`ecaz --help` exposes all top-level command groups and each group dispatches to the owning module.

### FR-037-AC-2

Profile-aware corpus, benchmark, compare, and stress workflows select the correct access method, opclass, embedding type, encoder, scan GUC, sweep axis, and known reloption set from `profiles.rs`.

### FR-037-AC-3

Commands that produce review evidence can write packet-local logs without shell `tee` wrappers.

### FR-037-AC-4

The operator README and user docs list the full implemented command surface.
