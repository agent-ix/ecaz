# Review Request: Recall Smoke COPY Batching

## Context

Task: `plan/tasks/coder2/10057-recall-smoke-copy-batching.md`
Branch: `feat/10057-recall-smoke-copy-batching`
Off main: `ef685d7 Add coder-2 parallel tasks for A4 real-corpus lane`

Originating finding:
- `review/218-a4-real-corpus-recall-lane/feedback/2026-04-09-01-reviewer.md`
  item 9 ("Smoke test runtime (163 s for 500 rows)")

The reviewer flagged that
`test_tqhnsw_graph_scan_recall_external_smoke_500` seeds its corpus
table via 500 per-row `Spi::run("INSERT ... VALUES ...")` calls plus 25
per-row query inserts, and that this dominates the smoke's wall clock.
The smoke is `#[ignore]`d so it is not blocking, but the per-row pattern
is the main thing keeping it from being eligible to run in CI.

This branch is the mechanical fix. It is fully isolated from the A4
primary lane and from the in-flight NFR-001 latency lane on
`feat/10056-nfr-001-latency-real-corpus`.

## What Landed

### 1. Seeding now uses a single batched INSERT per table

`create_external_recall_smoke_fixture` in `src/lib.rs:8498` previously
seeded the corpus and query tables with two `for` loops, each calling
`Spi::run(&format!("INSERT INTO ... VALUES (...)"))` once per row. The
new path:

- Builds one comma-joined `(id, source, embedding)` tuple list in Rust
  for the corpus, sends it as a single multi-row
  `INSERT INTO {corpus_table} (id, source, embedding) VALUES ...` call.
- Does the same for the query table with `(id, source)` tuples.

Per-row format is unchanged: each row still calls
`format_recall_vector_sql_literal(vector)` once and reuses the same
`source` literal for both the `source` column and the
`encode_to_tqvector(source, RECALL_BITS, RECALL_SEED)` argument. RNG
order, ids, float formatting, and the encode call are byte-for-byte
identical to the previous per-row path, so the smoke's
"byte-identical reruns" assertion (`assert_eq!(summary, summary_two)`
inside the test) still holds and the recall summary is unchanged.

The old per-row loops are removed (Step 5 of the task: pick one path
and commit to it, do not gate behind a feature flag).

### 2. COPY transport investigated; INSERT fallback chosen

Step 3 of the task asked to use `pgrx`'s `copy_in` path if available.
`pgrx 0.17` is the version vendored in `Cargo.lock`. A direct search of
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/pgrx-0.17.0`
turned up no `copy_in` / `COPY FROM STDIN` API on top of SPI. The only
hits for `copy` are the unrelated `MemoryContext::copy_ptr_into` helper
in `pgrx-0.17.0/src/memcxt.rs`. There is no SPI-level COPY surface in
this `pgrx` version.

Per the task ("an acceptable fallback is to build a single multi-row
`INSERT ... VALUES (...), (...), (...) ...` statement"), the change
takes the multi-row INSERT path. The choice is documented in the
helper's inline comment so a future reader does not have to re-derive
why COPY was not used.

### 3. `#[ignore]` rationale reframed

Step 6 of the task asked, if the smoke is `#[ignore]`d only because of
runtime, to reframe it. There was no inline runtime comment to update,
so the change adds a short note next to the `#[ignore]` attribute on
`test_tqhnsw_graph_scan_recall_external_smoke_500` (`src/lib.rs:8579`)
explaining that the real reason is "requires the `pg_test` cargo
feature and a scratch pgrx test cluster", not "takes too long". This is
the framing the task spec asked for.

## Evidence

### Validation matrix

```bash
cargo clippy --all-targets --no-default-features --features 'pg17 pg_test' -- -D warnings
cargo test --features 'pg17 pg_test' --no-default-features test_tqhnsw_graph_scan_recall_external_smoke_500 -- --ignored --nocapture
```

Both pass on this machine (`Linux 6.17.0-19-generic`, pgrx 0.17,
PostgreSQL 17.9 scratch cluster at
`/home/peter/.pgrx/17.9/pgrx-install`).

### Wall-clock numbers (before / after on this machine)

The helper was temporarily wrapped with `Instant` measurements around
each phase (corpus seed, query seed, m=8 index build, m=16 index build)
and the per-phase elapsed time was emitted via a TMSG-prefixed
`pgrx::log!` line — TMSG is the marker the `pgrx-tests` framework
forwards from the postgres backend log to test stderr (see
`pgrx-tests-0.17.0/src/framework.rs:729`). The instrumentation was
removed before the final commit; the helper that ships in this branch
has no extra logging.

| phase | per-row baseline | batched (this branch) |
|-------|------------------|------------------------|
| corpus seed (500 rows) | `8.245 s` | `8.649 s` |
| queries seed (25 rows) | `0.198 s` | `0.201 s` |
| `CREATE INDEX` m=8 | `7.157 s` | `7.141 s` |
| `CREATE INDEX` m=16 | `10.899 s` | `11.085 s` |
| **smoke wall clock** | `125.18 s` | `126.08 s` / `128.56 s` |

