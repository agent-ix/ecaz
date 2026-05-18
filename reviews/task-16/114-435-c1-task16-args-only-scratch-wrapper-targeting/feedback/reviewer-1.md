## Feedback: Args-only scratch wrapper targeting — ACCEPTED

Verified against:

- commit `50f62ad` adding `--socket-dir` / `--port` to five
  scratch wrappers and `-e`/`--env` forwarding to
  `scripts/restart_adr030_scratch.sh`
- `scripts/tests/test_pg17_scratch_psql_socket_resolution.py`
  extended for the new arg path

### What's right

- **Generic, not task-specific.** The `-e NAME=VALUE` forwarding is
  the load-bearing change. It lets any future runtime experiment
  control ride through the standard restart helper without a
  bespoke flag per seam. Good instinct: don't add a
  `--turboquant-exact-score-mode` flag when a generic env forwarder
  does the job.
- **Wrapper targeting is consistent across the five scripts.** All
  accept `--socket-dir` / `--port`. That is the right set for a
  user to target an owned scratch cluster without env-prefix.
- **Test updated alongside the code.** The wrapper socket-resolution
  test now covers the new arg path, so a future refactor of the
  resolution logic gets caught.
- **Correctly scoped.** The packet explicitly says "this is not
  task-16 scorer plumbing." It is the minimum generic surface
  needed so subsequent measurement packets can run through the
  approved script forms. Keeping this slice small and non-task-
  specific is the right call.

### Concerns

1. **`restart_adr030_scratch.sh` still force-exports
   `TQVECTOR_PQ_FASTSCAN_RERANK_MODE`.** This was flagged in packet
   `432` §4 as a measurement hazard — a user following the standard
   helper cannot measure the persisted-reloption default lane,
   because the helper overrides it. This packet adds the generic
   `-e` forwarder but does not address that specific existing
   behavior. Worth a short follow-up: either drop the forced export
   when the requested mode matches the Rust default, or add a
   `--no-rerank-mode-override` flag.
2. **`-e NAME=VALUE` validation.** The packet doesn't say whether
   the helper validates the env name pattern. A mis-typed env (e.g.
   `TQVECTOR_TUBROQUANT_EXACT_SCORE_MODE=...`) would be silently
   forwarded and simply not take effect. Not urgent, but worth a
   minimal lint for `TQVECTOR_*` / `PQ_FASTSCAN_*` prefixes at
   least, so measurement packets can't accidentally produce results
   under a typoed env.
3. **No script-surface test for the `-e` forwarder itself.** The
   new `--socket-dir` path is tested; the env forwarder is not.
   Since packets `436`/`437` both depend on this surface being
   correct, an end-to-end test that asserts the forwarded env
   actually reaches the Postgres backend would be high-value and
   cheap.

### Call

Accepted. Solid infrastructure packet that unlocks the rest of the
task-16 matrix work without approval-gated env-prefix
invocations. Please still land the packet-`432` restart-helper fix
(concern `1`) — that one is a measurement hazard, not a surface
polish.
