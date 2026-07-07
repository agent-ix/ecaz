# Review request: Task 168 closeout

- Branch: `task-168-diskann-batched-beam` (off main `b891c3743`), all
  commits pushed.
- Evidence: packets 001–005 under `reviews/task-168/`; this packet's
  `artifacts/manifest.md` holds the canonical-sweep confirmation run and
  the cumulative summary.

## Exit criteria vs task file

1. **Phase 1 characterization packet** — packet 001 (rabitq 10/50/100k
   wall-time split, flush-width histogram, recall references, ranked slice
   list). DONE.
2. **Each landed slice A/B'd at 10/50/100k, recall floor held** — packet
   002 (batched-beam, W-sweep, W=4 default), packet 004 (alloc cleanups +
   hasher, four arms), recall equal-or-better at every cell in every landed
   arm. Packet 003 documents the shelved prefetch slice (measured loss,
   reverted byte-identical). DONE.
3. **Batched-beam landed as the shared primitive** —
   `scan.rs::greedy_descent_beam_with` (+ profiled twin), consumed via
   `vamana_scan_beam_with`; documented as the ec_distann FR-081 hop-round
   shape (Task 162 consumes it). DONE.
4. **docs/benchmarks.md refreshed** — Task 168 note in the `ec_diskann`
   section (rabitq default, beam GUC, local-Intel deltas); AWS-lane cells
   intentionally left for their next canonical run. DONE (this packet).
5. **clippy pg18 -D warnings clean; no new unsafe** — clean **at HEAD**,
   packet-local log `artifacts/clippy-pg18-head.log` (exit 0); zero new
   `unsafe` blocks in the task diff. Per-commit clippy runs happened during
   development but were not captured as durable logs, so the claim is
   narrowed to HEAD-only per reviewer feedback. DONE (as narrowed).

### Validation evidence (added per 2026-07-07-01 reviewer feedback)

| Log | Result |
|---|---|
| `artifacts/clippy-pg18-head.log` | exit 0, `-D warnings` clean at HEAD |
| `artifacts/cargo-test-ec-diskann-scan-head.log` | scan 33/33 + tuple 17/17 pass |
| `artifacts/cargo-pgrx-test-pg18-ec-diskann-head.log` | 212 passed, 2 failed (both non-regressions, next rows) |
| `artifacts/guc-flake-single-threaded-head.log` | the GUC failure passes single-threaded (parallel-FFI flake) |
| `artifacts/turboquant-counters-preexisting-on-specs-branch.log` | the quantizer counters failure reproduces on unmodified main-derived src (task-161 worktree) |

Also included: `StorageFormat::DEFAULT` PqFastScan → RaBitQ with pg_test
updates (packet 004).

### Feedback 2026-07-07-01 — resolution

1. P2 validation logs: added (table above); clippy claim narrowed to HEAD.
2. P2 missing task file: `plan/tasks/168-diskann-batched-beam-and-prefetch.md`
   checked out from `task-161-ec-distann-specs` onto this branch — the
   packet is now self-contained.
3. P3 whitespace: all `build-profile.log` files + `profile-summary.txt`
   stripped; `git diff --check origin/main..HEAD` is clean.

## Open items for the reviewer

- Packets 001–004 requests are open; this closeout rolls them up. Feedback
  on any packet lands in that packet's `feedback/`.
- Pre-existing (NOT task-168) failure to triage:
  `diskann_turboquant_prepared_prefilter_batch_scores_and_records_counters`
  fails identically on unmodified main-derived src on this host.
- 1m axis not run locally (not staged); the AWS lanes pick up the new
  defaults on their next canonical run.
- Merge to main only with explicit operator approval per repo convention.
