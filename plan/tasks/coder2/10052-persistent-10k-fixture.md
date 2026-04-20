# Task: Persistent 10K Fixture for A4 Iteration

Motivation: The A4 recall investigation (reviews 194-212) requires repeated runs
of a 10k-element recall gate on the live graph path. Each run rebuilds the index
from scratch, which takes ~11 minutes (dominated by hnsw_rs graph construction in
CREATE INDEX). Review 210 split fixture reset from gate report, but every test
invocation still rebuilds. This is the single biggest practical blocker for
coder-1's iteration speed.
Priority: batch 2
Status: done

## Prompt

Make the 10k A4 fixture persistent across test invocations so the gate report
can rerun in seconds without rebuilding.

The current fixture helpers are in `src/lib.rs` (search for
`ec_hnsw_graph_scan_recall_fixture_gate_reset` and
`ec_hnsw_graph_scan_recall_fixture_gate_report`). The reset function creates
tables and indexes from scratch. The report function reads from them.

Approach: Modify the reset function to check if the fixture tables and indexes
already exist before rebuilding. If they exist with the expected shape (correct
row count, correct index parameters), skip the rebuild and return early.

Detection logic:
- Check if the corpus table exists (pg_class lookup via SPI)
- Check if it has the expected row count
- Check if the indexes exist with expected reloptions (m, ef_construction)
- If all checks pass, log "fixture already exists, skipping rebuild" and return
- Otherwise, DROP IF EXISTS and rebuild from scratch

The pgrx test framework runs each test in a transaction that gets rolled back.
This means tables created inside a test function disappear after the test. To
make fixtures persist:

- The fixture tables should be created via SPI in a separate committed
  transaction, OR
- Use a setup function that runs outside the test transaction, OR
- Create the fixtures in a schema that persists (the `pgrx_tests` database
  survives across test runs — tables created with explicit COMMIT persist)

The simplest approach may be: in the reset function, check if the table exists
first. If it does, return immediately. The test framework creates a fresh
connection for each test, but the database state persists between runs of the
same pgrx test binary.

Important: pgrx `pg_test` functions run inside a transaction. You may need to use
`Spi::run` with explicit transaction control, or restructure so the fixture
creation happens in a helper that commits. Investigate how pgrx handles this —
look at how the existing fixture reset function works and whether its tables
survive across invocations.

If full persistence is too complex within pgrx's transaction model, an acceptable
fallback is: reduce the fixture size to something that builds in <60 seconds while
still being large enough to produce meaningful recall numbers. A 5k fixture might
build in half the time while still giving useful signal.

## Validate

```bash
cargo test
cargo pgrx test pg17
```

Branch from current upstream main. Push branch for review.

## Completion Notes

### What changed
Added a `gate_fixture_already_exists` helper function in `src/lib.rs` that checks
whether the gate fixture tables and indexes already exist with the expected shape
before rebuilding. Modified `reset_graph_scan_recall_gate_fixtures` to call this
check first and return early if the fixture is already present.

### Detection logic implemented
1. Check if `{prefix}_corpus` table exists via `pg_class` lookup (relkind = 'r')
2. Check row count matches `corpus_size`
3. For each m in [8, 16], check if `{prefix}_m{m}_idx` exists with correct
   `reloptions` (m and ef_construction values) via `pg_class` lookup
4. If all checks pass, retrieve block counts from existing indexes and return
5. Otherwise, fall through to the existing DROP + rebuild path

### Decisions
- Used `pgrx::log!` (not `eprintln!`) for the "skipping rebuild" message so it
  goes through PostgreSQL's standard logging
- Kept the detection conservative: any mismatch in table existence, row count,
  or index reloptions triggers a full rebuild
- Did not modify `reset_graph_scan_recall_fixture` (single-index variant) since
  the task scope only covers the gate fixture functions

### Validation
- `cargo test`: all tests pass (2 passed, 20 ignored)
- `cargo pgrx test pg17`: all tests pass (170+ passed including pg_test integration tests)
