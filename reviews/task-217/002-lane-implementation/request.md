# Task 217 — lane implementation and proof run

Implementation commit: `15834e2e4`.

This packet adds the structured same-generation attestation to
`ecaz bench suite`/`ec_distann` and provides the committed SuiteConfig for the
100k A/A and deliberately different runtime-switch A/B proof. The A/A pair is
`aa-control` / `aa-candidate`; the A/B pair is `ab-control` / `ab-candidate`.

The config pre-registers conforming NFR-021 controls and candidates and uses no
coordinator traversal replica. The suite runner emits `results.jsonl`; the
packet-local `artifacts/manifest.md` is the evidence index.

Static validation completed:

- `cargo check -p ecaz-cli` passed (one pre-existing dead-code warning).
- `ecaz bench suite audit --config .../task217-same-generation.json` passed.
- Suite dry-run expansion was inspected; the final release-built CLI must
  include `--same-generation-recall-pair aa-control,aa-candidate`.

The packet remains review-open until the packet-local 100k run proves one
generation identity for every arm, byte-identical A/A predictions, a matching
identity for the deliberately different A/B arm, and NFR-021 conformance in
`results.jsonl`.
