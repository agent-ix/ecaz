# Pre-measurement baseline guard failure

The first production-step launch at extension/runner SHA `df34e95aa` completed
the fresh 100k generation build and topology publication, then stopped before
any recall or latency arm with:

`ERROR: relation "ec_distann_retry_attribution" does not exist`

This diagnostic relation belongs to a Task 167 fixture and is not a production
schema requirement. Upstream commit `c9c9628eb` makes retry attribution
optional by checking `to_regclass` before inserting. It was cherry-picked
unchanged onto this isolated branch as `c51e74c5e`. No fixture-local table was
created, no benchmark result was produced or retained, and the stopped 6.5 GB
cluster plus PostgreSQL/runner exhaust were removed before the rerun.
