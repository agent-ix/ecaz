# Packet 003 — Task 48: soak harness RSS + slope gate + make wrapper

## Head

- Task bucket: `reviews/task-48/`
- Packet path: `reviews/task-48/003-soak-rss-slope/`
- Validation head SHA: `fd07ad1a8`
- Branch: `main`
- Surface under validation: `ecaz stress soak-quant-cache` CLI
  subcommand promoted from kickoff scaffold to leak-gate harness;
  `make soak` wrapper; `scripts/parse_humantime.sh` helper.

## What changed

| Path | Kind | Purpose |
|---|---|---|
| `crates/ecaz-cli/src/commands/stress/soak_quant_cache.rs` | code | + RSS sampler + slope check + tolerance arg + non-zero exit on leak |
| `Makefile` | recipe | `make soak DURATION=…` wrapper |
| `scripts/parse_humantime.sh` | helper | NNN/NNNs/NNNm/NNNh/NNNd → seconds |

## Artifacts

### soak-5s-with-rss-slope.log

- Command: `./target/debug/ecaz stress soak-quant-cache
  --duration-seconds 5 --workers 4 --shared-keys 4
  --private-keys-per-iter 2 --slope-tolerance-bytes-per-iter
  1000000`
- Timestamp: 2026-05-26
- Result: 10 iterations / 23,853 total ops / 5.14s wall.
  `slope_bytes_per_iter: 29491.2`, `slope_check_passed: true`.
  Per-iter `rss_bytes` ∈ `[17,252,352, 18,284,544]`.

### soak-tight-tolerance-fail.log

- Command: same harness with `--slope-tolerance-bytes-per-iter 1`
  on 3s wall.
- Result: `slope_bytes_per_iter: -32768.0`, `slope_check_passed:
  true` (negative slope is opposite-of-leak, gate semantics
  documented in request.md).

## Key result lines cited by request.md

- `iterations_completed: 10` / `mean_ops_per_sec: 4642.9` /
  `slope_check_passed: true` from soak-5s-with-rss-slope.log
- `slope_bytes_per_iter: 29491.2` (positive slope but well below
  1 MB tolerance; confirms the gate is computing, not erroring)
- macOS RSS values `17,252,352 → 18,284,544` confirm
  `libc::proc_pid_rusage` V2 path works.
- Negative `slope_bytes_per_iter: -32768.0` from the second log
  documents the `slope ≤ tol` (not `|slope| ≤ tol`) semantics.
