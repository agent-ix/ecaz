# Manifest — Task 105 packet 003: local 10k config smoke

Packet-local source of truth for the local execution-smoke artifacts.
This packet is an **execution validation** of the Phase 2 sweep config
(`t105-sweep-10k.json`) before any paid AWS bench step — it doubles as
the local 10k reference column. No code under review here; the configs
under review are packet 002.

## Provenance

- **Task bucket / packet**: `reviews/task-105/003-local-10k-config-smoke/`
- **Git head**: post-Phase-1 local state (aarch64 NEON-first dispatch,
  IVF batch-decode default on, rabitq32 test envelope — the Phase 1
  slices merged as `main=1345ca603`). This is a **local x86 host**, so
  the merge SHA is the code identity; the exact running binary is pinned
  by the backend `.so` sha256 below rather than a git checkout marker.
- **Backend**: `release` build,
  `ecaz.so` sha256 `f8d64f667b0ad7bcf26df48601c2c452fba9eb5c686dbe47e450a3034de9f6d1`,
  path `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`
  (PG18.3). Per the local-build rule, this is the bench build, not a
  `pg_test` debug `.so`.
- **Lane**: local Intel desktop (not an AWS production lane).
- **Host / connection**: local pgrx, host `/home/peter/.pgrx`, port
  `28818`, database `postgres`.
- **Fixture**: local source-alias views over the real-DBpedia 10k
  fixture, built from `t105-fixtures-10k.sql` (packet 002) verbatim;
  see `fixtures-10k.log`.
- **Storage format / surface**: isolated **one-index-per-table**
  `t105_<am>_<quant>_10k` tables (11 variants), matching the AWS lane
  convention. Per-variant storage in `storage-t105_*_10k.log`.
- **Rerank mode**: per-family profile defaults (the sweep config sets
  no global rerank override); recall truth cache under `truth-cache/`.
- **Timestamp**: 2026-06-12T03:58:30Z (`generated_at_unix_ms`
  1781236710820).

## Command

Driven by `ecaz bench suite` (FR-038) against the packet-002 config:

    ecaz bench suite \
      --config reviews/task-105/002-full-scale-sweep-configs/artifacts/t105-sweep-10k.json \
      --artifacts-dir reviews/task-105/003-local-10k-config-smoke/artifacts

- Config: `reviews/task-105/002-full-scale-sweep-configs/artifacts/t105-sweep-10k.json`
- Config sha256: `51c5be53be8e223d8e5124c4d2e814ac2c82e40710afc23517a0cf802092cea2`
- `dry_run: false`; suite `task105-sweep-10k`; schema_version 1.
- Per-step commands are recorded in `suite-manifest.json` (`steps[].command`)
  and echoed in `suite-run.log`.

## Artifacts

| file | content |
|---|---|
| `suite-manifest.json` | structured run manifest (71 steps, per-step status/timing/command) |
| `results.jsonl` | 285 parsed result rows (latency, recall, counters, storage) |
| `suite-run.log` | runner stdout (per-step command echo) |
| `fixtures-10k.log` | fixture replication log |
| `latency-*-10k.log` (31) | per-step latency output |
| `recall-*-10k.log` (31) | per-step recall output |
| `storage-t105_*_10k.log` (11) | per-variant index storage |
| `truth-cache/` | recall ground-truth cache |

## Key result lines (cited by `request.md`)

- **Steps: 71/71 succeeded** (`suite-manifest.json` — all
  `steps[].status == "succeeded"`).
- **Recall parity: 28/28 on/off recall pairs byte-equal, 0 mismatches**
  (recomputed from `results.jsonl`).
- **Results: 285 rows** — metrics breakdown latency 60 / recall 60 /
  block_kernel_counters 42 / kernel_cell 2 / storage_field 99 /
  storage_index 22.
- **ISA attribution: `avx2` / `scalar` only** (28 / 14 counter rows),
  as expected for this x86 host — no foreign-lane attribution.

## Notes

- This is the local execution validation referenced by the Phase 2
  lane packets (004/005); it is **not** an AWS production-lane
  measurement and is not part of the published scale matrix's AWS cells.
- IVF on/off cells here are same-config pairs (see packet 002 request +
  packet 006 honest markers): the IVF kernel A/B is Task 99's pre-flip
  100k run, not this smoke.
