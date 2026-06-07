# Task 85 Packet 032: Candidate-Surface Stop Condition

## Result

This packet closes the Task 85 candidate-surface redesign workstream as
rejected for the current product-scale same-recall latency goal.

The goal was not "reduce candidates at any recall." The goal was lower latency
while retaining the current recall surface. Across Tasks 83-85, every
candidate-surface direction identified for this retained point has now exited
with packet-local evidence:

| Direction | Evidence | Decision |
| --- | --- | --- |
| blanket global cap growth | Task 83 packet `002` | rejected: recall gains came only by adding 1.02M-4.09M candidates |
| block8 geometry | Task 85 packet `004` | rejected: did not produce a better retained-recall latency/candidate point |
| per-leaf block cap | Task 85 packet `005` | rejected: changed the surface and lost recall |
| block32 geometry | Task 85 packet `006` | rejected: same-recall movement required candidate inflation |
| route-prior rescue | Task 84 packet `002`, closeout packet `006` | rejected: recovered zero selected-leaf misses |
| k=3 summary representatives | Task 84 packet `005`, closeout packet `006` | rejected: retained cap kept the same miss split; higher cap matched blanket-cap growth |
| selective near-cap rescue | Task 84 packet `006` | rejected: narrow predicates recover too little recall; broad predicates become blanket-cap growth |

## Why This Closes The Direction

The remaining miss set is dominated by selected-leaf block-pruning/candidate-cap
misses. Task 83 showed `81` selected-leaf misses at the retained surface, and
all `81` had target block rank beyond the retained `global1152` cap. Task 84
then tested the plausible bounded recovery mechanisms against those misses and
showed that none can recover enough recall without becoming another cap-growth
policy.

That means the unresolved candidate-surface path is not an untried knob inside
the current product profile. A genuinely new learned/calibrated policy would
need a new data contract, training/evaluation split, and review standard before
it could be trusted as a product implementation. There is no packet-local
evidence in Task 85 that such a policy is ready to implement, and it cannot be
used to close the current same-recall latency task as a promised future win.

## Task 85 Ledger Decision

- Mark `candidate-surface redesign with recall preservation` as `rejected`.
- Do not accept any candidate-surface latency win that lowers recall, lowers
  rerank width, or merely renames blanket cap growth.
- Treat a future learned/calibrated candidate policy as a new task only after
  it has a concrete model/data contract. It is not an unclosed Task 85
  workstream.

## Evidence

See `artifacts/manifest.md` for packet-local source references and key result
lines.
