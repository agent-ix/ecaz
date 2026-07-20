# Verdict: Task 179 AC-13 is factually complete; Task 172 remains open

## Task 179 acceptance evidence

The current physical design has canonical 10k/50k/100k A/B evidence for every
axis named by Task 179 acceptance criterion 13:

- recall@10 is measured against the same-run single-instance ec_distann control
  and is equal or higher at every scale;
- warmed latency has 10 untimed same-connection warmups plus 50 measured
  queries per arm and scale on the post-fix production read path;
- cluster physical-generation storage is measured at every scale; and
- topology proves exact/disjoint owner coverage, zero residue/orphans, and
  remote-path engagement.

The latency result is not a performance promotion: physical mean remains about
14–16x the same-host single-index control. The storage result likewise does not
promote NFR-018 because 50k/100k slightly exceed 4.0x raw-vector bytes. Those
are measured product findings, not missing evidence.

Subject to outside-reviewer acceptance, the specific Task 179 AC-13 evidence
condition is complete.

## Task 172 remains open

This packet does not claim the full Task 172 gate. Still missing are:

- concurrency/throughput curves at 1, 2, 4, 8, and 16;
- first-class per-query remote call/row/byte and coordinator timing telemetry;
- benchmark-mode versus full-metrics instrumentation overhead;
- CPU/RSS/IO bottleneck attribution; and
- a measured 1m/10m capacity model.

Those surfaces are valuable Task 172 performance work but are not additional
words in Task 179 AC-13, which specifically requires reviewer-accepted
10k/50k/100k A/B recall, latency, and storage evidence.
