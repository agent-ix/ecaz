# Feedback: 636 Concurrent DSM Code Corpus

## Verdict: Accept

Flat fixed-width packing is the right representation for a worker-accessible
code corpus: constant stride enables O(1) `code_for_node` lookup without a
separate length table. Variable-width rejection is correctly placed here.

Leaving scoring metadata (`dimensions`, `bits`, `seed`) out of the corpus is
fine — they belong to the insertion config, not the raw byte buffer. The corpus
is a pure byte store; the scorer receives the config separately.

## No Issues
