//! ec_spire access-method scaffold.

mod assign;
mod build;
mod cost;
mod insert;
mod meta;
mod routine;
mod scan;
mod storage;
mod update;
mod vacuum;

pub(crate) fn register_gucs() {}

fn not_implemented(callback: &str) -> ! {
    pgrx::error!("ec_spire {callback} is not implemented yet")
}
