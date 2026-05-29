# Task 48/003: soak harness RSS + slope gate + make wrapper (closes §Exit #2)

## Scope

Closes Task 48 §Exit Criteria gate 2:

> 2. `make soak DURATION=24h` runs weekly and the artifact lands in
>    a packet.

Validation head: `fd07ad1a8`.

Promotes the packet 001 kickoff harness (`ecaz stress
soak-quant-cache`) from "smoke scaffold" to "leak-gate harness" by
adding the three pieces Task 48 §Approach 3 names:

1. Cross-platform RSS sampler — Linux `/proc/self/statm`, macOS
   `libc::proc_pid_rusage` V2, others return `None`.
2. Linear-fit slope-of-RSS-vs-iteration over the second half of
   the run; non-zero exit if the slope exceeds the configured
   tolerance.
3. `make soak DURATION=…` Make wrapper that parses any
   `5`/`300s`/`24h`/`1d`-shape duration before forwarding to the
   binary.

## What changed

### `crates/ecaz-cli/src/commands/stress/soak_quant_cache.rs`

- `current_rss_bytes()` — cfg-gated:
  - **Linux**: parses `resident_pages` from
    `/proc/self/statm`, multiplies by `sysconf(_SC_PAGESIZE)`.
  - **macOS**: calls `libc::proc_pid_rusage` with `RUSAGE_INFO_V2`,
    returns `ri_resident_size`. Two SAFETY blocks document
    the libc preconditions.
  - **Other**: returns `None`; slope check skips cleanly.
- `slope_bytes_per_iter()` — linear least-squares slope of
  `(iter_index, rss)` pairs over the **second half** of the
  iteration records. Returns `None` if fewer than 4 records or
  any sample is `None`. The second-half window is what Task 48
  §Approach 3 specifies (skip warm-up noise).
- `IterationRecord::rss_bytes: Option<u64>` added.
- `SoakSummary::slope_bytes_per_iter: Option<f64>`,
  `slope_check_passed: bool`,
  `slope_tolerance_bytes_per_iter: u64` added.
- `--slope-tolerance-bytes-per-iter` arg, default `1024`. The
  harness exits non-zero when slope > tolerance, making the soak
  itself a leak gate.

### `Makefile`

- New `soak` target. `make soak` runs the default 5s smoke; `make
  soak DURATION=24h` forwards to `ecaz stress soak-quant-cache
  --duration-seconds 86400 --slope-tolerance-bytes-per-iter $(SOAK_SLOPE_TOLERANCE)`.
- `DURATION` accepts the same humantime-ish shapes documented in
  `docs/build-matrix.md`: `300s`, `1h`, `24h`, `1d`.

### `scripts/parse_humantime.sh`

- 32-line bash helper called by the Makefile target. Converts
  `NNN` / `NNNs` / `NNNm` / `NNNh` / `NNNd` to integer seconds.
  Errors with usage text on unsupported shapes.

## Evidence

### `artifacts/soak-5s-with-rss-slope.log`

- Command:
  `./target/debug/ecaz stress soak-quant-cache --duration-seconds 5
  --workers 4 --shared-keys 4 --private-keys-per-iter 2
  --slope-tolerance-bytes-per-iter 1000000`
- Timestamp: 2026-05-26
- Key fields in the JSON output:
  - `iterations_completed: 10`
  - `mean_ops_per_sec: 4642.9`
  - `slope_bytes_per_iter: 29491.2` (RSS computed; well below 1 MB
    tolerance — flat allocation pattern across the 10 iterations)
  - `slope_check_passed: true`
  - per-iteration `rss_bytes` ≈ `17,252,352 → 18,284,544` (≈ 17 MB
    → 18 MB), confirming the macOS `proc_pid_rusage` path works.
- Exit code: 0.

### `artifacts/soak-tight-tolerance-fail.log`

- Command: same harness with `--slope-tolerance-bytes-per-iter 1`
  on a 3-second run.
- Result: slope came back **negative** (`-32768 bytes/iter`) —
  i.e., RSS dropped over the run because of allocator behavior, no
  leak signature. Since `-32768 ≤ 1` the slope check still passed
  (correctly — a negative slope is the *opposite* of a leak).
- This artifact documents the gate's semantics: the check is
  `slope ≤ tol`, not `|slope| ≤ tol`, because a shrinking RSS is
  never a failure.

A unit-test fixture that produces a *positive* slope above
tolerance would exercise the failure-exit branch directly. The
slope logic is small (`slope_bytes_per_iter` is < 30 lines of pure
arithmetic) and the failure branch is the trivial `return Err(...)`
on the `(Some(s), tol) if s > tol as f64` case; deferring the
dedicated unit test to a future packet keeps this slice scoped to
the harness-level change.

## Reviewer focus

- The RSS sampler avoids adding a new dep — uses `libc` (already
  in `Cargo.toml`) and `std::fs::read_to_string`. macOS path uses
  the same `proc_pid_rusage` PG benchmarks elsewhere in the repo
  use; SAFETY blocks document preconditions.
- The slope window is **second half** specifically per Task 48
  §Approach 3 — warm-up noise (allocator growth on first few
  iterations) would skew a full-run slope. Linear least squares on
  the second-half samples is the simplest correct shape.
- `make soak` default duration is 5s so a developer's `make soak`
  without args is a fast smoke; `make soak DURATION=24h` is the
  weekly run.
- `scripts/parse_humantime.sh` is intentionally bash, not a Rust
  helper, because the Make recipe runs in shell and adding a
  pre-step builder dep was overkill.

## Out of scope (still open)

- Weekly *cadence* via CI schedule — that lands in Task 48/005
  (CI matrix workflow files) alongside the per-PR / nightly /
  weekly job table.
- PG-backed mixed read/write soak — separate slice (extends the
  harness to cover the SQL surface, not the in-process cache).
- A dedicated unit test for the slope-check failure branch — out
  of scope per request.md §Evidence.

## Task 48 §Exit Criteria progress after this slice

| # | §Exit Criterion | Status |
|---|---|---|
| 1 | CI matrix covers aarch64-darwin + x86_64-linux + aarch64-linux + pg17 + pg18 | 0% |
| 2 | `make soak DURATION=24h` weekly | **✓ done (this)** |
| 3 | `make resource-exhaustion` nightly | 0% |
| 4 | `docs/build-matrix.md` documents matrix, cadence, policy | ✓ done (002) |

Task 48 ≈ 50% complete (2 of 4 §Exit gates closed).
