# Task 43: Miri / cargo-careful Depth Expansion

Status: **proposed** — pushes the Miri and cargo-careful lanes from Task 34
beyond seed coverage into systematic, aliasing-strict, multi-seed runs over
every pure-Rust subsystem ECAZ has.

## Scope

Three depth-improvements to existing lanes:

1. **Tree Borrows** aliasing model (`-Zmiri-tree-borrows`) in addition to (or
   instead of) the default Stacked Borrows. Tree Borrows accepts more valid
   programs and is the direction the Miri team is moving; running both
   surfaces the largest set of UB.
2. **Many-seeds interleavings** (`-Zmiri-many-seeds`) to explore more thread
   schedules in the Miri data-race detector.
3. **Coverage expansion** beyond the current ~7 modules to every pure-Rust
   surface, with the lifted-module pattern from Task 40 used to bring
   currently-pgrx-tangled code into the Miri-able set.

## Why

Task 34 ran Miri over 19 `miri_` tests in 7 source files. That covers the
storage page, DiskANN metadata, SPIRE leaf V2 metadata, and one quant path —
real value, but a fraction of the pure-Rust footprint:

- The SPIRE coordinator state machine: not in Miri.
- The DiskANN graph traversal helpers: not in Miri.
- The HNSW graph traversal helpers: not in Miri.
- Top-k merge / candidate priority queue logic: not in Miri.
- The vacuum dead-tuple collection helpers: not in Miri.
- The remote payload parser (SPIRE): not in Miri.

Tree Borrows finds bugs that Stacked Borrows misses (and vice versa). The
many-seeds flag is the only way Miri exercises concurrent code at any
depth. `cargo-careful` runs the same tests under a debug-assertion-heavy
nightly stdlib; ECAZ runs it over only one harness today.

## Approach

1. **Tree Borrows lane.** Add a parallel run:

   ```sh
   MIRIFLAGS="-Zmiri-tree-borrows" cargo miri test --lib -- miri_
   ```

   Triaging differences between Stacked Borrows and Tree Borrows results goes
   into the inaugural packet. If both agree, pick one as the default and
   keep the other as a periodic audit.

2. **Many-seeds lane.** `MIRIFLAGS="-Zmiri-many-seeds=128"` over any `miri_`
   test that spawns threads or uses atomics. Default seeds is small; 128 is
   the documented sweet spot.

3. **Disable isolation where needed.** Tests that touch the clock or temp
   files run with `-Zmiri-disable-isolation`. Mark these explicitly so the
   isolation guarantee for everything else is preserved.

4. **Subsystem expansion.** New `miri_` tests for:
   - SPIRE coordinator state-machine transitions (lift via Task 40),
   - top-k candidate merge with bounded inputs,
   - DiskANN graph traversal with a fixture graph (small enough for Miri),
   - HNSW graph traversal with a fixture graph,
   - remote payload parser with adversarial inputs (also exercised by fuzz),
   - vacuum dead-tuple collection,
   - serialization / layout helpers for every on-disk type.

5. **cargo-careful expansion.** Where careful runs the pure-Rust harness
   today, extend it to cover the same `miri_` tests where they don't depend
   on pgrx callbacks. Careful is faster than Miri and catches different
   classes of bug (debug-assertion stdlib, more checks).

6. **Make lanes:**
   - `make miri` (existing) — default Stacked Borrows.
   - `make miri-tree` — Tree Borrows pass.
   - `make miri-many-seeds` — many-seeds pass over threaded tests.
   - `make miri-full` — runs all three.
   - `make careful` (existing) — extended to wider test set.

## Validation

- `make miri-tree` runs clean (or differences from Stacked Borrows are
  triaged into review packets).
- `make miri-many-seeds` finds no new data races; runs complete within the
  seed budget.
- Each newly added `miri_` test is genuinely Miri-able: passes locally and
  fails predictably if an obvious bug is injected (e.g., unsynchronized
  shared atomic without `Ordering::SeqCst`).
- `make careful` covers ≥ 80% of pure-Rust unit tests.

## Exit Criteria

- Both Stacked Borrows and Tree Borrows runs are part of
  `hardening-nightly-local`.
- Many-seeds run included where any threaded `miri_` test exists.
- Pure-Rust Miri coverage spans all major subsystems (SPIRE coordinator,
  DiskANN graph, HNSW graph, top-k merge, remote parser, vacuum,
  serialization).
- `docs/hardening.md` updates the Miri/careful section with the new flags,
  cadence, and triage process.

## Dependencies

- Task 40 (concurrency model checking) lifts the modules needed to bring
  SPIRE coordinator code into Miri.
- Independent of Tasks 36–39, 41–42.
- Cheap wins; should land early as it strengthens an already-existing lane.
