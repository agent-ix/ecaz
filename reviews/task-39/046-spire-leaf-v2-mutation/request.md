# Task 39 / 046 — SPIRE leaf_v2.rs mutation campaign

## Goal

First slice of the reviewer-prescribed SPIRE storage mutation cascade
(`reviews/task-39/044-helpers-expansion/feedback/2026-05-19-02-reviewer.md`).
Drive every mutation in `src/am/ec_spire/storage/leaf_v2.rs` to
**0 missed / 0 timeouts**, using the existing careful test suite from
packets 029 + 044 as the killing-test set.

## Result

**14 mutations enumerated, 14 KILLED, 0 MISSED, 0 TIMEOUTS.**

No new tests required. The 7 `miri_leaf_v2_validate_rejects_*` tests
(packet 044) plus `miri_leaf_v2_assignment_rows_round_trips_segments_back_to_rows`
(packet 044) and `miri_leaf_v2_meta_*` tests (packet 029) already
discriminate every operator swap and body replacement on `leaf_v2.rs`.

See `triage.md` for the per-mutation map and `artifacts/manual-verification.log`
for the apply/test/revert run.

## Important: reviewer's cargo-mutants invocation does not work

The reviewer's prescribed command
(`cargo mutants --package ecaz-careful-hardening --file hardening/careful/src/../../../src/am/ec_spire/storage/leaf_v2.rs ...`)
returns `Found 0 mutants to test  WARN No mutants found under the
active filters`. The cause: `src/am/ec_spire/storage.rs` and the careful
crate's `pub mod storage` block both mount SPIRE storage children via
`include!`, not via `mod` declarations. `cargo mutants` walks
`mod`/`pub mod` and ignores `include!` content. Packet 021's careful
harness made the SPIRE storage codec **lines** covered by llvm-cov
(which does track include sites), but never made them visible to
cargo-mutants' module discovery.

Two paths forward for the remaining 12 SPIRE storage files plus
`ec_spire/page.rs`:

1. **Per-file manual verification.** What this packet uses. Enumerate
   via `cargo mutants --Zmutate-file <absolute path> --list`, apply
   each mutation transiently to the production source, run the
   careful suite with a focused filter, record pass/fail, revert from
   a backup. ~30 s per file once the helper scripts are reused.
2. **Restructure storage.rs from `include!` to `mod`.** Either in
   production or just in the careful crate. Smaller refactor in the
   careful crate (production stays untouched), but every child file
   still needs `use super::*;` because they currently rely on the
   flat-namespace scope. After that refactor, cargo-mutants
   discovers all children and the reviewer's exact invocation works.

This packet flags path (1) as the next-12-packets default unless the
reviewer authorises path (2). The cost difference: path (1) is ~6
hours of mechanical per-file verification across the 13 files; path
(2) is a one-time ~30 minute careful-crate restructure that then
enables the reviewer's automated invocation for every subsequent
packet.

## Validation

Artifacts under `reviews/task-39/046-spire-leaf-v2-mutation/artifacts/`:

- `leaf-v2-mutants-enumerated.txt` — full mutation list (14 items).
- `initial-mutants-run.log` — proof that the reviewer's
  cargo-mutants invocation finds nothing under `include!`.
- `manual-verification.log` — per-mutation apply + careful test +
  revert sequence with KILLED/MISSED verdicts (14 KILLED / 0 MISSED /
  0 TIMEOUTS).
- `post-verification-tests.log` — `cargo test --manifest-path
  hardening/careful/Cargo.toml --lib`: **529 passed, 0 failed**
  after every mutation reverted.

No production code change. The leaf_v2.rs source file is byte-for-byte
identical to its pre-packet state (verified by
`diff /tmp/leaf_v2_original.rs src/am/ec_spire/storage/leaf_v2.rs`).

## Reviewer Direction

- Decide path (1) vs path (2) for the remaining 12 SPIRE storage
  files plus `ec_spire/page.rs`. Path (2) unblocks the reviewer's
  exact prescribed automation for every file at the cost of a
  careful-crate restructure.
- Confirm the manual verification format is acceptable as the
  "drive to 0 missed" evidence for files that path (1) covers.
