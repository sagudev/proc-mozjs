extern crate proc_macro2;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, Attribute, FnArg, Ident, ItemFn, PatType, ReturnType};

/// Option types (represents js default value when none) must be at end
#[proc_macro_attribute]
pub fn jsfn(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        panic!("#[jsfn] takes no arguments");
    }
    // clone original function as we keep it as-is
    let iitem = proc_macro2::TokenStream::from(item.clone());
    //parse as function
    let function = parse_macro_input!(item as ItemFn);
    // debug
    println!("{:#?}", function);
    // count nonoptional args
    let f_nargs = function.sig.inputs.len() as u32;
    // name of function
    let f_name = function.sig.ident;
    // name of internal function which will be binded to mozjs
    let f_mozjs_name = format_ident!("____to_be_mozjs_{}", f_name);
    // generate internal wrapper function
    let f = quote! {
        use mozjs::conversions::FromJSValConvertible;
        use mozjs::conversions::ToJSValConvertible;
        unsafe extern "C" fn #f_mozjs_name(context: *mut ::mozjs::jsapi::JSContext, argc: u32, vp: *mut ::mozjs::jsapi::Value) -> bool {
            let args = ::mozjs::jsapi::CallArgs::from_vp(vp, argc);
            args.rval().set(::mozjs::jsval::UndefinedValue());

            if args.argc_ == #f_nargs {
                #f_name(String::from_jsval(context, ::mozjs::rust::Handle::from_raw(args.get(0)),
                ()).unwrap().get_success_value().unwrap());
                true
            } else {
                ::mozjs::jsapi::JS_ReportErrorASCII(
                    context,
                    b"#f_name() requires exactly #f_nargs argument\0".as_ptr() as *const libc::c_char,
                );
                false
            }
        }
    };

    // ouput mozjs wrapper function and original as-is
    TokenStream::from(quote!(#f #iitem))
}
