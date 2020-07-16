#![feature(proc_macro_hygiene)]
#![allow(unused_imports)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(unused_unsafe)]

use skyline::nro::{self, NroInfo};
mod L_Cancels;
mod get_param;
mod utils;

fn nro_main(nro: &NroInfo) {
    match nro.name {
        "common" => {
            skyline::install_hooks!(
                L_Cancels::status_attackair_hook,
                L_Cancels::status_landing_hook
            );
        }
        _ => (),
    }
}

#[skyline::main(name = "L-Cancels")]
pub fn main() {
    skyline::install_hook!(get_param::get_param_float_hook);
    nro::add_hook(nro_main).unwrap();
}