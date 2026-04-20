use std::ffi::c_void;
use std::mem::size_of;

use pgrx::pg_sys;

const EC_PARALLEL_SCAN_STATE_MAGIC: u32 = u32::from_le_bytes(*b"ECPR");
const EC_PARALLEL_SCAN_STATE_VERSION: u16 = 1;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct EcParallelScanState {
    magic: u32,
    version: u16,
    flags: u16,
    descriptor_bytes: pg_sys::Size,
    rescan_epoch: u32,
    reserved_worker_slots: u32,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct ParallelScanAttachment {
    pub(crate) state: *mut EcParallelScanState,
    pub(crate) descriptor_bytes: pg_sys::Size,
    pub(crate) rescan_epoch: u32,
}

fn maxalign(size: pg_sys::Size) -> pg_sys::Size {
    let align = size_of::<usize>();
    debug_assert!(align.is_power_of_two());
    (size + align - 1) & !(align - 1)
}

pub(crate) fn ec_parallel_scan_state_size() -> pg_sys::Size {
    maxalign(size_of::<EcParallelScanState>())
}

fn initialize_parallel_scan_state(state: &mut EcParallelScanState) {
    *state = EcParallelScanState {
        magic: EC_PARALLEL_SCAN_STATE_MAGIC,
        version: EC_PARALLEL_SCAN_STATE_VERSION,
        flags: 0,
        descriptor_bytes: ec_parallel_scan_state_size(),
        rescan_epoch: 0,
        reserved_worker_slots: 0,
    };
}

unsafe fn validate_parallel_scan_state(
    state: *mut EcParallelScanState,
) -> Result<ParallelScanAttachment, &'static str> {
    if state.is_null() {
        return Err("AM-private parallel scan state pointer was null");
    }

    let state_ref = unsafe { &*state };
    if state_ref.magic != EC_PARALLEL_SCAN_STATE_MAGIC {
        return Err("AM-private parallel scan state magic was not initialized");
    }
    if state_ref.version != EC_PARALLEL_SCAN_STATE_VERSION {
        return Err("AM-private parallel scan state version was not initialized");
    }
    if state_ref.descriptor_bytes < ec_parallel_scan_state_size() {
        return Err("AM-private parallel scan state size was smaller than the shared header");
    }

    Ok(ParallelScanAttachment {
        state,
        descriptor_bytes: state_ref.descriptor_bytes,
        rescan_epoch: state_ref.rescan_epoch,
    })
}

#[cfg(feature = "pg17")]
unsafe fn parallel_scan_state_ptr(
    parallel_scan: pg_sys::ParallelIndexScanDesc,
) -> Result<Option<*mut EcParallelScanState>, &'static str> {
    if parallel_scan.is_null() {
        return Ok(None);
    }
    let offset = unsafe { (*parallel_scan).ps_offset };
    if offset == 0 {
        return Ok(None);
    }
    Ok(Some(
        unsafe { parallel_scan.cast::<u8>().add(offset) }.cast::<EcParallelScanState>(),
    ))
}

#[cfg(feature = "pg18")]
unsafe fn parallel_scan_state_ptr(
    parallel_scan: pg_sys::ParallelIndexScanDesc,
) -> Result<Option<*mut EcParallelScanState>, &'static str> {
    if parallel_scan.is_null() {
        return Ok(None);
    }
    let offset = unsafe { (*parallel_scan).ps_offset_am };
    if offset == 0 {
        return Ok(None);
    }
    Ok(Some(
        unsafe { parallel_scan.cast::<u8>().add(offset) }.cast::<EcParallelScanState>(),
    ))
}

pub(crate) unsafe fn parallel_scan_attachment(
    parallel_scan: pg_sys::ParallelIndexScanDesc,
) -> Result<Option<ParallelScanAttachment>, &'static str> {
    let Some(state) = (unsafe { parallel_scan_state_ptr(parallel_scan) })? else {
        return Ok(None);
    };
    Ok(Some(unsafe { validate_parallel_scan_state(state) }?))
}

pub(crate) unsafe fn initialize_parallel_scan_target(
    target: *mut c_void,
) -> Result<(), &'static str> {
    if target.is_null() {
        return Err("AM-private parallel scan target was null");
    }
    unsafe { initialize_parallel_scan_state(&mut *target.cast::<EcParallelScanState>()) };
    Ok(())
}

pub(crate) unsafe fn reset_parallel_scan_state(
    parallel_scan: pg_sys::ParallelIndexScanDesc,
) -> Result<Option<u32>, &'static str> {
    let Some(state) = (unsafe { parallel_scan_state_ptr(parallel_scan) })? else {
        return Ok(None);
    };
    let state_ref = unsafe { &mut *state };
    if state_ref.magic != EC_PARALLEL_SCAN_STATE_MAGIC
        || state_ref.version != EC_PARALLEL_SCAN_STATE_VERSION
    {
        return Err("AM-private parallel scan state was not initialized before rescan");
    }
    state_ref.rescan_epoch = state_ref.rescan_epoch.wrapping_add(1);
    Ok(Some(state_ref.rescan_epoch))
}