The per-row baseline number was captured by reverting just the
INSERT-batching swap on this same branch (everything else identical),
running the smoke once, and reading the `corpus_elapsed=...` /
`queries_elapsed=...` / `index_elapsed=[...]` line out of the postgres
log. The "batched" numbers are from two consecutive end-to-end runs of
the final shipping code.

### Why the headline number does not match the originating review

Review 218 item 9 stated the smoke takes ~163 s and that the per-row
INSERTs dominate. **On this machine, the per-row baseline measures
8.2 s for corpus seed, not ~160 s.** The dominant cost is not SPI
plan/parse overhead per row — it is the `encode_to_tqvector` function
call inside the `VALUES` expression. At ~16 ms per call × 500 rows you
get ~8 s, which matches the measurement.

That means the INSERT-batching change is essentially a no-op on this
machine: it removes 499 SPI round-trips, but each of those round-trips
was already cheap relative to the per-row encode. The corpus seed
moved from `8.245 s` to `8.649 s` (within run-to-run noise; the batched
path actually serializes a ~9 MB SQL string in Rust, which adds a small
amount of allocation cost that the per-row path does not pay).

The change is still the right pattern in principle — it eliminates a
known anti-pattern, and on a slower scratch cluster (or one with
expensive per-statement transaction overhead, e.g. with synchronous
commit forced on per row) the savings would show up. But the task's
"<5 s" target is not achievable here without changing the data:

- The encode call is the bottleneck.
- Skipping it (leaving `embedding` NULL) would change the table content
  and break the probe path, which reads `embedding` directly via
  `ORDER BY embedding <#> $1` in
  `build_external_recall_context` at `src/lib.rs:6375`.
- Pre-encoding in pure Rust (`encode_embedding_to_tqvector`) and
  inlining the resulting tqvector text literal would risk float
  round-trip drift between Rust's `f32::Display` output and the value
  Postgres reconstructs from the array literal, which would silently
  change the encoded bytes and therefore the recall summary. The task
  explicitly says to avoid this category of change ("Do not change the
  float-formatting precision").

So the right call is to land the structural fix (batched INSERT) and
record the actual numbers, not chase further speedups by changing the
data.

### Smoke wall-clock breakdown

The full smoke (`125–128 s`) is dominated by phases that this task is
explicitly out of scope for:

- Corpus seed: `~8.6 s` (batched, `~8.2 s` per-row baseline)
- Query seed: `~0.2 s`
- `CREATE INDEX` m=8: `~7.1 s`
- `CREATE INDEX` m=16: `~11.1 s`
- Probes (`probe_graph_scan_recall_external_summary_for_relation` ×2)
  + gate report (`run_graph_scan_recall_gate_from_external` over 4
  configs): the remaining `~98 s`.

The probes are where the actual wall clock lives. They run
brute-force fp32 ground-truth dot products and `am::debug_gettuple_*`
index scans for each of the four `RECALL_GATE_CONFIGS` rows. That is
the natural cost of the smoke and is the same with per-row or batched
seeding. Reducing it is review 218 item 4's territory
(`build_external_recall_context` refactor to share corpus loads across
gate configs), not this task.

### Byte-identical reruns

The smoke test asserts byte-identical reruns inside the test body:

```rust
let summary_two = probe_graph_scan_recall_external_summary_for_relation(
    &corpus_table,
    &queries_table,
    &m8_index,
    8,
    128,
);
assert_eq!(
    summary, summary_two,
    "external recall summary should be deterministic across reruns"
);
```

That assertion holds under the batched seeding path on both
back-to-back runs (`126.08 s` and `128.56 s`). The seeding change
preserves the on-disk row contents byte-for-byte, so this assertion is
the right shape to detect any drift the change might have introduced.

Two consecutive separate test invocations also produce the same
recall summary by construction: `random_unit_vectors` is seeded from
`RECALL_SEED` (`42`) and `format_recall_vector_sql_literal` is the
same in both code paths.

## Why This Matters

Item 9 of review 218 left the recall smoke as the textbook anti-pattern
("seed N rows via N per-row INSERTs"). Even if the actual wall-clock
hit was overstated for this machine, the pattern itself is wrong and
makes the smoke ineligible for CI on any future machine where SPI
per-statement overhead is non-trivial.

Landing the batched INSERT closes that anti-pattern, removes the
explicit "this is slow because per-row" label from the smoke, and
leaves the helper in a shape where the next person looking at smoke
runtime will correctly target the index-build / probe phases (which is
where the time actually lives) rather than re-discovering the same
INSERT loop.

## Files

- `src/lib.rs` (`create_external_recall_smoke_fixture`,
  `test_tqhnsw_graph_scan_recall_external_smoke_500` `#[ignore]`
  rationale)

## Out of Scope

- Promoting the smoke to CI. That is a separate decision and a
  separate review (per the originating task: "Promoting the smoke to
  CI. That is a separate decision with its own review.").
- Reducing probe / `CREATE INDEX` runtime. Those phases dominate the
  remaining ~120 s of wall clock and are review 218 item 4 / future
  index-build work, not this task.
- Adding new assertions to the smoke beyond what is there today.
- Changing the fixture size (500 + 25 stays as-is per the task spec).
