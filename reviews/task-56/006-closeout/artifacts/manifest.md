# Task 56 Packet 006 — Closeout Artifact Manifest

## Provenance

- Branch: `task-56-spire-burndown`
- Pre-Task-56 HEAD: `329925109` (Task 57 merge).
- Post-Task-56 HEAD: `af69128e8` (slice 006 — wal.rs revert +
  safety-doc parity + typed-handle refactor) + this packet's commit.
- Closeout scope: SPIRE subsystem under `src/am/ec_spire/`.

## Artifacts

- `final-block-counts.txt` — per-file `unsafe { … }` counts at the
  post-Task-56 HEAD.
- `suite.json` — packet-local `SuiteConfig` for the SPIRE 10k bench
  gate.
- `suite-run.log` — `ecaz bench suite run` stdout/stderr.
- `suite-manifest.json` — structured suite manifest emitted by the
  runner.
- `results.jsonl` — per-step parsed metric records.
- `load-10k-spire.log` — load step raw output.
- `recall-10k-spire.log` — recall sweep raw output.
- `latency-10k-spire.log` — latency sweep raw output.
- `storage-10k-spire.log` — storage report raw output.

## Bench gate headline

- Recall: 0.9920 → 1.0000 (ci95) across nprobe 8 → 24.
- Latency: 5.44 ms → 17.2 ms mean across nprobe 8 → 32.
- Storage: 9.4 MiB SPIRE index / 17,691 B per-row total at 10k rows.

Build invariant: `cargo pgrx install --release --no-default-features
--features pg18` was run immediately before the suite so PG18 loaded
the Task-56 HEAD binary, not the prior debug build.
