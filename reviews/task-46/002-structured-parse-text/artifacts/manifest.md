# Packet 002 — Task 46: structured parse_text target

## Head

- Task bucket: `reviews/task-46/`
- Packet path: `reviews/task-46/002-structured-parse-text/`
- Validation head SHA: `2bfcca062` (Task 46/002 code commit, on
  top of `e5cb093e8` Task 48/001 coder reply)
- Branch: `main`
- Surface under validation: `fuzz/src/lib.rs:58` `parse_text` —
  the canonical-format `[dim=…,bits=…,seed=…,gamma=…]:hex` parser
  used by the bench/CLI binary fixtures and the existing
  `fuzz_parse_text` raw-byte target.
- Storage format / fixture: N/A — fuzz target generates its input
  via `arbitrary::Arbitrary` and the matched raw baseline uses the
  35-file warm seed corpus that ships with `fuzz/corpus/fuzz_parse_text`.
- Rerank mode / lane: N/A — coverage + round-trip property
  assertion, not a recall/latency benchmark.
- Surface isolation: single-process libFuzzer binary, no PG backend.

## What changed

| Path | Kind | Purpose |
|---|---|---|
| `fuzz/Cargo.toml` | manifest | registers `fuzz_parse_text_structured` |
| `fuzz/src/lib.rs` | re-exports | promotes `DEFAULT_QUANT_*` to `pub`; re-exports `payload_len` via `bench_api` |
| `fuzz/fuzz_targets/parse_text_structured.rs` | target | new structured round-trip target |

## Artifacts

### fuzz-parse-text-structured-10s.log

- Command:
  `PATH=$HOME/.rustup/toolchains/nightly-aarch64-apple-darwin/bin:$PATH
   RUSTUP_TOOLCHAIN=nightly cargo fuzz run fuzz_parse_text_structured
   -- -max_total_time=10 -print_final_stats=1`
- Timestamp: 2026-05-25
- Key result lines (cited by request.md):
  - `#2340875 DONE   cov: 253 ft: 635 corp: 79/11693b lim: 4096 exec/s: 212806`
  - `stat::number_of_executed_units: 2340875`
  - `stat::new_units_added:          280`
  - `Done 2340875 runs in 11 second(s)`
- Result: clean exit (0), no crashes — the round-trip property
  assertion held for every input the fuzzer produced in the 10s
  window.

### fuzz-parse-text-raw-baseline-10s.log

- Command:
  `PATH=$HOME/.rustup/toolchains/nightly-aarch64-apple-darwin/bin:$PATH
   RUSTUP_TOOLCHAIN=nightly cargo fuzz run fuzz_parse_text
   -- -max_total_time=10 -print_final_stats=1`
- Timestamp: 2026-05-25 (immediately after the structured run)
- Key result lines (cited by request.md):
  - `#7033219 DONE   cov: 127 ft: 367 corp: 141/10005b lim: 4096 exec/s: 639383`
  - `stat::number_of_executed_units: 7033219`
  - `stat::new_units_added:          453`
  - `Done 7033219 runs in 11 second(s)`
- Result: clean exit (0), no crashes; warm 35-file seed corpus.

## Comparison cited by request.md

| metric | raw | structured | ratio |
|---|---:|---:|---:|
| cov  | 127 | 253 | 1.99× |
| ft   | 367 | 635 | 1.73× |
| corp | 141 | 79  | 0.56× |
| cov/corp | 0.90 | 3.20 | 3.55× |

Absolute coverage ratio is under the Task 46 §Validation ≥ 5×
threshold; coverage **density** (per kept corpus entry) is 3.55×.
Request.md documents the asymmetry honestly: `parse_text` has 7
distinct error branches, the raw target reaches those by accident
on rejection paths, the structured target stays on the success
path by construction. The structured target's distinctive
contribution is the round-trip property assertion, which the raw
target cannot make.

## Notes

- The structured target's exec/sec (213k) is ~1/3 of the raw
  target's (639k) because each iteration pays for an Arbitrary
  draw + an f32 round-trip + a hex-encode + the full parse path,
  versus the raw target's typical "reject at first gate" exit.
- The structured target's corpus file count (79) is smaller than
  the raw target's (141) but covers 2× as many edges — the
  structural-density signal Task 46 §Why predicts.
- `fuzz/corpus/fuzz_parse_text_structured/` is populated by the run
  but stays gitignored per `.gitignore:4`. Corpus minimize + commit
  remains a tracked follow-up from packet 001.
