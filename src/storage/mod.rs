//! Cross-AM physical storage primitives.
//!
//! Lives outside `crate::am::*` because both `tqhnsw` and `ecdiskann` reach for
//! the same page-layout, item-pointer, and GenericXLog helpers. AM-specific
//! tuple codecs (e.g. `TqElementTuple`) stay under their owning AM.

pub mod page;
pub mod wal;
