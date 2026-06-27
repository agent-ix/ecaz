---
id: FR-037
title: Ecaz CLI Operator Surface
type: FR
status: IMPLEMENTED
object: interface
relationships:
  - target: "ix://agent-ix/ecaz/US-016"
    type: "implements"
    cardinality: "N:1"
---
# FR-037: Ecaz CLI Operator Surface

## Description

Ecaz SHALL provide a single `ecaz` CLI for operator workflows that prepare and load corpora, run benchmarks, compare external engines, exercise stress harnesses, inspect quantizer feasibility, and manage local development helpers.

## Behavior

1. The binary SHALL be named `ecaz` and expose global PostgreSQL connection flags: `--database`, `--host`, `--port`, `--user`, and `--password`.
2. Global connection flags SHALL fall back to `PGDATABASE`, `PGHOST`, `PGPORT`, `PGUSER`, and `PGPASSWORD` when omitted.
3. `--log-file` SHALL mirror command output into a stable artifact file and suppress transient progress bars.
4. Profile-aware workflows SHALL use the `IndexProfile` registry for `ec_hnsw`, `ec_ivf`, and `ec_diskann`.
5. `ecaz corpus` SHALL expose `fetch`, `prepare`, `generate`, `load`, `inspect`, and `list`.
6. `ecaz bench` SHALL expose `recall`, `latency`, `storage`, `overhead`, `diskann-graph`, `diskann-build-probe`, and `suite`.
7. `ecaz compare` SHALL expose `pgvector` and `vectorscale`.
8. `ecaz dev` SHALL expose `install`, `scratch`, `sql`, and `test`. The `ecaz dev` nested commands SHALL cover local ecaz/pgvector install, pgrx scratch restart/sql/debug-helper flows, pgrx SQL execution, pgrx tests, and PG18 preload/pgstat validation.
9. `ecaz quant` SHALL expose `feasibility` for offline quantizer recall and error-bound calibration.
10. `ecaz stress` SHALL expose `vacuum`, `ivf-insert`, and `ivf-vacuum-scale`.
11. Commands that interpolate relation, schema, or option names into SQL SHALL validate identifiers or reloption ownership before execution.
12. Reloption passthrough SHALL accept AM-specific `key=value` pairs while rejecting collisions with native CLI flags.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-037-AC-1 | `ecaz --help` exposes all top-level command groups and each group dispatches to the owning module | Test |
| FR-037-AC-2 | Profile-aware workflows select the correct access method, opclass, embedding type, encoder, scan GUC, sweep axis, and known reloption set from `profiles.rs` | Test |
| FR-037-AC-3 | Commands that produce review evidence can write packet-local logs without shell `tee` wrappers | Test |
| FR-037-AC-4 | The operator README and user docs list the full implemented command surface | Inspection |

### FR-037-AC-1

`ecaz --help` exposes all top-level command groups and each group dispatches to the owning module.

### FR-037-AC-2

Profile-aware corpus, benchmark, compare, and stress workflows select the correct access method, opclass, embedding type, encoder, scan GUC, sweep axis, and known reloption set from `profiles.rs`.

### FR-037-AC-3

Commands that produce review evidence can write packet-local logs without shell `tee` wrappers.

### FR-037-AC-4

The operator README and user docs list the full implemented command surface.

## Contract

```yaml
interface: ecaz
description: >-
  Top-level operator CLI. A single binary named `ecaz` dispatches to one of
  six command groups; PostgreSQL connection and logging flags are global and
  apply to every group.
global_flags:
  - name: --database
    type: string
    default: tqvector_bench
    env: PGDATABASE
    semantics: target PostgreSQL database name
  - name: --host
    type: string
    env: PGHOST
    semantics: host name or Unix socket directory; optional
  - name: --port
    type: u16
    env: PGPORT
    semantics: PostgreSQL port; optional
  - name: --user
    type: string
    env: PGUSER
    semantics: PostgreSQL user; optional
  - name: --password
    type: string
    env: PGPASSWORD
    semantics: PostgreSQL password (env values hidden); prefer .pgpass for non-local use
  - name: --log-file
    type: path
    semantics: >-
      mirror stdout/stderr into a packet-local artifact file and suppress
      progress bars so the file stays stable and diffable
operations:
  - name: corpus
    inputs: { subcommand: [fetch, prepare, generate, load, inspect, list] }
    output: corpus data moved in/out of Postgres or manifest report
    semantics: corpus plumbing - load fixtures, inspect what is loaded, verify manifests
  - name: bench
    inputs: { subcommand: [recall, latency, storage, overhead, diskann-graph, diskann-build-probe, suite, comparator, spire-pipeline] }
    output: measurement artifacts against a loaded corpus
    semantics: benchmarks (recall/latency/storage), plus standalone competitor measurement via `bench comparator`
  - name: dev
    inputs: { subcommand: [install, scratch, sql, test, fault, spire-multicluster, pg-upgrade-smoke, ...] }
    output: local install / pgrx scratch / SQL / test side effects
    semantics: development, setup, and test helpers owning the old wrapper-script surface
  - name: quant
    inputs: { subcommand: [feasibility], database_only: true }
    output: offline quantizer recall / error-bound study (no DB required)
    semantics: offline quantizer feasibility and recall studies
  - name: stress
    inputs: { subcommand: [vacuum, ivf-insert, ivf-vacuum-scale] }
    output: correctness-under-load result
    semantics: correctness-under-load harnesses (vacuum concurrency, crash recovery, ...)
  - name: cloud
    inputs: { subcommand: ecaz_cloud::CloudCommand }
    output: AWS stack lifecycle (delegated to the ecaz-cloud crate)
    semantics: cloud benchmark harness - provision, install, load, bench, teardown on AWS (FR-044)
```

## Dependencies

- **Upstream**: US-016 (implements)
- **Downstream**: FR-038 (extended by the configured benchmark suite runner)
