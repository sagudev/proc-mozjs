extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn mozjs(attr: TokenStream, input: TokenStream) -> TokenStream {
    match macro_support::expand(attr.into(), input.into()) {
        Ok(tokens) => {
            if cfg!(feature = "__dbg") {
                println!("{}", tokens);
            }
            tokens.into()
        }
        Err(diagnostic) => (quote! { #diagnostic }).into(),
    }
}