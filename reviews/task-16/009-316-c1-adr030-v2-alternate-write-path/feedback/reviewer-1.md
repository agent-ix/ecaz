## Feedback: ADR-030 v2 Alternate Write Path

The alternate write path is the right shape: it exists alongside the scalar path
without replacing it, gated by the same build-time conditions that will gate runtime
later.

### What's right

- Keeping the scalar and grouped write paths separate (instead of a conditional inside
  the scalar writer) means neither path grows accidental complexity. Each can be
  reasoned about independently.
- The alternate write path only engages under the `build_source_column` condition that
  is already required for v2 builds. One gate, not two.

### Risks to watch

1. **Drift.** Two write paths that share no code will drift. They share tuple-tag
   writing in `page.rs` through the DataPage chain, which is good, but anything above
   that (layering, free-space accounting, FSM integration) will need to stay in sync.
   Worth adding an assertion inside the scalar writer that it never sees
   `GraphStorageFormat::GroupedV2` metadata, so that drift at the metadata layer is
   caught at runtime.

2. **Partial-failure recovery.** If an alternate write path crashes mid-flush, does the
   index end up in a state that the scalar path can clean up? Today, probably not —
   there is no vacuum awareness of grouped tuples. Not a blocker for this packet, but
   record it so we don't discover it under load.

### Observation

The fact that the alternate path is testable in isolation before runtime scoring exists
is the correct way to de-risk this. The incremental packet strategy keeps paying off.
