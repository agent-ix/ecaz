# Task 57 Packet 005 — Closeout Artifact Manifest

## Provenance

- Branch: `task-57`
- HEAD at close: this packet's owning commit
- Pre-Task-57 HEAD (main merge baseline): `9afb2c6b8`
- Closeout scope: IVF subsystem under `src/am/ec_ivf/`.

## Artifacts

This closeout cites artifacts from the upstream burndown packet so
that there is one provenance source per artifact:

- **Block counts**:
  `reviews/task-57/004-additional-burndown/artifacts/block-counts.txt`
  (per-file IVF `unsafe { }` counts + `src/` total at slice close).
- **cargo check (lib pg18)**:
  `reviews/task-57/004-additional-burndown/artifacts/cargo-check.log`.
- **cargo check (all-targets pg18)**:
  `reviews/task-57/004-additional-burndown/artifacts/cargo-check-all-targets.log`.

## Bench gate

**Status: executed.** All 4 steps Succeeded under
`ecaz bench suite run`. Artifacts in this directory:

- `suite.json` — packet-local `SuiteConfig` (sha256
  `e72627fe81629993ff1307d71ae4f767cfac5ddd4f5b5c9fac505181f531b23a`).
- `suite-run.log` — `ecaz bench suite run` stdout/stderr.
- `suite-manifest.json` — structured suite manifest emitted by the
  runner.
- `results.jsonl` — per-step parsed metric records (recall, latency,
  storage_field, storage_index).
- `load-10k-ivf-rabitq-n64.log` — `ecaz corpus load` output.
- `recall-10k-ivf-rabitq-n64.log` — recall sweep raw output.
- `latency-10k-ivf-rabitq-n64.log` — latency sweep raw output.
- `storage-10k-ivf-rabitq-n64.log` — storage report raw output.
- `truth-ec-real-10k-q200-k10.json` — cached ground-truth top-k for
  the 200 queries against the 10k corpus (recall reference).

Headline results (see `request.md` §Bench for full tables):

- recall@10 reaches 1.0000 (ci95 lower 0.9981) at nprobe ≥ 16
- latency mean 0.43 ms → 1.36 ms across nprobe 8 → 64
- IVF index size 3.6 MiB / 381.7 B per row at 10k rows.

Build invariant: `cargo pgrx install --release --no-default-features
--features pg18` was run immediately before the suite so PG18 loaded
the Task-57 HEAD binary, not the prior debug build.
