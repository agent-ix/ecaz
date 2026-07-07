# Manifest — Task 162 packet 001 (M0 scaffold + build + local scan)

- Head SHA under review: `6e8e58572` (branch `task-162-ec-distann-m0`)
- Commit chain in scope: `29e88ebd4` (slice 1 scaffold), `7ba55012b`
  (slice 2 record + identity), `30ad585c7` (slice 3 build), `644122ffe`
  (slice 4 head index + FR-081 loop + frozen seam), `6e8e58572` (TC-037
  order/determinism additions).
- Task bucket: `reviews/task-162/001-m0-scaffold-build-scan/`
- Fixture: pg_test in-tree fixtures only (8 unit-norm dim-4 vectors);
  no corpus load in this packet. Bench evidence lands in packet 002+.
- Isolation: each pg_test creates its own table + index (one index per
  table).

## Artifacts

| File | Command | Key result | Timestamp |
|------|---------|-----------|-----------|
| `clippy-pg18.log` | `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` | `Finished 'dev' profile` — zero warnings | 2026-07-07 |
| `pg18-ec-distann-tests.log` | `cargo pgrx test pg18 ec_distann` | `test result: ok. 42 passed; 0 failed` | 2026-07-07 |

Both runs executed in worktree `~/dev/ecaz-task162` at head `6e8e58572`
on the Intel desktop (PG18.3 pgrx test harness).
