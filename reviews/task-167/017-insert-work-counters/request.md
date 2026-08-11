# Task 167 checkpoint: bounded physical insert-work counters

This checkpoint adds benchmark-only counters for insert attempts, search
candidates, selected forward neighbors, backlink amendments, owner writes,
and graph records appended. The fixture resets the counters immediately before
the physical insert arm, emits each value and `mean_per_insert`, and rejects a
forward-neighbor or backlink total above `graph_degree * inserts`.

The physical path records work at the coordinator before owner RPCs, so the
packet measures the bounded planning and routing work even when the owner is a
remote participant. The counter module is available in normal builds, while
the SQL snapshot/reset endpoints remain behind the benchmark feature.

Validation passed for normal PG18 extension compilation, benchmark-instrumented
PG18 extension compilation, and `ecaz-cli` compilation; see
`artifacts/validation.log`. The configured physical suite has not executed on
this host because the installed `ecaz` operator binary and staged corpora are
absent. This packet is review-open and does not claim Task 167 closeout.
