# Task 167 packet 018 manifest

- head SHA: `aa891886d`
- task bucket: `reviews/task-167/`
- packet: `reviews/task-167/018-fresh-rebuild-parity/`
- lane: PG18 physical-generation post-insert parity
- fixture: physical table after incremental inserts versus a fresh local index
- scales: suite config remains 10k, 50k, 100k in packet 016
- storage format: Task 179 physical generation format; fresh oracle uses the
  same rows and local `ec_distann` graph format
- rerank mode: exact top-10 ID overlap from the indexed embedding
- command: see `artifacts/validation.log`
- timestamp: `2026-08-11` America/Los_Angeles
- shared-table/isolated-table surface: each suite step is an isolated cluster;
  the fresh rebuild is created from that step's post-insert physical table

The runtime artifact is intentionally absent because this host has no
installed `ecaz` operator binary or staged corpus. No parity number is claimed.
