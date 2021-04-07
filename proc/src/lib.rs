extern crate proc_macro2;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, Attribute, FnArg, Ident, ItemFn, PatType, ReturnType};

/// Option types must be at end
/// apply only  on standalone functions
#[proc_macro_attribute]
pub fn jsfn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let iitem = proc_macro2::TokenStream::from(item.clone());
    if !attr.is_empty() {
        panic!("#[jsfn] takes no arguments");
    }
    //println!("{:#?}", function);
    let function = parse_macro_input!(item as ItemFn);
    println!("{:#?}", function);
    // count nonoptional args
    let f_args = function.sig.inputs.len() as u32;
    let f_name = function.sig.ident;
    let f_ident = format_ident!("____to_be_mozjs_{}", f_name);
    let f = quote! {
        use mozjs::conversions::FromJSValConvertible;
        use mozjs::conversions::ToJSValConvertible;
        unsafe extern "C" fn #f_ident(context: *mut ::mozjs::jsapi::JSContext, argc: u32, vp: *mut ::mozjs::jsapi::Value) -> bool {
            let args = ::mozjs::jsapi::CallArgs::from_vp(vp, argc);
            args.rval().set(::mozjs::jsval::UndefinedValue());

            if args.argc_ == #f_args {
                #f_name(String::from_jsval(context, ::mozjs::rust::Handle::from_raw(args.get(0)),
                ()).unwrap().get_success_value().unwrap());
                true
            } else {
                ::mozjs::jsapi::JS_ReportErrorASCII(
                    context,
                    b"#f_name() requires exactly #f_args argument\0".as_ptr() as *const libc::c_char,
                );
                false
            }
        }
    };

    // Work around https://github.com/rust-lang/rust/issues/46489
    //let attributes: TokenStream = attributes.to_string().parse().unwrap();
    //f.sig.unsafety
    //let item = proc_macro2::TokenStream::from(item);

    /* let input = proc_macro2::TokenStream::from(item);

    let output: proc_macro2::TokenStream = { /* transform input */ };

    TokenStream::from(output) */
    TokenStream::from(quote!(#f #iitem))
}
