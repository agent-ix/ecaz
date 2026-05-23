//! Typed wrappers for PostgreSQL datum / varlena / flat-float4 array
//! boundaries shared across the HNSW, IVF, DiskANN, and SPIRE AMs.
//!
//! Program P6 of `reviews/task-50/030-comprehensive-unsafe-burndown-plan`:
//! "Datum, Varlena, Vector, And Quantized Payload Contracts" — encapsulates
//! the `FromDatum` / detoast / `pg_sys::ArrayType` header / `from_raw_parts`
//! data-slice pattern into typed wrappers so consumer call sites become safe.
//!
//! Each wrapper records its PG-datum lifetime invariant at its constructor.
//! The construction site is `unsafe` (it asserts the datum is a live, typed
//! varlena owned by the surrounding PG arena scope); the read methods on the
//! resulting wrapper are safe and encapsulate the underlying PG primitive.

use std::{ffi::c_int, marker::PhantomData};

use pgrx::pg_sys;

use super::detoast::DetoastedVarlena;

/// Datum kinds that [`FlatFloat4Source`] knows how to absorb.
///
/// This mirrors the per-AM `SourceDatumKind` enums (HNSW today, IVF / DiskANN
/// / SPIRE later) so callers can dispatch between the array-typed and
/// byte-backed varlena variants through a single typed wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FlatFloat4Kind {
    /// PostgreSQL `real[]` (`FLOAT4OID`) one-dimensional array varlena.
    RealArray,
    /// Byte-backed `bytea` / `ecvector` varlena, treated as a packed
    /// `[f32]` payload.
    Varlena,
}

/// Typed wrapper over the `Datum -> &[f32]` extraction used by every AM that
/// scores against a flat float4 source vector.
///
/// Absorbs both array-typed (`real[]`) and byte-backed (`bytea` / `ecvector`)
/// varlenas behind a single safe `as_slice()` accessor. The wrapper owns the
/// detoasted backing storage and ties its slice lifetime to `'a` so the
/// borrowed view cannot escape the surrounding PG-arena scope.
pub(crate) struct FlatFloat4Source<'a> {
    _detoasted: DetoastedVarlena,
    data_ptr: *const f32,
    len: usize,
    _marker: PhantomData<&'a [f32]>,
}

impl<'a> FlatFloat4Source<'a> {
    /// Construct a typed flat-float4 view over `datum`.
    ///
    /// Returns `None` if `datum` is SQL NULL. Otherwise dispatches on `kind`
    /// to either the `pg_sys::ArrayType` header / dims / data-offset path or
    /// the byte-aligned varlena path, validating element type, dimension
    /// count, NULL-element absence (array path), and `f32` alignment.
    ///
    /// # Safety
    /// `datum` must be a live PostgreSQL Datum that was type-checked to match
    /// `kind` before dispatch, and whose backing varlena outlives `'a`. The
    /// returned wrapper owns the detoasted copy (when one was produced) and
    /// releases it via [`DetoastedVarlena`]'s Drop.
    pub(crate) unsafe fn from_datum(
        datum: pg_sys::Datum,
        kind: FlatFloat4Kind,
        label: &str,
    ) -> Option<Self> {
        if datum.is_null() {
            return None;
        }

        // SAFETY: caller asserts `datum` is a live varlena Datum.
        let detoasted = unsafe { DetoastedVarlena::plain_from_datum(datum) }
            .unwrap_or_else(|| pgrx::error!("flat float4 source could not detoast {label}"));

        let (data_ptr, len) = match kind {
            FlatFloat4Kind::RealArray => {
                // SAFETY: detoasted storage is held alive by `detoasted` for
                // the borrow lifetime returned to the caller.
                unsafe { Self::extract_array(&detoasted, label) }
            }
            FlatFloat4Kind::Varlena => {
                // SAFETY: detoasted storage is held alive by `detoasted`.
                unsafe { Self::extract_varlena(&detoasted, label) }
            }
        };

        Some(Self {
            _detoasted: detoasted,
            data_ptr,
            len,
            _marker: PhantomData,
        })
    }

