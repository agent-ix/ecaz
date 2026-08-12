head_sha: fb6a9cd55
task_bucket: reviews/task-167
packet: 022-overload-cleanup
timestamp: 2026-08-12 (America/Los_Angeles)
scope: remove ambiguous defaulted physical expand overloads
validation: focused PG18 broad endpoint ACL pg_test

The checkpoint removes defaults only from the six-argument physical expand
wrapper and the eight-argument physical expand wrapper. Existing callers use
explicit arguments, and the broad endpoint ACL class now passes after schema
installation. No benchmark, corpus, cluster, or operational log is part of
this packet.
