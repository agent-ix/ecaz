# Review request — Task 105 packet 007: resolution of the 002/003/006 blockers

- Task: `plan/tasks/105-production-optimization-full-scale-sweep.md`
- Packet: `reviews/task-105/007-blocker-resolution/`
- Coder: Task 99/102/103 author lane
- Date: 2026-06-13
- Kind: doc/config-only resolution (no `src/**` change, no rerun, no AWS)

## Summary

Resolves the three open blocking findings from the 2026-06-13-01
reviewer pass on packets 002, 003, and 006. All fixes are doc/config
amendments to the existing packets; the underlying measured data was
already present and is unchanged. Landed in commit `083dcdbd6`.

## What changed, per blocker

1. **003 — missing `artifacts/manifest.md` (blocking).** Added
   `reviews/task-105/003-local-10k-config-smoke/artifacts/manifest.md`
   with the full manifest contract (head/build identity, lane, fixture,
   storage surface, command, config sha256, timestamp, cited result
   lines). Reverified from the packet's own artifacts: **71/71** steps
   succeeded, **285** results rows, **28/28** on/off recall pairs
   byte-equal (0 mismatches, recomputed), ISA `avx2`/`scalar` only.
   Pure bookkeeping — no rerun.

2. **002 — IVF off-arm mislabeled as a kernel A/B (blocking).**
   Relabeled, not re-measured. The off arm omits
   `--ivf-scratch-soa-batch-decode`; the suite runner treats the
   `False` value the same as absent, so post-default-flip both arms run
   batch decode ON. The generator comment (`gen_t105_sweep.py`) and
   `request.md` now state the IVF on/off pairs are **same-config
   stability pairs**, not a differential, with the IVF kernel A/B
   delegated to Task 99's pre-flip 100k run. A true non-100k IVF
   differential would need a `suite.rs` `False`→explicit-`off` fix plus
   a snapshot-restore rerun — out of scope for this closeout and noted
   as such.

3. **006 — 100k matrix cells / AC2 over-claim (blocking).** Resolved by
   **narrowing the claim** (the reviewer's accepted alternative), and
   re-syncing the published tables:
   - Re-embedded the canonical `gen_matrix.py` output into
     `full-scale-matrix.md`. The embedded block had been hand-trimmed
     (Intel 100k recall/ISA columns dropped, storage re-sorted), which
     both violated the "generated — do not hand-edit" contract and hid
     that the Intel 100k cells are empty. They now show `—` honestly.
   - Rewrote the AC2 mapping and the 100k "How to read" bullet: the
     100k full on/off matrix is **delegated to Task 99 by Phase 2
     design** (008 G4 / 009 Intel), and the `—` 100k cells are
     intentional delegations, not gaps. The packet no longer claims the
     100k on/off cells as collected within Task 105.

## Correction to my own 006 seq-2 note (transparency)

My earlier reviewer follow-up (`006/.../2026-06-13-02-reviewer.md`)
claimed the full 100k on/off matrices for **both** lanes could simply
be ingested from Task 99 008/009. During implementation I verified that
is **wrong for G4**: the Task 99 G4 *default* profile-run predates the
Phase 1 NEON-first flip and dispatches SVE-eligible families through
`sve2` (verified: `diskann-pqfs-grouped-pq`, `diskann-tq`,
`hnsw-tq-full_lut`, `ivf-pq_fastscan`, `ivf-turboquant`,
`spire-turboquant`, qjl families). Importing it would inject `sve2`
cells into a matrix that claims zero foreign-ISA, and it is not
lane-equivalent to post-flip code. The Task 99 **NEON-capped** subset
that *is* post-flip-equivalent is kernel-on only, so a clean full on/off
G4 100k dataset under NEON dispatch was never collected — which is
exactly why Phase 2 specified only a G4 100k *confirmation* column.
Task 99's 100k campaign also swept an extra `ef_search=32`/`nprobe=8`
point Task 105 dropped. For these reasons the correct resolution is
delegation + honest markers, not ingestion. (Intel dispatch is
unchanged by the flip, so 009 is post-flip-equivalent for Intel and is
cited as the canonical Intel 100k source.)

## Review asks

1. Confirm the narrowed AC2 framing is acceptable (vs. requiring an
   ingest that, for G4, would be non-lane-equivalent).
2. Confirm the 003 manifest meets the packet contract.
3. Confirm the 002 relabel is the right call vs. a fresh IVF off-arm
   rerun.

## Open item for closure (not resolved here)

AC5 teardown: the 2026-06-12-01 G4 feedback observed the G4 snapshot
still `pending` and the stack not yet destroyed. Packet 006 asserts
both stacks destroyed 2026-06-12 with snapshots retained. This should
be confirmed against live AWS state before the task is closed/merged.
