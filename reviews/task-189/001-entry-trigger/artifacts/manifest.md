# Task 189 entry-trigger evidence

| artifact | source / result |
|---|---|
| Task 183 exact-neighbor comparison | `reviews/task-183/002-codec-attribution/`; same-seed RaBitQ recall 0.9625 vs exact 0.9605; warm p50 43.8 ms vs 113.1 ms |
| Task 183 STOP decision | `reviews/task-183/006-full-scale-decision/request.md`; no codec candidate and no unchanged exact-neighbor rerun |
| Task 188 Phase 1 attribution | `reviews/task-188/002-search-graph-attribution/`, branch head `c3f52fdd6`; owner oracle recall 0.9970 at 2487.70 ms mean, bounded BW4/H100 recall 0.9740 at 42.40 ms |
| Task 188 same-seed screen | `reviews/task-188/002-search-graph-attribution/`, branch head `c3f52fdd6`; BW8/H100 recall 0.9805 at 42.70 ms mean |
| Task 188 full-scale confirmation | `reviews/task-188/003-isolated-candidate/` and `004-full-scale-decision/`, branch head `171b84898`; BW8 recall 1.0000/0.9865/0.9805 at 10k/50k/100k, storage equal to BW4 at every scale |
| Task 188 attribution interpretation | BW8 changes beam width/search work while head seed digest, graph, payload codec, and persisted bytes remain fixed; no codec-ordering loss was isolated |
| provenance | Task 183 and Task 188 preserve same-seed attribution; Task 189 adds no code or production format |
