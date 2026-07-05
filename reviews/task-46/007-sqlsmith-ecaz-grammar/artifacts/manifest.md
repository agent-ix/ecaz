# Packet 007 — Task 46: ECAZ-grammar SQLsmith

## Head

- Task bucket: `reviews/task-46/`
- Packet path: `reviews/task-46/007-sqlsmith-ecaz-grammar/`
- Validation head SHA: post-commit landing this slice's code +
  workflow.
- Branch: `main`
- Surface under validation: new crate `crates/ecaz-sqlgen/`,
  committed seed corpus `fixtures/sqlsmith-ecaz/`, Makefile
  `sqlsmith-ecaz` target, nightly GitHub Actions workflow
  `sqlsmith-ecaz-nightly.yml`.

## What changed

| Path | Kind |
|---|---|
| `crates/ecaz-sqlgen/Cargo.toml` | new manifest |
| `crates/ecaz-sqlgen/src/lib.rs` | new — five generation templates |
| `crates/ecaz-sqlgen/src/main.rs` | new — generate + execute CLI |
| `fixtures/sqlsmith-ecaz/seed-42-count-32.sql` | new — committed seed corpus (85 lines) |
| `Makefile` | new `sqlsmith-ecaz` recipe |
| `.github/workflows/sqlsmith-ecaz-nightly.yml` | new workflow (cron 04:37 UTC) |

## Artifacts

### unit-tests.log

- Command: `cargo test -p ecaz-sqlgen --lib`
- Result: `3 passed; 0 failed; 0 ignored; 0 measured`. Exit 0.

### seed-42-count-32-snapshot.sql

- Mirror of `fixtures/sqlsmith-ecaz/seed-42-count-32.sql` for the
  packet. 85 lines, 32 statement groups.

## Determinism contract

Re-generating the committed seed corpus must be a no-op:

```sh
./target/release/ecaz-sqlgen generate \
    --seed 42 --count 32 \
    --out fixtures/sqlsmith-ecaz/seed-42-count-32.sql
git diff fixtures/sqlsmith-ecaz/
```

The diff is empty by construction (ChaCha8Rng + fixed seed +
deterministic template draws). If the diff is non-empty after a
template change, the slice that introduced that change must
commit the regenerated corpus alongside.

## Task 46 progress after this packet

**Task 46: 5 of 5 §Exit gates closed (100%).**

All five Task 46 §Exit Criteria now met:

| # | Criterion | Closed by |
|---|---|---|
| 1 | structured-input → Arbitrary | 005 |
| 2 | `make sqlsmith-ecaz` nightly + seed corpus | this (007) |
| 3 | Honggfuzz + AFL+ weekly + cross-pollinate | 006 |
| 4 | `fuzz/corpus/` minimized + committed | 003 |
| 5 | `docs/hardening.md` engine matrix | 004 |
