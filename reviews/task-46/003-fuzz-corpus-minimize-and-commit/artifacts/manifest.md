# Packet 003 — Task 46: fuzz corpus minimize + commit

## Head

- Task bucket: `reviews/task-46/`
- Packet path: `reviews/task-46/003-fuzz-corpus-minimize-and-commit/`
- Validation head SHA: `5d84cedc9` (corpus content commit, on top
  of `56783b5eb` policy/tooling commit)
- Branch: `main`
- Surface under validation: all 9 fuzz targets registered in
  `fuzz/Cargo.toml`; the corpora directories they read on start.
- Storage format / fixture: N/A — this packet *is* the fixture commit.
- Rerank mode / lane: N/A — corpus management, not a recall/latency
  benchmark.
- Surface isolation: cmin is single-process libFuzzer per target;
  no cross-target dependencies.

## What changed

Two commits:

- `56783b5eb` — `.gitignore` un-ignores `fuzz/corpus/` (keeping
  `/fuzz/target` and `/fuzz/artifacts` ignored); `Makefile` adds
  `make fuzz-corpus-minimize`.
- `5d84cedc9` — initial commit of minimized corpora across all 9
  registered targets (396 files / ~1.6 MB).

## Artifacts

### cmin-parse-text.log

- Command:
  `PATH=$HOME/.rustup/toolchains/nightly-aarch64-apple-darwin/bin:$PATH
   RUSTUP_TOOLCHAIN=nightly cargo fuzz cmin fuzz_parse_text`
- Timestamp: 2026-05-26
- Result:
  - `#520 DONE cov: 138 ft: 378 ...`
  - `MERGE-OUTER: 143 new files with 378 new features added; 138 new coverage edges`
- 520 → 143 files (3.6× reduction), coverage preserved.

### cmin-batch.log

- Command: bash loop over the other 8 targets, sequential cmin.
- Timestamp: 2026-05-26
- Result: clean exit on each target; cov/ft preservation lines
  retained per the per-target summary in request.md.

### corpus-sizes-post-cmin.log

- Command:
  `for d in fuzz/corpus/*/; do echo "$(ls "$d" | wc -l) $(du -sh "$d" | cut -f1) $d"; done`
- Timestamp: 2026-05-26 (immediately after cmin batch)
- Result: 396 files / 1.6 MB total across the 9 target subdirs.
  Pre-cmin baseline in comments for diff.

## Key result lines cited by request.md

- `Pre-cmin total: 884 files / ~3.3 MB` (corpus-sizes-post-cmin.log
  comment block)
- `Post-cmin: 396 files / ~1.6 MB` (corpus-sizes-post-cmin.log)
- `Reduction: 55% file count, 52% disk` (corpus-sizes-post-cmin.log)
- Per-target `MERGE-OUTER: N new files with M new features added;
  K new coverage edges` lines (cmin-parse-text.log + cmin-batch.log)

## Notes

- `cargo fuzz cmin` is deterministic for a given input corpus + target
  binary at a given head: re-running it on the committed corpus
  should be a no-op (every input is in the spanning set by
  construction).
- `make fuzz-corpus-minimize` is intentionally a thin loop over the
  per-target `cargo +nightly fuzz cmin` invocation — no
  re-implementation of cmin's spanning logic.
- The structured targets (`fuzz_parse_text_structured`,
  `fuzz_unpack_mse_structured`) retain their 2× / 1.7× coverage
  advantage over their raw siblings after cmin; the structural-fuzz
  density signal in packet 001/002 is not an artifact of corpus
  bloat.
