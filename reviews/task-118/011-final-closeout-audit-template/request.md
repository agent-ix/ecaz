# Task 118 Review Request: Final Closeout Audit Template

## Scope

This checkpoint adds a packet-local audit template for the final Task 118 closeout.

The template maps Task 118 acceptance criteria to concrete evidence checks over the future Intel artifacts:

- artifact presence and selected-step status;
- synthetic score-correlation runtime test pass/fail check;
- normalized result row counts;
- candidate containment rows;
- rerank-boundary counters;
- source-build vs compressed-build recall comparison;
- score-correlation rows;
- packet 018 final-table extractor row-width and row-count checks;
- final dominant-loss classification table.

No benchmark is run here. This is a closeout-quality control artifact so packet 006 can be updated from the Intel rows without weakening the task definition.

## Validation

- The template uses only `ecaz bench suite` outputs and `jq` queries over packet-local artifacts.
- The expected row counts match the current checked-in Task 118 suite shape after packet 008 narrowed large-scale diagnostics to `ef_search=200` and packet 015 narrowed 10k diagnostics to the same final decision shape.

## Remaining Task 118 Closeout Work

Run the Intel 10k, 50k, and 100k suites, commit the final packet 006 artifacts, then fill the decision table using the audit template.
