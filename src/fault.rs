use std::sync::atomic::{AtomicI64, Ordering};

use pgrx::{GucContext, GucFlags, GucRegistry, GucSetting};

static FAULT_PALLOC_AFTER_GUC: GucSetting<i32> = GucSetting::<i32>::new(-1);
static FAULT_PALLOC_COUNTER: AtomicI64 = AtomicI64::new(0);

pub(crate) fn register_gucs() {
    GucRegistry::define_int_guc(
        c"ecaz.fault_palloc_after",
        c"Fail the Nth instrumented ECAZ palloc site.",
        c"Task 38 live-fault knob. -1 disables injection; positive values raise ERROR before the Nth instrumented ECAZ palloc site in the backend.",
        &FAULT_PALLOC_AFTER_GUC,
        -1,
        i32::MAX,
        GucContext::Userset,
        GucFlags::default(),
    );
}

pub(crate) fn reset_palloc_counter() {
    FAULT_PALLOC_COUNTER.store(0, Ordering::SeqCst);
}

pub(crate) fn maybe_fail_palloc(label: &str) {
    if cfg!(test) {
        return;
    }
    let after = FAULT_PALLOC_AFTER_GUC.get();
    if after <= 0 {
        return;
    }
    let count = FAULT_PALLOC_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
    if count >= i64::from(after) {
        pgrx::error!("ecaz fault injection palloc failure at {label} (allocation #{count})");
    }
}
