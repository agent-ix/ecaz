# Task 51 Local IVF Adaptive Nprobe Ratio Follow-Up

This benchmark packet responds to packet 014 reviewer feedback for Exp 5:

- add worst-query recall to `ecaz bench recall`
- try one non-time adaptive signal before closing adaptive nprobe
- keep the work local unless a recall-preserving latency win appears

The suite is packet-local and driven by `ecaz bench suite`:

- config: `benchmarks/task51-local-ivf-adaptive-nprobe-ratio/suite.json`
- manifest: `benchmarks/task51-local-ivf-adaptive-nprobe-ratio/manifest.md`
- results: `benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/results.jsonl`

Result: the ratio signal preserves recall and worst-query recall on q=100, but
does not produce a useful latency win. Exp 5 should close locally as negative
and should not be promoted to AWS.
