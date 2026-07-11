# Task 163 packet 004 artifacts manifest

- **Head SHAs:** implementation
  `de9d6fca3e0bd05f44ad6b0d376a2480e4023798`; test-only all-target clippy
  follow-up `c9b74c4f258c699def7ffc951e6abf47762565a4`
- **Task bucket / packet:** `reviews/task-163/004-d8-review-fixes`
- **Branch:** `task-179-ec-distann-physical-shards`
- **Timestamp:** `2026-07-10T20:24:00-07:00`
- **Lane:** Task 163 D8 / ADR-085 feedback remediation
- **Fixture:** pure corrupt-spool/shard suite plus focused live local PG18
- **Storage format:** temporary per-shard PostgreSQL `BufFile`; no persisted
  index-format change
- **Rerank mode / corpus:** not applicable
- **Isolation:** focused tests create one isolated ec_distann table/index; no
  shared-table benchmark surface and no benchmark measurement is claimed

## Provenance

All commands ran from a clean detached worktree at the applicable head SHA
above. The first three artifacts use the implementation SHA; the all-target
clippy artifact uses the follow-up SHA. `CARGO_TARGET_DIR` points at the owning
branch's target directory for dependency reuse; source, SQL, and generated
extension schema came from the detached exact-SHA checkout.

## Commands and artifacts

```text
CARGO_TERM_COLOR=never CARGO_TARGET_DIR=/home/peter/dev/ecaz/.claude/worktrees/task-179-physical-shards/target cargo test --lib am::ec_distann::shard_build::tests --no-default-features --features pg18
CARGO_TERM_COLOR=never CARGO_TARGET_DIR=/home/peter/dev/ecaz/.claude/worktrees/task-179-physical-shards/target cargo clippy --lib --no-default-features --features pg18 -- -D warnings
CARGO_TERM_COLOR=never CARGO_TARGET_DIR=/home/peter/dev/ecaz/.claude/worktrees/task-179-physical-shards/target cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings
CARGO_TERM_COLOR=never CARGO_TARGET_DIR=/home/peter/dev/ecaz/.claude/worktrees/task-179-physical-shards/target cargo pgrx test pg18 --no-default-features --features pg18 ec_distann_d8_multiblock_buffile_spool
```

- `cargo-test-shard-build.log` — 16 focused pure tests, including six corrupt
  spool failures and retained-memory accounting.
- `cargo-clippy-pg18.log` — strict PG18 library clippy.
- `cargo-clippy-all-targets-pg18.log` — strict PG18 all-target clippy after the
  test-helper spelling follow-up.
- `cargo-pgrx-test-d8-multiblock.log` — focused live PG18 runtime `BufFile`
  block-boundary fixture.

## Key result lines

- `test result: ok. 16 passed; 0 failed`
- `Finished dev profile` with both clippy command exit statuses 0.
- Focused PG18 result: `1 passed; 0 failed`.

## Scope note

This packet is correctness and implementation evidence. It contains no corpus,
suite config, `results.jsonl`, latency, recall, storage, or RSS result. The
10k/50k/100k and quality A/B condition remains open for a later measurement
packet driven only by `ecaz bench suite`.
