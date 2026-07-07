# Task 146 Packet 002 Artifact Manifest

- Head SHA: `fecabfe0b2463f4894334332782aa1aa31f561f1`
- Task bucket: `reviews/task-146`
- Packet path: `reviews/task-146/002-multinode-suite-config`
- Timestamp: `2026-07-07T00:30:08Z`
- Packet type: suite-config review request
- Isolated one-index-per-table or shared-table surface: pending execution; the
  config uses separate coordinator/remote index names per scale/shape

## Artifacts

- `suite-task146-pareto-multinode.json`
  - Command used: hand-authored `ecaz bench suite` config matching packet 001
  - Key result: 18 `spire-local-multinode` steps for 3 scales x 6 shapes
- `dry-run-suite-manifest.json`
  - Command used:
    `target/release/ecaz bench suite run --config reviews/task-146/002-multinode-suite-config/artifacts/suite-task146-pareto-multinode.json --dry-run --manifest-output reviews/task-146/002-multinode-suite-config/artifacts/dry-run-suite-manifest.json --log-file reviews/task-146/002-multinode-suite-config/artifacts/dry-run.log`
  - Key result: 18 selected steps
- `dry-run.log`
  - Command used: same dry-run command as above
  - Key result: expanded all 18 suite commands without execution
- `audit.log`
  - Command used:
    `target/release/ecaz bench suite audit --config reviews/task-146/002-multinode-suite-config/artifacts/suite-task146-pareto-multinode.json`
  - Key result:
    `[suite:task146-pareto-multinode] audit passed: 18 steps`

## Non-Claims

- No matrix cells were executed in this packet.
- No recall, latency, storage, or scan-fraction conclusion is made here.
- Bound-prune remains excluded because Task 145 proved it inert/null on the
  remote path.
- Packet 001 feedback obligations are not closed by this config packet; the
  matrix/result packet still owes HNSW anchors, matched 10k/50k anchors,
  epoch-cache engagement proof, viability-vs-dominance wording, and 15% scan
  gate justification.

## Sweep Rationale

The multinode config intentionally uses the SPIRE program sweep
`8,16,32,64,96`, not the registered `ec_spire` default sweep
`8,16,24,32`. This is a bespoke grid for comparability with the accepted SPIRE
program evidence: Task76 and Task139 used this high-recall sweep shape, and the
Task146 frontier/gate compares against anchors and prior release evidence at
the same operating points. The grid is therefore fixed as part of the packet
001 preregistration and should not be changed during execution.
