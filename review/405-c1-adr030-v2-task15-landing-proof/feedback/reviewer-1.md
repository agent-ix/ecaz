## Feedback: Task-15 Landing Proof

Read the packet's four proof sections, cross-referenced the cited
artifact paths in `tmp/real_corpus_runs/`, re-ran the `rg` grep for
the old gate / unsupported symbols against the current tree, and
checked packet `393` for the cited round-trip test names.

### What's right

- **Correct framing: proof packaging, not implementation.** The
  packet explicitly disclaims code change. That is the right shape
  for a landing packet — it names the head, points at already-landed
  artifacts, and makes *one* reviewable claim: "task 15's technical
  bar is satisfied at this SHA." No hidden behavior change slipped
  in under a rollup packet.
- **All four task-15 acceptance criteria named with concrete
  evidence.** Explicit `turboquant` gate (TSV), explicit
  `pq_fastscan` gate on the 404 default lane (TSV), round-trip test
  names in `src/lib.rs` citing packet `393`, and a `rg` incantation
  proving the old symbols are absent from runtime code. Each claim
  traces to something a reviewer can re-verify locally.
- **Runtime settings captured inline with the `pq_fastscan` run.**
  Listing `window=64 / score_mode=binary / rerank_mode=heap_f32 /
  rerank_source=build_source_column` directly next to the result
  row means future readers can tell what configuration produced
  the `0.9078` without having to dig through 404 and 401 to
  reconstruct the lane.
- **`m=8, ef=128` clears `0.89` on both formats.** The one
  real-corpus row that was still missing through packet `403` is
  now green on both `turboquant` (`0.8927`) and `pq_fastscan`
  (`0.9078`). That is the load-bearing data point for task 15.
- **Grep for old symbols is reproducible.** The exact `rg`
  pattern is in the packet body, so the "runtime is clean of old
  gates" claim is testable in one command rather than by
  inspection.

### Concerns

1. **Proof was produced on `~/.pgrx`, not on the hardened scratch
   cluster.** The `TQV_PG_SOCKET_DIR=/home/peter/.pgrx` invocations
   mean this landing proof ran on the *other* cluster — the exact
   one packet `402` just taught the wrappers to refuse to drift
   onto silently. That is fine provided the `~/.pgrx` cluster has
   the current-head extension binary installed (the packet claims
   it does), but a second gate run through the preferred scratch
   cluster would close the loop. Without it, "landing proof"
   depends on trust that the right binary was installed on
   `~/.pgrx`.
2. **Round-trip proof is claimed by reference, not exercised in
   the packet.** The round-trip tests were written in packet `393`
   and have never executed — same linker gap as the rest of the
   arc. A landing-proof packet that rests on "test X is defined in
   src/lib.rs" is exactly as strong as whether test X has actually
   run. For a merge-gating packet that needs to be said out loud,
   not glossed.
3. **No latency / throughput number alongside the recall row.**
   The task-15 definition-of-done focuses on recall, but a merge
   claim "the new default lane is correct" that cites only recall
   leaves a regression window on latency. A `mean_query_latency_ms`
   column in the same gate TSV would make this landing packet
   fully bilateral.
4. **Head SHA named, but no `git status` / `git log` pin.** The
   packet cites `215db8ce...` but does not confirm the tree at that
   SHA is clean (no staged or unstaged changes) and that it is the
   tip of the pushed branch. A merge review should be able to check
   out exactly that commit and reproduce the artifacts. A one-line
   `git status --porcelain` at that SHA + a `git log --oneline -1`
   output would lock that down.
5. **No mention of the `invalid=298` column on the
   `pq_fastscan` summary.** Separate from this packet's scope, but
   the live TSV this packet cites shows 298/1000 invalid queries —
   that is a big enough fraction to mention once in the landing
   narrative, even if it is a harness bookkeeping quirk rather than
   a recall regression. Quiet on it risks being read as "we didn't
   look."
6. **CI gap unaddressed.** This is the merge-bar packet for task
   15. "Every `#[pg_test]` across 378–404 has never executed
   anywhere" is a real gap, not a workstation quirk. A landing
   packet that does not speak to that gap leaves the merge
   reviewer to discover it.

### Observation

The shape is right — a rollup packet at the landing moment is how
you hand a merge reviewer something reviewable in one read. The
content is mostly there. What would turn this from "likely ready"
to "ready" is a one-paragraph section that says, explicitly: "the
following pg tests added across 378–404 are unexecuted because no
CI lane runs `cargo pgrx test pg17` on this project; before merge,
we will either stand up that lane or run these tests manually on a
machine where the link works and capture the output." Without that,
the landing proof is honest about recall and silent about test
execution, and silence on the test-execution question is where
merge reviewers will (correctly) pause.
