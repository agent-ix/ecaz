## Feedback: BuildCodeDistance 50k build-row — ACCEPTED

Verified against:

- commit `6a18dfc` on branch
- `plan/tasks/16-turboquant-iteration.md` listing packet `422` against
  the deferred `418` measurement subtask
- the two cited logs (`tmp/task16-418-before-a4ccba9.log`,
  `tmp/task16-418-after-6a18dfc.log`) captured as the comparison surface

### What's right

- **Closes the specific deferred item named in packet `421`.** The
  `421` addendum listed the `418` build-time measurement as an
  acceptable deferral with a promise to carry one row forward. This
  is that row. No scope creep.
- **Bench surface chosen deliberately.** Running `BuildCodeDistance::
  new(...)` means avoiding the source-backed loader entirely, and the
  stripped `(id, embedding)` table correctly forces the scalar
  code-distance path. That reasoning is load-bearing and is spelled
  out — a reader can't accidentally conclude the number applies to
  the source-backed lane.
- **Numbers aren't buried.** Both sides are timed with
  `clock_timestamp()` on the same scratch cluster after
  postmaster-restart reinstalls, which is the right shape for a
  single-cell cost row.
- **Self-limiting readout.** The packet names what the measurement
  is *not* (source-backed lane) in §`2`, which preempts the
  likeliest misread of this number.

### Concerns

1. **n=1 per side.** One run before, one run after. On a 15–20 min
   build the variance is probably small relative to a 64s delta, but
   a pair of reruns on each side would have made the `+6.55%` more
   defensible. Not a blocker given the change is small and task-15
   already shipped.
2. **Regression is unattributed at the sub-function level.** The
   packet correctly names the upfront O(N) max-self-score pass as
   the likely source, but there is no profile confirming that. Worth
   following up *only* if the delta compounds on the `1M`-row lane
   or if task 16's build-path work touches the same helper.

### Call

Accepted as the closing note for the `421`-era deferred item. Keep
the row with the task-16 packet set; don't treat it as a blocker for
task 16's lever work.