    /// Decode the array-typed (`real[]`) variant.
    ///
    /// # Safety
    /// `detoasted` must wrap a live one-dimensional `FLOAT4OID` ArrayType.
    unsafe fn extract_array(detoasted: &DetoastedVarlena, label: &str) -> (*const f32, usize) {
        let array_ptr = detoasted.as_ptr().cast::<pg_sys::ArrayType>();

        // SAFETY: `array_ptr` points at the detoasted ArrayType backing
        // storage, held alive by `detoasted` for the duration of this borrow.
        let array_header = unsafe { &*array_ptr };
        let ndim = match usize::try_from(array_header.ndim) {
            Ok(value) => value,
            Err(_) => pgrx::error!("flat float4 source {label} must be a one-dimensional real[]"),
        };
        if ndim != 1 {
            pgrx::error!("flat float4 source {label} must be a one-dimensional real[]");
        }
        if array_header.elemtype != pg_sys::FLOAT4OID {
            pgrx::error!("flat float4 source {label} must be a real[]");
        }
        // SAFETY: `array_ptr` is a valid detoasted ArrayType.
        if unsafe { pg_sys::array_contains_nulls(array_ptr) } {
            pgrx::error!("flat float4 source {label} arrays must not contain NULL elements");
        }

        // SAFETY: detoasted one-dimensional flat ArrayType.
        let dims_ptr = unsafe { flat_array_dims_ptr(array_ptr) };
        // SAFETY: `ndim` and `dims_ptr` come from the same ArrayType header.
        let len = usize::try_from(unsafe { pg_sys::ArrayGetNItems(array_header.ndim, dims_ptr) })
            .expect("flat float4 array length should fit in usize");
        // SAFETY: Data offset is computed from the same flat ArrayType header;
        // alignment is checked before exposing the f32 slice.
        let data_ptr = unsafe {
            array_ptr
                .cast::<u8>()
                .add(flat_array_data_offset(array_ptr, ndim))
                .cast::<f32>()
        };
        if (data_ptr as usize) % std::mem::align_of::<f32>() != 0 {
            pgrx::error!(
                "flat float4 source {label} data pointer is not aligned for float4 access"
            );
        }

        (data_ptr, len)
    }

    /// Decode the byte-backed (`bytea` / `ecvector`) variant.
    ///
    /// # Safety
    /// `detoasted` must wrap a live varlena whose bytes are a packed `[f32]`.
    unsafe fn extract_varlena(detoasted: &DetoastedVarlena, label: &str) -> (*const f32, usize) {
        let bytes = detoasted.as_bytes();
        if bytes.len() % std::mem::size_of::<f32>() != 0 {
            pgrx::error!(
                "flat float4 source {label} bytea payload length must be a multiple of 4 bytes"
            );
        }
        // SAFETY: `align_to` is used only to validate exact f32 alignment;
        // any non-empty prefix/suffix is rejected before the body is stored.
        let (prefix, body, suffix) = unsafe { bytes.align_to::<f32>() };
        if !prefix.is_empty() || !suffix.is_empty() {
            pgrx::error!(
                "flat float4 source {label} bytea payload is not aligned for float4 access"
            );
        }
        (body.as_ptr(), body.len())
    }

    /// Borrow the underlying flat float4 payload as a safe slice.
    pub(crate) fn as_slice(&self) -> &[f32] {
        // SAFETY: `data_ptr` and `len` were validated during construction; the
        // detoasted backing storage is owned by `self` for the returned
        // slice's borrow lifetime.
        unsafe { std::slice::from_raw_parts(self.data_ptr, self.len) }
    }

    /// Number of `f32` elements in the payload.
    pub(crate) fn len(&self) -> usize {
        self.len
    }

    /// Alias for [`Self::len`] — the dimensionality of the vector payload.
    pub(crate) fn dims(&self) -> usize {
        self.len
    }
}

/// Typed wrapper over the `Datum -> EcVector` extraction used by HNSW
/// `source.rs`, IVF `scan.rs`, DiskANN `insert.rs`, and SPIRE `insert.rs`.
///
/// Today this is a thin shim over [`FlatFloat4Source`] in `Varlena` mode,
/// since the `ecvector` payload is a byte-backed packed `[f32]`. When a
/// first-class `EcVector` / `EcVectorView` type lands (per the Task 53 spec),
/// this wrapper is the single seam to swap.
///
/// TODO(slice-003): wire to the actual `EcVector` / `EcVectorView` types once
/// they exist. For now `view()` returns the underlying flat float4 slice via
/// [`EcVectorView`].
pub(crate) struct EcVectorDatum<'a> {
    source: FlatFloat4Source<'a>,
}

/// Safe borrowed view returned by [`EcVectorDatum::view`].
///
/// Today this is a slice view; when a first-class `EcVector` type lands it
/// will gain the typed accessors that `EcVector` exposes.
pub(crate) struct EcVectorView<'a> {
    data: &'a [f32],
}

