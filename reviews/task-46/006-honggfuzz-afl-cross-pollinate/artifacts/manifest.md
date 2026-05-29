# Packet 006 — Task 46: Honggfuzz + AFL+ + cross-pollinate

## Head

- Task bucket: `reviews/task-46/`
- Packet path: `reviews/task-46/006-honggfuzz-afl-cross-pollinate/`
- Validation head SHA: `ea0cb5b76`
- Branch: `main`
- Surface under validation: `scripts/hardening.sh` (three new
  cases: `fuzz-afl`, `fuzz-honggfuzz`, `fuzz-cross-pollinate`),
  `Makefile` (three new top-level targets), and
  `.github/workflows/fuzz-cross-pollinate-weekly.yml` (weekly CI
  cadence).

## What changed

| Path | Kind | Purpose |
|---|---|---|
| `scripts/hardening.sh` | new cases | `fuzz-afl`, `fuzz-honggfuzz`, `fuzz-cross-pollinate` |
| `Makefile` | new recipes | wrappers for the three cases |
| `.github/workflows/fuzz-cross-pollinate-weekly.yml` | new workflow | Sundays 03:00 UTC schedule |

## Artifacts

This packet ships no first-run output — `fuzz-cross-pollinate` is
the *weekly* lane; the first run happens on the next Sunday after
merge. `bash -n scripts/hardening.sh` exits 0 (syntax check); the
three new Make targets parse cleanly (`make -n …` returned the
expected `bash scripts/hardening.sh …` lines).

## Cadence

| Mode | Trigger | Engines | Budget |
|---|---|---|---|
| `make fuzz-cross-pollinate` (local) | manual | libFuzzer + (AFL+ if installed) + (Honggfuzz if installed) | `FUZZ_SECONDS` (default 60s) per engine per target |
| `make fuzz-afl` (local) | manual | AFL+ only | build-only (libFuzzer cmin still pays for runtime) |
| `make fuzz-honggfuzz` (local) | manual | Honggfuzz only | `FUZZ_SECONDS` (default 30s) |
| CI weekly | `cron: "0 3 * * 0"` | all three | 120s per engine per target |

## Task 46 progress

4 of 5 §Exit gates closed (#1, #3, #4, #5). #2 (SQLsmith
ECAZ-grammar) remains.
