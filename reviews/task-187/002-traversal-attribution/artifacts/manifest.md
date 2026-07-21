# Task 187 traversal attribution manifest

- Source packet: `reviews/task-187/001-post-materialization-baseline/`
- Source head: `fe98ea2cc2273465ae28a942b7dacf9ca347a0de`
- Source suite: `task187-post-materialization-baseline-100k`, audit passed.
- Decision: STOP; no isolated production candidate selected.
- Dominant measured component: remote owner expansion `6.174150 ms` of
  traversal `7.468375 ms` (82.7%); local expansion `1.229638 ms`; derived
  control/frontier remainder `0.064587 ms`.
- Related non-traversal materialization: remote materialize `10.018400 ms`.
- Durable evidence remains in packet 001 `run/results.jsonl` and compact
  summary; no corpus or operational logs are copied here.
