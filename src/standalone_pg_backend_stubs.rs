use std::ffi::{CStr, c_char};

#[link(name = "tqvector_pg_test_stubs", kind = "static")]
unsafe extern "C" {
    fn tqvector_test_pg_backend_stubs_anchor();
}

#[used]
static STUBS_ANCHOR: unsafe extern "C" fn() = tqvector_test_pg_backend_stubs_anchor;

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn tqvector_test_pg_backend_panic(message: *const c_char) -> ! {
    let message = if message.is_null() {
        "Postgres ERROR".to_owned()
    } else {
        unsafe { CStr::from_ptr(message) }
            .to_string_lossy()
            .into_owned()
    };
    std::panic::panic_any(message);
}
