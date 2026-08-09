# Task 219 packet 001 — frontier assembly

This packet assembles the accepted normal-release frontier from Task 215's
committed `results.jsonl` at 10k/50k/100k. It preserves the Task 206
work-surface caveat and does not treat those diagnostic rows as release
forecast points.

The frontier has two measured arms at each scale: shipped BW4/H100/L32 and
BW64/H8/L64-effective. The candidate is higher-recall but slower at every
scale, with effectively identical storage. No intermediate arm is needed for
the policy decision; any new arm would require a separately justified normal
release run.

Please review `artifacts/frontier.md` and
`artifacts/manifest.md`. The requested verdict is whether the frontier is
complete and correctly traced to the source `results.jsonl`.
