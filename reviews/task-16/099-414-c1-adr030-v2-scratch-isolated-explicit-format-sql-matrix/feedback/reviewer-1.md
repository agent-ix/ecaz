## Feedback: Scratch Isolated Explicit-Format SQL Matrix

Read the packet, the eight per-cell summary-file citations, the
readout sections, and the `50k, m=16` crossover claim.

### What's right

- **Closes the SQL-side gap packet `413` explicitly deferred.**
  Pairing the direct-runtime matrix with the planner-verified SQL
  matrix on the same runtime lane is what makes a merge case from
  measurement, not just one.
- **Isolated one-index-per-table surfaces answer the planner
  problem honestly.** Packet `413` named the shared-table planner
  cross-choosing issue and declined to fake around it. This
  packet builds the surface where the planner *can* be verified,
  runs there, and explicitly names why the result does not
  generalize to shared tables. That is the right shape: fix the
  measurement surface, don't fake the measurement.
- **Every cell plan-inspected.** `--session-mode per-cell` +
  launcher plan-verification means every number in the tables
  is a genuinely planner-chosen `pq_fastscan`/`turboquant` index
  hit. No quiet planner drift.
- **The `50k, m=16, ef>=128` crossover is the landing headline.**
  On the clean surface, `pq_fastscan` is ahead on *both* recall
  (from `413`) and SQL latency (from this packet). That is the
  first bilateral-win cell on the entire 378–418 arc. Worth
  treating as the durable landing quote.
- **Git-provenance honesty.** The packet doesn't paper over the
  dirty `plan/`, `vendor/`, etc. — it narrows provenance to
  `src scripts docs` and confirms that narrower scope is clean.
  That is exactly the way to frame provenance when the tree is
  messy for unrelated reasons.

### Concerns

1. **Crossover is at `ef=128`+ only on `50k, m=16`.** The
   headline cell is strong, but the other three (`10k, m=8`,
   `10k, m=16`, `50k, m=8`) still favor `turboquant` on latency.
   If someone reads only the readout, "pq_fastscan crosses over"
   sounds like a general finding — it is specifically a
   `50k+, m=16+, ef>=128` finding. The packet does say this, but
   the outcome section could lead with the scoped claim, not the
   crossover.
2. **No standard error on SQL latency.** `query-limit 50` with 3
   warmup passes is the minimum to call a number "warm," but the
   observed within-cell variance is not reported. Some of the
   crossover margins are small (`4.263ms` vs `4.437ms` at
   `ef=128`) — a +/- on the mean would tell a reader which of
   these is a real crossover vs a noise crossover.
3. **Shared-table planner problem is named but not tracked as
   task.** The packet correctly scopes itself to isolated
   surfaces, but the unresolved "why does the planner cross-
   choose between sibling `m` indexes" question is now the
   blocking issue for a canonical production-shape SQL claim.
   That should be a real follow-up task, not just a closing
   paragraph.
4. **`cargo test` still "fails" per the validation list, but
   packet `415` has since changed that.** This packet was written
   before `415`'s standalone stubs landed. The linker-boundary
   boilerplate here is stale for the current head — reviewers
   reading 414 after 415 should read the test-execution claim
   through the 415 lens.

### Observation

Together with `413`, this is the full merge-case on measurement
terms. The `50k, m=16, ef>=128` bilateral-win cell is the single
most defensible landing quote the branch has produced, and the
isolated-surface honesty is the right way to make it. What is
left for task-15 merge is no longer measurement — it is test
execution (now unblocked by `415`) and a decision on the
shared-table planner question.
