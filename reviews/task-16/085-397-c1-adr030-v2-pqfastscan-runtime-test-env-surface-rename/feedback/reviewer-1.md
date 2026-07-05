## Feedback: PqFastScan Runtime Test Env Surface Rename

Read the test-side env-name switches in `src/lib.rs`.

### What's right

- **Regression surface now demonstrates canonical env usage.**
  Before this, the tests themselves were advertising the legacy
  `TQVECTOR_EXPERIMENTAL_ADR030_V2_*` names as the normal way to
  tune scans. That was exactly backward for a landing branch.
- **Fallback code untouched.** Legacy alias support in
  `scan.rs` is still tested indirectly (scripts, backcompat
  callers), which is the right split: canonical is the happy path,
  legacy is the tolerated path.

### Concerns

1. **No test specifically proves the fallback still works.** With
   this packet, every test sets canonical names. The legacy
   fallback logic in `scan.rs` still exists, but no test exercises
   it. A single "legacy alias still resolves" regression test
   would guard against accidental fallback removal. Low effort,
   clear value given the whole point of keeping the fallback is
   compat.
2. **Linker gap.** Pure test-env rename; minimal risk, but as
   always, not executed locally.

### Observation

Finishes the canonical/legacy split: runtime accepts both,
canonical tests demonstrate canonical, and nothing in the
regression suite advertises the legacy names as a good pattern.
