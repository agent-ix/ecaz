# Changelog

## Unreleased

### Changed

- DiskANN ambuild now treats exact duplicate source vectors as distinct Vamana
  graph nodes during index build. Runtime insert overflow heap-TID chaining is
  unchanged, but build no longer performs the prior O(N^2) exact-match scan
  over already-collected heap tuples.
