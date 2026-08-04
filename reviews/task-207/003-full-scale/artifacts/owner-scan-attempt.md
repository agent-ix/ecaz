# Full-scale owner-scan disposition

The benchmark-enabled release extension was installed and the 50k control
fixture passed release preflight, topology, serving, and remote-owner checks.
The registered owner-scan child then ran an exact owner scan for each recall
and latency query; after approximately 15 minutes it had emitted no child
result artifact. The suite was interrupted before the candidate step to avoid
leaving a multi-hour exact-scan matrix resident.

The 50k/100k closeout suites therefore compare the production persisted-head
strategy only. Owner-scan remains covered by the 10k Task 207 diagnostic and
the feature-enabled installation is retained in the packet provenance for the
attempt. The failed-attempt setup logs are under
`run-50k-feature/control/`; no result numbers from that attempt are claimed.
