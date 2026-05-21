use std::{ffi::CStr, marker::PhantomData, ptr::NonNull};

use pgrx::{itemptr::item_pointer_set_all, pg_sys};

use crate::storage::page::ItemPointer;

pub(crate) struct TupleSlotReader<'slot> {
    slot: NonNull<pg_sys::TupleTableSlot>,
    am_name: &'static str,
    _slot: PhantomData<&'slot mut pg_sys::TupleTableSlot>,
}

impl<'slot> TupleSlotReader<'slot> {
    pub(crate) unsafe fn from_raw_slot(
        slot: *mut pg_sys::TupleTableSlot,
        am_name: &'static str,
    ) -> Result<Self, String> {
        let slot = NonNull::new(slot)
            .ok_or_else(|| format!("{am_name} slot reader received a null tuple slot"))?;
        Ok(Self {
            slot,
            am_name,
            _slot: PhantomData,
        })
    }

    pub(crate) fn clear(&mut self) {
        // SAFETY: construction requires a live TupleTableSlot owned by the
        // caller for this callback scope.
        unsafe { pg_sys::ExecClearTuple(self.slot.as_ptr()) };
    }

    pub(crate) fn required_datum(
        &mut self,
        attnum: i32,
        label: &str,
    ) -> Result<pg_sys::Datum, String> {
        let attr_index = usize::try_from(attnum - 1)
            .map_err(|_| format!("{} heap attribute number must be positive", self.am_name))?;

        // SAFETY: construction requires a live slot, and `attnum` names an
        // indexed/source attribute resolved from relation metadata. The block
        // materializes through `attnum` before reading the value/null arrays.
        unsafe {
            let slot = self.slot.as_ptr();
            if (*slot).tts_nvalid < attnum as i16 {
                pg_sys::slot_getsomeattrs_int(slot, attnum);
            }
            if *(*slot).tts_isnull.add(attr_index) {
                return Err(format!("{} does not support NULL {label}", self.am_name));
            }
            Ok(*(*slot).tts_values.add(attr_index))
        }
    }
}

pub(crate) struct HeapSlotReader<'slot> {
    heap_relation: pg_sys::Relation,
    snapshot: pg_sys::Snapshot,
    slot: TupleSlotReader<'slot>,
}

impl<'slot> HeapSlotReader<'slot> {
    pub(crate) unsafe fn from_raw(
        heap_relation: pg_sys::Relation,
        snapshot: pg_sys::Snapshot,
        slot: *mut pg_sys::TupleTableSlot,
        am_name: &'static str,
    ) -> Result<Self, String> {
        if heap_relation.is_null() {
            return Err(format!(
                "{am_name} heap slot reader received a null heap relation"
            ));
        }
        if snapshot.is_null() {
            return Err(format!(
                "{am_name} heap slot reader received a null snapshot"
            ));
        }
        let slot = unsafe { TupleSlotReader::from_raw_slot(slot, am_name)? };
        Ok(Self {
            heap_relation,
            snapshot,
            slot,
        })
    }

    pub(crate) fn clear(&mut self) {
        self.slot.clear();
    }

    pub(crate) fn fetch_row_version(&mut self, heap_tid: ItemPointer) -> Result<bool, String> {
        let mut tid = pg_sys::ItemPointerData::default();
        item_pointer_set_all(&mut tid, heap_tid.block_number, heap_tid.offset_number);
        self.clear();
        // SAFETY: construction binds a live heap relation, snapshot, and tuple
        // slot. The TID was initialized from the index candidate being fetched.
        Ok(unsafe {
            pg_sys::table_tuple_fetch_row_version(
                self.heap_relation,
                &mut tid,
                self.snapshot,
                self.slot.slot.as_ptr(),
            )
        })
    }

    pub(crate) fn required_datum(
        &mut self,
        attnum: i32,
        label: &str,
    ) -> Result<pg_sys::Datum, String> {
        self.slot.required_datum(attnum, label)
    }
}

pub(crate) struct TupleSlotAttribute {
    pub(crate) attnum: pg_sys::AttrNumber,
    pub(crate) name: String,
    pub(crate) typid: pg_sys::Oid,
    pub(crate) typmod: i32,
}

pub(crate) struct TupleDescView<'desc> {
    tuple_desc: NonNull<pg_sys::TupleDescData>,
    natts: std::ffi::c_int,
    context: &'static str,
    _desc: PhantomData<&'desc pg_sys::TupleDescData>,
}

impl<'desc> TupleDescView<'desc> {
    pub(crate) unsafe fn from_raw(
        tuple_desc: pg_sys::TupleDesc,
        context: &'static str,
    ) -> Result<Self, String> {
        let tuple_desc = NonNull::new(tuple_desc)
            .ok_or_else(|| format!("{context} tuple descriptor is null"))?;
        // SAFETY: construction requires a live PostgreSQL tuple descriptor for
        // the returned view's borrow scope.
        let natts = unsafe { tuple_desc.as_ref().natts };
        Ok(Self {
            tuple_desc,
            natts,
            context,
            _desc: PhantomData,
        })
    }

