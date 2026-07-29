# Rerun outcome

- Config: `../task188-batch10-stage-counters-suite.json`
- Command: `ecaz bench suite run` with the preregistered 100k physical
  stage-counter step and explicit `materialization_batch_size=10` for BW4 and
  BW8.
- Disk preflight: passed with approximately 653 GB free.
- Physical setup: passed; all three nodes reached `Published`, with 100,000
  source rows and zero orphaned rows.
- Benchmark outcome: incomplete. The PostgreSQL latency backend grew to
  approximately 52 GB RSS during the repeated-query phase, leaving about 10
  GB available memory and continuing to grow. The run was terminated to avoid
  exhausting the host.
- Evidence decision: no latency or stage-counter values from this rerun are
  valid closeout evidence. The final diagnostic remains outstanding.