#[cfg(feature = "pg17")]
pub(crate) unsafe extern "C-unwind" fn ec_amestimateparallelscan(
    _nkeys: std::ffi::c_int,
    _norderbys: std::ffi::c_int,
) -> pg_sys::Size {
    ec_parallel_scan_state_size()
}

#[cfg(feature = "pg18")]
pub(crate) unsafe extern "C-unwind" fn ec_amestimateparallelscan(
    _index_relation: pg_sys::Relation,
    _nkeys: std::ffi::c_int,
    _norderbys: std::ffi::c_int,
) -> pg_sys::Size {
    ec_parallel_scan_state_size()
}

pub(crate) unsafe extern "C-unwind" fn ec_aminitparallelscan(target: *mut c_void) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            initialize_parallel_scan_target(target)
                .unwrap_or_else(|err| pgrx::error!("ec_hnsw parallel scan init failed: {err}"));
        })
    }
}

pub(crate) unsafe extern "C-unwind" fn ec_amparallelrescan(scan: pg_sys::IndexScanDesc) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                return;
            }
            reset_parallel_scan_state((*scan).parallel_scan)
                .unwrap_or_else(|err| pgrx::error!("ec_hnsw parallel scan rescan failed: {err}"));
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;
    use std::ptr;

    #[repr(C, align(8))]
    struct TestParallelScanStorage {
        bytes: [u8; 256],
    }

    impl Default for TestParallelScanStorage {
        fn default() -> Self {
            Self { bytes: [0; 256] }
        }
    }

    const TEST_PARALLEL_SCAN_OFFSET: usize = 64;

    unsafe fn test_parallel_scan_desc(
        storage: &mut TestParallelScanStorage,
    ) -> pg_sys::ParallelIndexScanDesc {
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        {
            unsafe { (*parallel_scan).ps_offset = TEST_PARALLEL_SCAN_OFFSET };
        }
        #[cfg(feature = "pg18")]
        {
            unsafe { (*parallel_scan).ps_offset_am = TEST_PARALLEL_SCAN_OFFSET };
        }
        parallel_scan
    }

    unsafe fn test_parallel_scan_target(storage: &mut TestParallelScanStorage) -> *mut c_void {
        unsafe { storage.bytes.as_mut_ptr().add(TEST_PARALLEL_SCAN_OFFSET) }.cast::<c_void>()
    }

    #[test]
    fn ec_parallel_scan_state_size_is_maxaligned() {
        let descriptor_bytes = ec_parallel_scan_state_size();
        assert!(
            descriptor_bytes >= size_of::<EcParallelScanState>(),
            "shared parallel scan descriptor must cover the common header"
        );
        assert_eq!(
            descriptor_bytes % size_of::<usize>(),
            0,
            "shared parallel scan descriptor must stay MAXALIGN-sized"
        );
    }

    #[test]
    fn initialize_parallel_scan_target_round_trips_through_attachment() {
        let mut storage = TestParallelScanStorage::default();
        let parallel_scan = unsafe { test_parallel_scan_desc(&mut storage) };
        let target = unsafe { test_parallel_scan_target(&mut storage) };

        unsafe { initialize_parallel_scan_target(target) }.expect("parallel target should init");
        let attachment = unsafe { parallel_scan_attachment(parallel_scan) }
            .expect("parallel descriptor should validate")
            .expect("parallel descriptor should expose AM state");

        assert!(
            ptr::eq(attachment.state.cast::<c_void>(), target),
            "attachment should point at the AM-private target that init populated"
        );
        assert_eq!(
            attachment.descriptor_bytes,
            ec_parallel_scan_state_size(),
            "attachment should report the initialized descriptor size"
        );
        assert_eq!(
            attachment.rescan_epoch, 0,
            "freshly initialized parallel scan state should start at epoch zero"
        );
    }

    #[test]
    fn reset_parallel_scan_state_advances_rescan_epoch() {
        let mut storage = TestParallelScanStorage::default();
        let parallel_scan = unsafe { test_parallel_scan_desc(&mut storage) };
        let target = unsafe { test_parallel_scan_target(&mut storage) };

        unsafe { initialize_parallel_scan_target(target) }.expect("parallel target should init");
        assert_eq!(
            unsafe { reset_parallel_scan_state(parallel_scan) }
                .expect("parallel rescan should succeed")
                .expect("parallel rescan should see the initialized state"),
            1,
            "first rescan should advance the shared epoch to one"
        );
        assert_eq!(
            unsafe { reset_parallel_scan_state(parallel_scan) }
                .expect("parallel rescan should keep succeeding")
                .expect("parallel rescan should keep using the initialized state"),
            2,
            "each rescan should advance the shared epoch once"
        );
    }

    #[test]
    fn parallel_scan_attachment_rejects_uninitialized_state() {
        let mut storage = TestParallelScanStorage::default();
        let parallel_scan = unsafe { test_parallel_scan_desc(&mut storage) };

        let err = unsafe { parallel_scan_attachment(parallel_scan) }
            .expect_err("attachment should reject uninitialized AM-private state");
        assert!(
            err.contains("magic"),
            "uninitialized shared state should fail the magic check first"
        );
    }
}
