# Task 39 / 048 — SPIRE header.rs mutation campaign

## Goal

Third slice of the reviewer-prescribed SPIRE storage mutation cascade
(`reviews/task-39/044-helpers-expansion/feedback/2026-05-19-02-reviewer.md`).
Drive every mutation in `src/am/ec_spire/storage/header.rs` to
**0 missed / 0 timeouts**.

## Result

**35 mutations enumerated → 35 KILLED, 0 MISSED, 0 TIMEOUTS.**

No new tests required. The existing careful suite (cumulative from
packets 021/028/029/044/046/047) already discriminates every operator
swap and body replacement on this file. Triage map in `triage.md`.

## Code change

None. Only documentation and packet artifacts.

The methodology helper has been generalised in
`artifacts/run-spire-mutations.py` from the per-file scripts used in
046/047; future cascade packets reuse it directly.

## Validation

Artifacts under `reviews/task-39/048-spire-header-mutation/artifacts/`:

- `header-mutants-enumerated.txt` — full 35-mutation enumeration.
- `run-spire-mutations.py` — generic per-file verification helper.
- `manual-verification.log` — **35 KILLED, 0 MISSED, 0 PATCH-FAIL.**
- `post-verification-tests.log` — `cargo test --manifest-path
  hardening/careful/Cargo.toml --lib`: **534 passed, 0 failed**
  after every mutation reverted.

Source file byte-for-byte identical to pre-packet state.