    pub(crate) fn as_ptr(&self) -> pg_sys::TupleDesc {
        self.tuple_desc.as_ptr()
    }

    pub(crate) fn natts(&self) -> std::ffi::c_int {
        self.natts
    }

    pub(crate) fn attribute(
        &self,
        attr_index: std::ffi::c_int,
    ) -> Result<Option<TupleSlotAttribute>, String> {
        // SAFETY: callers iterate `attr_index` in `0..self.natts()`, and
        // construction validates the tuple descriptor pointer. PostgreSQL owns
        // the attribute descriptor and fixed-size NameData storage.
        unsafe {
            let attr = pg_sys::TupleDescAttr(self.as_ptr(), attr_index);
            if attr.is_null() {
                return Ok(None);
            }
            let attr = &*attr;
            if attr.attisdropped {
                return Ok(None);
            }
            let name = CStr::from_ptr(attr.attname.data.as_ptr())
                .to_str()
                .map_err(|_| format!("{} relation attribute name is not UTF-8", self.context))?
                .to_owned();
            Ok(Some(TupleSlotAttribute {
                attnum: attr.attnum,
                name,
                typid: attr.atttypid,
                typmod: attr.atttypmod,
            }))
        }
    }
}

pub(crate) struct TupleSlotWriter<'slot> {
    slot: NonNull<pg_sys::TupleTableSlot>,
    tuple_desc: TupleDescView<'slot>,
    context: &'static str,
    _slot: PhantomData<&'slot mut pg_sys::TupleTableSlot>,
}

impl<'slot> TupleSlotWriter<'slot> {
    pub(crate) unsafe fn from_raw_slot(
        slot: *mut pg_sys::TupleTableSlot,
        context: &'static str,
    ) -> Result<Self, String> {
        let slot =
            NonNull::new(slot).ok_or_else(|| format!("{context} tuple payload slot is null"))?;
        // SAFETY: construction requires a live TupleTableSlot for the callback
        // scope; the tuple descriptor pointer is owned by PostgreSQL.
        let tuple_desc = unsafe {
            let tuple_desc = slot.as_ref().tts_tupleDescriptor;
            TupleDescView::from_raw(tuple_desc, context)?
        };
        Ok(Self {
            slot,
            tuple_desc,
            context,
            _slot: PhantomData,
        })
    }

    pub(crate) fn tuple_desc_view(&self) -> &TupleDescView<'slot> {
        &self.tuple_desc
    }

    pub(crate) fn natts(&self) -> std::ffi::c_int {
        self.tuple_desc.natts()
    }

    pub(crate) fn validate_input_width(&self, width: usize) -> Result<(), String> {
        if width != usize::try_from(self.natts()).unwrap_or(usize::MAX) {
            return Err(format!(
                "{context} tuple payload input cache width mismatch",
                context = self.context
            ));
        }
        Ok(())
    }

    pub(crate) fn clear(&mut self) {
        // SAFETY: construction requires a live slot for this callback scope.
        unsafe { pg_sys::ExecClearTuple(self.slot.as_ptr()) };
    }

    pub(crate) fn attribute(
        &self,
        attr_index: std::ffi::c_int,
    ) -> Result<Option<TupleSlotAttribute>, String> {
        self.tuple_desc.attribute(attr_index)
    }

    pub(crate) fn set_null(&mut self, attr_index: std::ffi::c_int) {
        self.set_value(attr_index, true, pg_sys::Datum::from(0));
    }

    pub(crate) fn set_datum(&mut self, attr_index: std::ffi::c_int, datum: pg_sys::Datum) {
        self.set_value(attr_index, false, datum);
    }

    fn set_value(&mut self, attr_index: std::ffi::c_int, is_null: bool, datum: pg_sys::Datum) {
        let attr_index = attr_index as usize;
        // SAFETY: callers iterate `attr_index` in `0..self.natts()`, so the
        // slot value/null arrays contain this entry.
        unsafe {
            *self.slot.as_ref().tts_isnull.add(attr_index) = is_null;
            *self.slot.as_ref().tts_values.add(attr_index) = datum;
        }
    }

    pub(crate) fn store_virtual_tuple(mut self) -> Result<*mut pg_sys::TupleTableSlot, String> {
        let natts = self.natts();
        let nvalid = i16::try_from(natts)
            .map_err(|_| format!("{} tuple descriptor too wide", self.context))?;
        // SAFETY: construction validates the live slot pointer and this writer
        // has populated the virtual slot arrays through `natts`.
        unsafe {
            self.slot.as_mut().tts_nvalid = nvalid;
            Ok(pg_sys::ExecStoreVirtualTuple(self.slot.as_ptr()))
        }
    }
}
