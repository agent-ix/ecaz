# Request: Make Linear Result Materialization Match Bootstrap Shape

Commit: `31bab71`

Summary:
- The linear scan helper now materializes the next result into `current_result` plus pending heap TIDs and returns a boolean, instead of immediately emitting the first heap TID itself.
- `amgettuple` now owns the actual visible tuple emission for both bootstrap and linear result production through the same explicit pending-drain step.
- This keeps the staged executor on one clearer shape: materialize a result first, then drain it.

Files:
- `src/am/scan.rs`

Why this matters:
- The prior slices already made pending duplicate drain explicit and removed the dead fallback drain branch from the linear helper.
- This follow-on removes the remaining asymmetry where bootstrap materialized first but linear materialized and emitted in one helper.
- The current staged executor should now be easier to evolve toward more ordered graph-search result production because both paths share the same result/drain contract.

Review focus:
- Whether the bootstrap and linear paths now have the right common “materialize then emit” structure
- Whether any current behavior still depends on the old linear helper returning the first heap TID directly
- Whether this is a good base for later work that further decouples result production from the old linear scan path