impl<'a> EcVectorView<'a> {
    /// Borrow the underlying flat float4 payload.
    pub(crate) fn as_slice(&self) -> &'a [f32] {
        self.data
    }

    /// Dimensionality of the vector.
    pub(crate) fn dims(&self) -> usize {
        self.data.len()
    }
}

impl<'a> EcVectorDatum<'a> {
    /// Construct a typed ecvector view over `datum`.
    ///
    /// Returns `None` if `datum` is SQL NULL.
    ///
    /// # Safety
    /// `datum` must be a live PostgreSQL Datum that was type-checked as
    /// `ecvector` (a byte-backed varlena packed `[f32]`) before dispatch,
    /// and whose backing varlena outlives `'a`.
    pub(crate) unsafe fn from_datum(datum: pg_sys::Datum, label: &str) -> Option<Self> {
        // SAFETY: caller asserts `datum` is a live ecvector Datum.
        let source =
            unsafe { FlatFloat4Source::from_datum(datum, FlatFloat4Kind::Varlena, label) }?;
        Some(Self { source })
    }

    /// Borrow the typed view. Does the FromDatum + detoast + flat-array
    /// boundary in one place; the returned [`EcVectorView`] is safe.
    pub(crate) fn view(&self) -> EcVectorView<'_> {
        EcVectorView {
            data: self.source.as_slice(),
        }
    }
}

/// Safe encapsulation of the `pg_sys::get_attnum` catalog lookup boundary.
///
/// PostgreSQL's `get_attnum` extern takes a relation OID and a NUL-terminated
/// C string and returns the attribute number (or `InvalidAttrNumber`).
/// Consumers across the four AMs reach for the same `unsafe { pg_sys::...
/// (*heap_relation).rd_id, cstr.as_ptr() }` pattern; this wrapper lifts that
/// to a single safe call site.
pub(crate) struct AttnumLookup;

impl AttnumLookup {
    /// Look up the attribute number of `attname` on `rel`.
    ///
    /// Returns `None` if either input is invalid (null relation, NUL-bearing
    /// column name) or PostgreSQL reports the attribute is absent / dropped
    /// (`<= 0`). The successful return is the positive 1-based attribute
    /// number reported by PostgreSQL.
    pub(crate) fn lookup(rel: pg_sys::Relation, attname: &str) -> Option<pg_sys::AttrNumber> {
        if rel.is_null() {
            return None;
        }
        let attname = std::ffi::CString::new(attname).ok()?;
        // SAFETY: `rel` is non-null per the guard above; `attname` is a
        // freshly-allocated CString that lives across the extern call. The
        // PostgreSQL relation is owned by the caller's callback scope.
        let attnum = unsafe { pg_sys::get_attnum((*rel).rd_id, attname.as_ptr()) };
        if attnum <= 0 {
            None
        } else {
            Some(attnum)
        }
    }
}

/// Pointer to the dims[] array immediately following an `ArrayType` header.
///
/// # Safety
/// `array_ptr` must point at a detoasted flat ArrayType. Array dims follow
/// the fixed ArrayType header in PostgreSQL's on-disk layout.
unsafe fn flat_array_dims_ptr(array_ptr: *const pg_sys::ArrayType) -> *const c_int {
    // SAFETY: see fn-level contract.
    unsafe {
        array_ptr
            .cast::<u8>()
            .add(std::mem::size_of::<pg_sys::ArrayType>())
            .cast::<c_int>()
    }
}

fn maxaligned_size(len: usize) -> usize {
    let align =
        usize::try_from(pg_sys::MAXIMUM_ALIGNOF).expect("MAXIMUM_ALIGNOF should fit in usize");
    (len + align - 1) & !(align - 1)
}

/// Offset (in bytes) from the start of a flat ArrayType to its data payload.
///
/// # Safety
/// `array_ptr` must point at a detoasted flat ArrayType whose `dataoffset`
/// header field is valid.
unsafe fn flat_array_data_offset(array_ptr: *const pg_sys::ArrayType, ndim: usize) -> usize {
    // SAFETY: see fn-level contract.
    let dataoffset = unsafe { (*array_ptr).dataoffset };
    if dataoffset != 0 {
        usize::try_from(dataoffset).expect("flat float4 array dataoffset should fit in usize")
    } else {
        maxaligned_size(
            std::mem::size_of::<pg_sys::ArrayType>() + (2 * ndim * std::mem::size_of::<c_int>()),
        )
    }
}
