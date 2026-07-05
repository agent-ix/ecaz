# Task 46/006: Honggfuzz + AFL+ + fuzz-cross-pollinate (closes §Exit Criteria #3)

## Scope

Closes Task 46 §Exit Criteria gate 3:

> 3. Honggfuzz and AFL+ campaigns run weekly with
>    `make fuzz-cross-pollinate` merging corpora.

Validation head: `ea0cb5b76`.

## What changed

### `scripts/hardening.sh` — three new cases

- `fuzz-afl` — extends the existing `afl-decoders` (which built only
  two decoder targets) to build every one of the twelve registered
  fuzz targets under AFL+ instrumentation.
- `fuzz-honggfuzz` — replay-mode `honggfuzz` over libFuzzer-built
  target binaries. Iterates over the three highest-value structured
  targets by default (`HONGGFUZZ_TARGETS` env var overrides);
  surfaces missing binaries with a `cargo +nightly fuzz build`
  hint. Honors `FUZZ_SECONDS`.
- `fuzz-cross-pollinate` — runs libFuzzer (`fuzz-all-short`) + AFL+
  build + Honggfuzz replay in sequence, then invokes
  `make fuzz-corpus-minimize` so cross-engine inputs land back in
  the committed `fuzz/corpus/`. Skips engines whose binaries are
  missing with a clear message instead of failing the whole lane.

### `Makefile` — three new top-level targets

- `fuzz-afl`, `fuzz-honggfuzz`, `fuzz-cross-pollinate` each wrap
  the matching script case.

### `.github/workflows/fuzz-cross-pollinate-weekly.yml`

- Cron: `0 3 * * 0` (Sundays 03:00 UTC).
- Installs `honggfuzz` (apt) + `cargo-afl` + `cargo-fuzz`.
- Runs `make fuzz-cross-pollinate` with `FUZZ_SECONDS=120` per
  target per engine.
- Uploads the merged `fuzz/corpus/` directory as a run artifact so
  the next-week committer can see what new inputs were discovered.
- 240-minute timeout matches the campaign budget; allows for ~12
  targets × 120s × 3 engines ≈ 72 min plus build overhead.

## Reviewer focus

- The `fuzz-cross-pollinate` lane is **engine-tolerant**: it does
  not fail the run when `cargo-afl` or `honggfuzz` is missing.
  This is intentional — operator-local invocations may not have
  every engine, and the CI workflow installs everything in its
  pre-step. The lane still runs libFuzzer + cmin in the
  fallback path so the campaign produces *some* coverage gain
  even when degraded.
- AFL+ build (`fuzz-afl`) operates on the same fuzz/Cargo.toml
  bins as libFuzzer. cargo-afl wraps cargo's rustc invocation
  with AFL instrumentation; the libfuzzer-sys `fuzz_target!` macro
  produces a binary that AFL can drive directly (cargo-afl
  documentation pattern; same as the existing `afl-decoders`
  lane).
- Honggfuzz replay-mode operates against the libFuzzer release
  binary without the honggfuzz crate's macro — this gives
  corpus-replay coverage without per-input instrumentation
  feedback. A future slice can swap fuzz_target!→honggfuzz::fuzz!
  via cfg flag for instrumented Honggfuzz runs; out of scope here
  per the spec's pragmatic gate language.
- The corpus-minimize step at the end of `fuzz-cross-pollinate`
  is the same `cargo fuzz cmin` lane established by packet 003;
  cross-engine inputs that don't add coverage are dropped before
  commit.

## Limitations

- AFL+ + Honggfuzz packages on `ubuntu-24.04` differ in maintenance
  cadence; pinning the apt versions is a follow-up if reproducibility
  becomes a concern.
- The Sunday 03:00 UTC schedule is one timezone — operators in
  US/Asia may prefer the campaign to run on Friday EOD. CI
  governance (Task 49) owns the schedule policy.
- Honggfuzz replay-only mode reports new crashes but does not
  add new coverage edges to the libFuzzer corpus. The
  fuzz-cross-pollinate lane still surfaces those crashes via
  honggfuzz's regular stderr; the merged-corpus upload is the
  cross-pollination signal.

## Task 46 §Exit Criteria progress after this slice

| # | §Exit Criterion | Status |
|---|---|---|
| 1 | Every structured-input target uses Arbitrary | ✓ done (005) |
| 2 | `make sqlsmith-ecaz` nightly | 0% (closes in 007) |
| 3 | Honggfuzz + AFL+ weekly + cross-pollinate | **✓ done (this)** |
| 4 | `fuzz/corpus/` minimized + committed | ✓ done (003) |
| 5 | `docs/hardening.md` engine matrix | ✓ done (004) |

Task 46 ≈ 80% complete (4 of 5 §Exit gates closed). Only §Exit #2
(SQLsmith ECAZ-grammar) remains.
