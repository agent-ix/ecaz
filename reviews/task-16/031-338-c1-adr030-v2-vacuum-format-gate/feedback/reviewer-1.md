## Feedback: ADR-030 v2 Vacuum Format Gate

This closes the vacuum-path safety gap from packet 328 / 333. Verified:
`src/am/vacuum.rs` has `validate_vacuum_storage_format`, and both `ambulkdelete` and
`amvacuumcleanup` gate through it at lines 81 and 97.

### What's right

- Both vacuum callbacks covered, not just `ambulkdelete`. A `VACUUM (FULL)` with no
  dead rows will invoke only `amvacuumcleanup` — the one-callback-only gap would be
  subtle. This packet avoids it.
- Two unit tests plus a pg-test. Explicit coverage of the experimental build path
  → vacuum attempt → specific error.
- Updating the existing grouped-v2 ordered-scan rejection test to drop `ANALYZE`
  (now correctly tripping the vacuum gate) is the right follow-through. Tests
  should model reality.

### Concerns

1. **`ANALYZE` reaches vacuum callbacks.** Not obvious from the surface — `ANALYZE`
   reaching `ambulkdelete` is a pgrx / Postgres AM convention quirk. The fact that
   this caused an existing test to need updating is a good signal that the gate is
   doing real work. Worth a one-line note in the ADR or README that `ANALYZE` is
   gated too, since an operator might try `ANALYZE` to "just check" their v2 index
   and get a loud error.

2. **`autovacuum` interaction.** A grouped-v2 index in production with autovacuum
   enabled will produce errors at every autovacuum run. That's probably the right
   behavior today (experimental index should not accumulate dead-tuple debt
   silently), but operators who can't turn off autovacuum per-index will see log
   spam. Worth documenting how to coexist.

3. **`pg_repack` / third-party maintenance tools.** Out of scope for this packet,
   but the same gate principle will apply. When the gate is eventually lifted,
   documentation should name the maintenance tools that will need to be grouped-v2-
   aware vs those that go through the AM interface and are auto-covered.

### Observation

Insert + vacuum gates together close the two most pressing gate-lift blockers. Cold
rerank fetch (packet 339) is next.
