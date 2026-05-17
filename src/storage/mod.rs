//! Cross-AM physical storage primitives.
//!
//! Lives outside `crate::am::*` because both `ec_hnsw` and `ec_diskann` reach for
//! the same page-layout, item-pointer, and GenericXLog helpers. AM-specific
//! tuple codecs (e.g. `TqElementTuple`) stay under their owning AM.

pub mod page;
pub(crate) mod relation_guard;
pub(crate) mod slot_guard;
pub(crate) mod snapshot_guard;
pub mod wal;
