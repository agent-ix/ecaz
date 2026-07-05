# Artifact Manifest

Packet: `30147-task28-ivf-recall-truth-cache`

No measurement artifacts are recorded in this packet. The packet records a
code-only harness change plus focused unit-test validation.

Validation command:

```text
cargo test -p ecaz-cli recall -- --nocapture
```

Key result:

```text
23 passed; 0 failed; 0 ignored
```

The same validation command was re-run after replacing the per-query full sort
with partial top-k selection.

Key result:

```text
23 passed; 0 failed; 0 ignored
```
