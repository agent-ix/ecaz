## Feedback: PqFastScan Runtime Env Surface Rename

Read the env-constant table at `src/am/scan.rs:23-47`.

### What's right

- **Canonical name wins, legacy name stays as compatibility
  fallback.** That's the right conservative sequence — introducing
  new names without breaking existing local tooling. Lookup
  semantics are documented as "canonical first, legacy second," so
  operators who set both end up with the canonical value, which is
  the expected precedence.
- **User-visible error for rerank source column now points at
  the canonical env name.** That was the one place where a legacy
  name would have confused a user who only ever saw the canonical
  name in docs. Fixed here.
- **Scratch script, debug helper, and pg tests all updated.** The
  canonical names get actual regression coverage, not just
  definition.

### Concerns

1. **No deprecation timeline for the legacy names.** ADR-030 branch
   history is well-intended to preserve here, but "keep the
   `EXPERIMENTAL_ADR030_V2` names forever" is also not the end
   state. Worth a one-line comment on the fallback table saying
   "legacy aliases; slated for removal in task N" (even if N is
   TBD) so future cleanup has a pointer.

2. **Same env set on both canonical and legacy names produces no
   warning.** Silent precedence is fine for the common case, but
   if an operator sets both with different values (e.g., during a
   migration), they'll see canonical-wins behavior with no
   indication that their legacy setting is being ignored. Not a
   blocker, but a one-line "mismatch detected, using canonical"
   at debug level would save debugging time later.

3. **Linker gap.** Env-parsing tests did not run locally.

### Observation

Well-staged compatibility rename. Canonical names first with
fallback aliases is the shape task 15 needs — eliminates
"EXPERIMENTAL" branding from the public surface without breaking
anyone's existing workflow.
