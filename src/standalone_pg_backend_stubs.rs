use std::ffi::{c_char, CStr};

#[link(name = "ecaz_pg_test_stubs", kind = "static")]
unsafe extern "C" {
    fn ecaz_test_pg_backend_stubs_anchor();
}

#[used]
static STUBS_ANCHOR: unsafe extern "C" fn() = ecaz_test_pg_backend_stubs_anchor;

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn ecaz_test_pg_backend_panic(message: *const c_char) -> ! {
    let message = if message.is_null() {
        "Postgres ERROR".to_owned()
    } else {
        // SAFETY: The C test stub passes a null-terminated error message pointer
        // or null; the null case is handled above before constructing `CStr`.
        unsafe { CStr::from_ptr(message) }
            .to_string_lossy()
            .into_owned()
    };
    std::panic::panic_any(message);
}
