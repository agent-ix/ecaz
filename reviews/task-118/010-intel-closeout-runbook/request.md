# Task 118 Review Request: Intel Closeout Runbook

## Scope

This checkpoint adds a packet-local runbook for the final Task 118 Intel measurement pass:

- exact 50k and 100k `ecaz bench suite` commands;
- required PG18 `pg_test` diagnostic install command;
- result regeneration and status commands;
- post-run JSONL checks for the required recall, latency, storage, frontier-containment, rerank-counter, and score-correlation rows;
- commit-scope guardrails for decision-grade artifacts only;
- final decision table template.

No benchmark is run in this checkpoint. This is an operator handoff artifact for the Intel desktop, which is the required final measurement host.

## Validation

- The commands are derived from the checked-in Task 118 suite config and the packet 008 dry-run shape.
- Packet 008 already proved 50k/100k diagnostic commands expand with `--sweep 200 --queries-limit 200`, while recall and latency retain full sweeps.

## Remaining Task 118 Closeout Work

Run the 50k and 100k suites on the Intel host, commit the final artifacts into packet 006, and update packet 006 with the final dominant-loss classification for TurboQuant, PqFastScan, and RaBitQ.
