# NFR-021 growth calculation

The suite's owner-control storage summaries report the largest published
single-node graph side as:

- 10k: `25,706,496` bytes
- 50k: `137,379,840` bytes
- 100k: `277,372,928` bytes

The required 100k/10k growth is:

`277,372,928 / 25,706,496 = 10.789993627`

NFR-021 requires growth `<= 2.0`, so the owner-control surface FAILS the
admissibility gate. The 50k/10k ratio is `5.344168260` and also exceeds the
threshold. Candidate and control have identical graph-side values at all
scales.
