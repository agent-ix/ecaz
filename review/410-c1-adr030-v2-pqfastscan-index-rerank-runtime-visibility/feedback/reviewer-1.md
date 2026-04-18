## Feedback: PqFastScan Index Rerank Runtime Visibility

Read `PqFastScanRerankModeResolution` / `PqFastScanRerankModeDecision`
at `src/am/scan.rs:528–553`, `resolve_grouped_rerank_mode_decision`
at :1090+, the non-pq_fastscan short-circuit at :1128 and the
`for_index` wiring at :1348–1352. Verified the helper surface is
wired through `src/lib.rs` and covered with three pg tests.

### What's right

- **Completes the pattern packet `408` started.** Traversal and
  rerank now both follow the same "decision object + resolution
  reason" shape, and they share call-through helpers so the hot
  path still bottoms out on `.mode`. No hot-path cost for the new
  surface — exactly the right refactor shape.
- **Resolution enum names are operator-legible.**
  `DefaultHeapF32WithBuildSourceColumn`,
  `DefaultQuantizedMissingBuildSourceColumn`, `EnvOverride`,
  `NonPqFastScanStorage`. An operator reading a debug-helper row
  can tell *why* they got this mode without cross-referencing
  source. That is the whole point of the visibility packet.
- **Effective source-column only reported when heap rerank is
  live.** Not "nominal column if present" — the helper gates the
  column on the actual active mode, so an env-override to
  `quantized` nulls out the column field. This is the right
  contract for a "what did this index actually do" helper.
- **Three pg tests cover the three lanes an operator will ask
  about.** Source-backed default, env `quantized`, env `heap_f32`
  with explicit `source_raw` column. The env-override test proves
  overrides *and* the source-column plumbing in one shot.

### Concerns

1. **Fourth resolution value `NonPqFastScanStorage` is dead for
   the `_for_index(...)` helper.** Packet `408` already rejects
   non-pq_fastscan indexes up-front, so the `NonPqFastScanStorage`
   branch at :1128 / :1352 is only reachable if a caller uses the
   library helper directly. Fine to keep for completeness, but
   worth a one-line comment that the runtime helper's rejection
   path makes this branch cold in practice — otherwise a future
   reader may add ceremony around a dead path.
2. **Three helpers on the surface now, each with partially
   overlapping fields.** `tqhnsw_debug_pq_fastscan_runtime_settings`,
   `tqhnsw_debug_adr030_runtime_settings`, and the index-aware
   `_for_index`. The index-aware helper now reports *more* than
   the global — it has become the canonical one. Worth a
   follow-up packet to either mark the globals as deprecated for
   per-index questions or document the split-of-responsibility.
3. **Rerank-mode resolution is index-aware; traversal-mode
   resolution is still index-aware-but-separate.** Both decisions
   now live on the same helper but come from two resolver
   functions. Fine today — they answer different questions — but
   if a third "why did I get this" decision lands (e.g., live
   window adaptation), the pattern should probably consolidate
   into a single `PqFastScanRuntimeDecision` aggregate rather than
   growing N parallel resolvers.
4. **Same linker gap caveat applies, but see packet `415`.** The
   `cargo pgrx test pg17` failure cited here is the same boundary,
   but packet `415` has since landed standalone-stubs that make
   plain `cargo test` a real checkpoint — this packet was written
   before that. The merge reviewer should read them together: 410
   defines the contract, 415 makes some of it actually testable.

### Observation

The right finish to `408`. The enum-per-resolution-reason pattern
is starting to look like the durable shape for "why did this
scan behave this way" debugging, and consolidating both decisions
into the index-aware helper means the next recall investigation
has one helper to call instead of three.
