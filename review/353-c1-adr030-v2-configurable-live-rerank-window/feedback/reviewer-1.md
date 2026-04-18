## Feedback: ADR-030 v2 Configurable Live Rerank Window

Read `resolve_grouped_live_rerank_window` at `scan.rs:715`, the amrescan
resolution at `scan.rs:599-608`, the buffer sizing at `scan.rs:3311-3312`,
and the active-window use inside the refill loop at `scan.rs:1941`.

### What's right

- **Closes the 351 hardcoded-constant concern cleanly.** The window
  is now resolved per-scan from env during `amrescan` for grouped-v2
  descriptors, defaulting to `4` and bounded by `1..=16`. Exactly the
  scope I asked for in 351 feedback.
- **Buffer storage is sized to the max, not the resolved width.** Line
  3311-3312 keeps `grouped_live_rerank_buffer:
  [BufferedGroupedScanResult; ADR030_GROUPED_V2_MAX_LIVE_RERANK_WINDOW]`.
  That means the worst-case memory footprint is fixed (16 ×
  `sizeof(BufferedGroupedScanResult)`), and changing the active
  window is a `u8` field update, not a reallocation. Right call —
  makes the runtime predictable and keeps the struct `Copy`-friendly.
- **Active-width wiring threaded through both the refill condition
  and the push invariant.** `push_buffered_grouped_scan_result` now
  asserts against `grouped_live_rerank_window(opaque)` at line 1839,
  and the refill `while` at 1942 compares `buffer_len < active_window`.
  So setting a smaller runtime window genuinely bounds the buffer fill,
  not just the capacity. If these two checks had drifted (e.g., refill
  capped at MAX but assert at active), the behavior would be silently
  wrong; this packet keeps them aligned.
- **Scalar descriptors keep the default inert.** The resolution at
  `scan.rs:599-608` only calls `resolve_grouped_live_rerank_window`
  when `scan_graph_storage` matches `GroupedV2(_)`. Scalar scans pick
  up the compile-time default `4` without touching env. This means
  the env var cannot break scalar scans even if someone sets it
  globally — appropriate scoping.
- **pg coverage exercises all three axes.** Rejects invalid env,
  matches default against `window=4` simulation, matches env-configured
  `window=8` against `window=8` simulation. These are the three
  boundaries that matter.

### Concerns

1. **Env resolution at amrescan, not at connection/session start.**
   `resolve_grouped_live_rerank_window` reads `std::env::var_os`
   inside `amrescan`. Per-scan resolution sounds cheap, but it means
   (a) the env value is re-parsed for every scan, and (b) env
   changes mid-session can take effect silently on the next scan.
   The first is a minor cost. The second is a subtle footgun: an
   operator running `\setenv` in psql and then a query gets one
   behavior; a process-level `systemctl set-env` affects only new
   backends. For an experimental gate-only knob this is acceptable
   — but the moment this becomes a supported tuning surface, it
   needs to become a GUC (`tqvector.adr030_grouped_rerank_window`)
   with `USERSET` context so the resolution is explicit in the query
   plan, respects `SET`/`RESET`, and shows up in `pg_settings`.

   For this packet, env is fine because it's already gated behind
   the `_SCAN` experimental env. Mention this as a known transition
   step for gate-lift, not a bug today.

2. **No test that changing the env between two scans in the same
   session changes the active window.** The pg coverage exercises
   each env value in isolation. A test that asserts "set env → first
   scan uses X → unset env → second scan uses default 4" would
   protect against regression where someone caches the resolved
   value on the scan-opaque init and forgets to re-resolve. (Today
   the code resolves in `amrescan` and stores on the opaque — if
   someone refactors to resolve in init, the per-scan contract is
   silently broken.)

3. **Error path on invalid env is `pgrx::error!` with a
   two-line message.** Fine, but the two messages at lines 722-726
   and 729-733 are nearly identical except for the "got {:?}" vs
   "got {}" formatting (debug-quoted vs raw). Unifying would make
   the "bad input" user experience consistent. Small nit.

4. **353's own 50k measurement is actually invalid** — see packet
   354. Not this packet's fault structurally (the env + resolution
   code is correct), but the reported `window=8` rerun in this
   packet at lines 100-124 was measured against an index that was
   not grouped-v2 on disk. That means the "widening the live rerank
   window alone does not close the 50k recall gap" conclusion at
   line 114 was derived from noise. The grouped numbers in this
   packet should be struck — 354 has the verified numbers.

   This is worth calling out for process hygiene: packet 352 asked
   the scratch cluster whether it had been restarted with the ADR-030
   build gate, and packet 352's answer was effectively "probably, but
   not verified." Packet 353 inherited that assumption. Packet 354
   caught it with the debug SQL surface. Going forward, the first
   step of any grouped measurement packet should be:
   `select emitted_result_count, grouped_result_count from
   tqhnsw_debug_grouped_scan_windowed_summary(...)` — and failing
   the packet if `grouped_result_count = 0`. Treat verification as
   structural, not optional.

### Observation

This is the right control-surface change made at the right time.
The code is clean. The problem is that 353's measurement conclusion
("widening alone does not close the gap") turned out to be
accidentally correct but for the wrong reason — the actually-scalar
index of course doesn't care how wide the grouped rerank window is.

Packet 354 reaches the same conclusion on verified indexes for
different reasons: at `window=16` on a real grouped index, the gap
narrows but doesn't close. So widening is still a lever, just not a
silver bullet. Net result: this packet's *code* is good, this
packet's *measurements* are contaminated.

### Measurement gap still open

This packet doesn't close any measurement gap — that's 354's job.
What it does close is a *tuning-surface gap*: without this packet,
354 couldn't have run window=16 without a code edit. Credit where
due.
