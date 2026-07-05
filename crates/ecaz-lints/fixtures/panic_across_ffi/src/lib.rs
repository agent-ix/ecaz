#![allow(dead_code, improper_ctypes_definitions, unknown_lints)]

unsafe extern "C-unwind" fn unguarded_callback() {
    panic!("boom");
}

unsafe extern "C-unwind" fn guarded_by_pgrx_helper() {
    unsafe { pgrx::pgrx_extern_c_guard(|| panic!("caught by pgrx")) };
}

unsafe extern "C-unwind" fn guarded_by_catch_unwind() {
    let _ = std::panic::catch_unwind(|| panic!("caught locally"));
}

extern "C-unwind" fn pg_finfo_fixture() -> *const () {
    core::ptr::null()
}

mod pgrx {
    pub unsafe fn pgrx_extern_c_guard<F: FnOnce()>(callback: F) {
        callback();
    }
}
