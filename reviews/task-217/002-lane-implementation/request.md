# Task 217 — lane implementation and proof run

Implementation commit: `15834e2e4`; evidence run head: `15834e2e4`.

This packet adds the structured same-generation attestation to
`ecaz bench suite`/`ec_distann` and provides the committed SuiteConfig for the
100k A/A and deliberately different runtime-switch A/B proof. The A/A pair is
`aa-control` / `aa-candidate`; the A/B pair is `ab-control` / `ab-candidate`.

The config pre-registers conforming NFR-021 controls and candidates and uses no
coordinator traversal replica. The suite runner emits `results.jsonl`; the
packet-local `artifacts/manifest.md` is the evidence index.

Validation and evidence completed:

- `cargo check -p ecaz-cli` passed (one pre-existing dead-code warning).
- `ecaz bench suite audit --config .../task217-same-generation.json` passed.
- Suite dry-run expansion included `--same-generation-recall-pair
  aa-control,aa-candidate`.
- `ecaz bench suite status` completed one step with zero failures, skipped
  steps, missing artifacts, or stale artifacts.
- All four physical arms reported the same epoch-fingerprint generation
  identity; the A/A prediction files were byte-identical.
- A/A recall was 0.9275 for both arms; warm latency was 37.70 ms control and
  38.20 ms candidate. Storage was 2,496,659,456 bytes with amplification
  1.351173.

The packet remains review-open pending outside review. The packet-local 100k
run proves one generation identity for every arm, byte-identical A/A
predictions, a matching identity for the deliberately different A/B arm, and
NFR-021 conformance in `results.jsonl`. The compact cited lines are in
`artifacts/run3/100k/attestation-evidence.log`; the complete structured output
is `artifacts/run3/results.jsonl`.
