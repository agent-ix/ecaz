# Task: Batch Recall Smoke Test Inserts via COPY

Motivation: Review 218 item 9 flagged that the recall smoke test in
`tests/recall_integration.rs` inserts 500 corpus rows plus 25 query rows
via per-row `Spi::run("INSERT ... VALUES ...")` and takes ~163 s, which
dominates its wall clock. That is acceptable while the smoke stays
`#[ignore]`d and manual, but is the main thing blocking it from being
eligible to run in CI. Reviews 220/221/222 did not touch this. The fix is
mechanical and completely isolated from the A4 primary lane, so it is safe
coder-2 work to pick up in parallel.
Priority: batch 3
Status: ready

## Prompt

Rewrite the recall smoke test's corpus and query table seeding to use a
single `COPY ... FROM STDIN` stream per table instead of per-row
`INSERT ... VALUES`. The target runtime for the seeding portion of the
smoke is under 5 seconds on the scratch cluster.

### Step 1 — locate the current slow path

In `tests/recall_integration.rs`, find the smoke helper that currently
seeds the corpus and query tables. It will be near the other ignored
`recall_smoke_*` tests and visibly consists of a loop of
`Spi::run(&format!("INSERT INTO ... VALUES ({}, '{}')", ...))` calls. Read
the whole helper before touching anything — there is usually a shared
random-vector generator and a specific seed/order contract that the rest
of the smoke depends on, and the COPY rewrite must preserve the exact same
rows in the exact same order.

### Step 2 — mirror the loader's COPY literal format

`scripts/load_real_corpus.py` already uses the curly-brace `real[]` COPY
literal format: `{v0,v1,...}` with `repr(float(v))` per element (see
`_format_real_array_literal` near `scripts/load_real_corpus.py:305`). Use
the same format in the smoke — it is already proven against the canonical
corpus table schema. Per-element format: `repr(f32 as f64)` is fine; the
target column is `real[]`, and Postgres will accept any format Rust's
`{}` emits for `f32` values.

### Step 3 — use pgrx's COPY support

pgrx exposes `Spi::connect` and a `copy_in` path. Read how existing pgrx
tests use COPY — search the `pgrx` source under the vendored dep directory
for `copy` / `COPY FROM STDIN` to find the idiomatic pattern. If the
vendored pgrx version does not expose a stable `copy_in`, an acceptable
fallback is to build a single multi-row `INSERT ... VALUES (...), (...),
(...) ...` statement (all rows in one `Spi::run` call). Multi-row INSERT
is still orders of magnitude faster than per-row INSERT because the
overhead is per-statement, not per-row.

Prefer the COPY path if available. Document in the commit message which
path you took and why.

### Step 4 — preserve row order and content byte-for-byte

The existing smoke asserts "byte-identical reruns". Do not regenerate
vectors with a different RNG call order. Do not change the row ids. Do
not change the float-formatting precision. The goal is exclusively to
change the insert transport, not the data. If you are unsure whether your
change preserves bytes, run the smoke twice before and after your change
and diff the recall summary output — it must be unchanged to the last
digit.

### Step 5 — remove the old per-row path

Once the COPY path is working, delete the old per-row loop rather than
keeping both behind a feature flag. Leaving both creates drift; pick one
and commit to it.

### Step 6 — reduce or remove the smoke's wall clock budget comment

If there is a comment in the smoke saying "this takes ~N seconds" or
similar, update it to reflect the new budget. If the `#[ignore]` was
justified only by the long runtime, add a note next to the `#[ignore]`
explaining why it is still ignored (the real reason should be "requires
pg_test build and scratch cluster", not "takes too long").

## Design notes

- Do not touch any of the recall computation logic. The fix is entirely
  in the seeding helper.
- Do not introduce a new crate dependency. `pgrx` already provides
  everything you need; if it does not, fall back to multi-row `INSERT`
  rather than adding a Postgres client crate.
- Do not rewrite the real loader (`scripts/load_real_corpus.py`). Its
  COPY path already works and this task is only about the Rust smoke
  test.
- The target is "seeding drops from ~160s to <5s". If you hit that, stop
  — there is no need to chase further speedups on an ignored test.

## Out of scope

- Promoting the smoke to CI. That is a separate decision with its own
  review.
- Adding new assertions to the smoke beyond what is there today.
- Changing the fixture size. 500 + 25 is small enough to stay small; do
  not scale it up or down as part of this task.

## Validate

```bash
cargo clippy --all-targets --no-default-features --features 'pg17 pg_test' -- -D warnings
cargo test --features 'pg17 pg_test' --no-default-features recall_smoke -- --ignored --nocapture
```

Run the ignored smoke twice in a row and confirm:

1. Both runs complete in under 10 seconds wall clock (seeding + probes).
2. The recall summary printed by both runs is byte-identical.

Attach the wall-clock numbers (before and after) to the review packet.

Branch from current upstream main. Push branch for review.
