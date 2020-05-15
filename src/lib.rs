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


#[skyline::main(name = "L-Cancels")]
pub fn main() {
    println!("Hello from L-Cancels plugin!");
    L_Cancels::function_hooks();
    get_param::get_param_function_hooks();
}