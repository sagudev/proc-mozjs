//#[macro_use] extern crate proc;
// https://github.com/rust-lang/rust/issues/40090
pub use proc::*;

#[macro_export]
macro_rules! jsfnn {
    ( $x:ident ) => {{
        concat_idents::concat_idents!(jsfn = ____to_be_mozjs_, $x {
            Some(jsfn)
        })
    }};
}
