# lattice-trie-dictionary

Persistent compressed Unicode-grapheme radix dictionary port of 0xLattice/TrieDictionary. Upstream SHA is in UPSTREAM and original license is retained.

Implemented: insertion/update/removal with split/collapse, lookup, persistent setting/removing, iteration, prefix traversal, child traversal, value/key mapping, filtering, partitioning, gathering prefix values, persistent batch edits, grapheme-length filters, unique values and merging. Tests compare 1,000 deterministic mutations against BTreeMap and assert compression after every mutation.

Remaining parity: Swift-specific collection/index wrappers and full API differential fixtures. Rust exposes IntoIterator and FromIterator instead. The current suite includes 100 actual Swift mutation snapshots and canonical-equivalent Unicode lookup/update/delete tests. This is not a complete parity release.
