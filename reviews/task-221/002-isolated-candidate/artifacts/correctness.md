# Task 221 MAT-22 correctness evidence

The physical PG18 suite reported `pass=true` for every materialization
correctness scenario. For each digest-checked scenario, the eager and
candidate digests matched; `null_ok=true` and `external_toast_ok=true` were
also reported. The scenarios were:

- fewer than one window
- exactly one window
- more than one window
- reject first window
- reject multiple windows
- null payload
- toasted projection qualification
- mixed local/remote rows
- post-first-batch remote failure

The same-generation recall gate also reported `byte_identical=true`; both
arms recorded recall and membership recall of `0.9290`. The complete structured
rows, including digests, payload-read bounds, remote/local consumption, and
the query SHA, are in `artifacts/results.jsonl`.
