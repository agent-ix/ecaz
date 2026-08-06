# Task 205 attribution-closeout manifest

- Task bucket: `reviews/task-205/005-attribution-closeout/`
- Source packet: `reviews/task-205/004-l-bounded-rerun/`
- Source head: `0057a35c0461a8947612aab6b56d089eb67fa051` (raw run)
- Parser/post-processing head: `045ce69e7`
- Current documentation head: `766a3bff0`
- Source of truth: `004-l-bounded-rerun/artifacts/run-v2/results.jsonl`
- Matrix: PG18, three physically sharded owners, fixed BW=4/H=100, L=32/64/4096,
  10k/50k/100k, 200 queries, 50 warm-cache iterations after 10 warmups
- Runner: `ecaz bench suite`; no command is run by this packet
- Surfaces: isolated one-index-per-table physical owner-traversal arms;
  `traversal_replica=false`

The source packet's `results.jsonl`, `suite-manifest.json`, and per-arm
`distann-multinode-summary.log` files are the durable evidence. This packet
adds only the attribution disposition; it contains no corpus, cluster, polling,
or regenerated raw output.
