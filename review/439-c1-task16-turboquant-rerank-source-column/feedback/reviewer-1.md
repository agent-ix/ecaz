## Feedback: TurboQuant `rerank_source_column` — ACCEPTED

Verified against:

- commit `95bf079` — `src/am/options.rs` reloption parser no longer
  gates `rerank_source_column` on `storage_format = 'pq_fastscan'`
- `src/am/build.rs`: `validate_grouped_rerank_source_column(...)` and
  `validate_grouped_rerank_source_column_for_empty_build(...)`
  apply to any storage format and keep `real[] or bytea` as the
  type policy
- `src/am/scan.rs`: `effective_grouped_rerank_source_column(...)`
  returns env > `rerank_source_column` > `build_source_column`
  with no storage-format branch; default resolution also uses it
- new coverage in `src/lib.rs`:
  `test_turboquant_persisted_rerank_source_default_stays_quantized`,
  `test_turboquant_persisted_bytea_rerank_emits_scores`, and the
  renamed `test_turboquant_rerank_source_rejects_wrong_type`

### What's right

- **Gate removal is the minimal correct change.** Packet `431`'s
  pq_fastscan-only fence was a scope hedge, not a principled
  restriction — the underlying runtime path was already generic.
  Removing the reloption parser gate and renaming
  `validate_pq_fastscan_rerank_source_column*` to
  `validate_grouped_rerank_source_column*` actually matches what
  the code does.
- **Default policy is unchanged.**
  `test_turboquant_persisted_rerank_source_default_stays_quantized`
  locks in that adding `rerank_source_column` does not flip
  TurboQuant to heap rerank. This is the exact shape the reviewer
  asked for when flagging packet `431` — productize the option
  without moving the default lane.
- **Runtime precedence is explicit.**
  `effective_grouped_rerank_source_column(...)` is:
  `env > rerank_source_column > build_source_column`. Three
  rules, named, in one function, with no storage-format branch.
  Whoever touches this next won't need to guess.
- **bytea parity test proves the column is actually used.** The
  new fixture persists a `source_raw bytea` that disagrees with
  `source` and asserts emitted scores match `source_raw`. That
  closes the "is the DDL accepted but ignored?" gap that a
  column-presence-only test would leave open. This is the right
  way to test a seam that could silently fall back to
  `build_source_column`.
- **Error-text generalization.** Dropping `PqFastScan`-specific
  wording from the heap-rerank missing-source error was the right
  sweep to pair with the gate removal — otherwise users of a
  TurboQuant index would get a misleading message about
  `pq_fastscan` that no longer applies.

### Concerns

1. **This is DDL plumbing, not a runtime verdict.** The packet is
   honest about this ("it is still not a measurement packet") and
   packet `440` then measures the supported path. Worth keeping
   that framing tight: the *value* of this slice is unlocked only
   by `440`'s measurement. Landing `439` without `440` would be
   shipping a bigger product surface without proof it helps.
2. **Type policy still `real[] or bytea`.** That's consistent with
   `build_source_column`, and it should stay that way. But if
   ADR-043's `tqvec` type lands, the type policy will need to grow
   a new accepted type — worth a one-line note on the
   `validate_grouped_rerank_source_column` function pointing at
   the ADR, so the `tqvec` addition has an obvious extension
   point. Not a blocker.
3. **No script-surface test that verifies the reloption survives
   ALTER INDEX SET / RESET round-trip.** Packet `440` relies on
   `ALTER INDEX ... SET (rerank_source_column = source_raw)` and
   `RESET (rerank_source_column)` to flip between modes on the
   same index. A pg_test that exercises that ALTER cycle (not
   just the CREATE-with-option path) would lock in that the
   reloption is both settable *and* resettable on an already-built
   index — because if RESET is broken, `440`'s measurement
   methodology silently breaks too.
4. **`test_turboquant_rerank_source_rejects_wrong_type` replaces
   the old non-TurboQuant rejection test.** That's correct — the
   old test was testing the wrong thing (a gate we just removed)
   — but the commit message / packet should name that coverage
   wasn't *dropped*, it was *replaced*. Otherwise a future audit
   might count this as a coverage regression.

### Questions for coder-1

1. **When `rerank_source_column` is set but `build_source_column`
   is not, and the default-lane resolution flows through the
   precedence chain for a non-heap_f32 mode, what actually
   happens?** `effective_grouped_rerank_source_column(...)` early-
   returns `None` for non-`HeapF32` modes, so the source-column
   selection is a no-op on the quantized lane — which is right.
   Worth confirming in a one-liner test that a TurboQuant index
   with `rerank_source_column` set and no `build_source_column`
   still runs the quantized default cleanly without trying to
   bind the rerank column.
2. **Does the runtime fixture exercise the ALTER INDEX SET path,
   or only the CREATE-with-option path?** See concern `3`.
3. **Is there any path where a user could set both
   `rerank_source_column` and `build_source_column` to the same
   column name and silently pay a double-resolution cost?** If so
   this is cheap to detect in the validator and probably worth
   rejecting at DDL time, since it's a configuration mistake
   rather than a valid setup.

### Call

Accepted. Correct product-surface widening with default-preserving
semantics and real coverage that the persisted column is actually
consumed. Concerns are polish; none block the slice. Packet `440`'s
measurement is what turns this slice from "DDL plumbing" into a
justified lever, so please read the two together.
