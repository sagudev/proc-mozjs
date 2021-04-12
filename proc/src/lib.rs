extern crate proc_macro2;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, Attribute, FnArg, FnArg::Typed, Ident, ItemFn, PatType,
    ReturnType, Type, TypeReference,
};
use Type::Reference;

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
    //println!("{:#?}", function);
    // count nonoptional args
    let f_nargs = function.sig.inputs.len() as u32;
    // iter over args
    let mut f_iter_narg = Vec::new();
    let mut f_iter_targ = Vec::new();
    for (i, input) in function.sig.inputs.iter().enumerate() {
        match input {
            FnArg::Typed(PatType { ty, .. }) => {
                f_iter_narg.push(i);
                f_iter_targ.push(ty.clone());
            }
            FnArg::Receiver(_) => {
                panic!("This fuction should not have self!")
            }
        }
    }
    // name of function
    let f_name = function.sig.ident;
    // name of internal function which will be binded to mozjs
    let f_mozjs_name = format_ident!("____to_be_mozjs_{}", f_name);
    // generate internal wrapper function
    // TODO: Handle Result as in https://github.com/Redfire75369/spiderfire/blob/feature/js_fn_macro/ion/src/functions/macros.rs#L16
    // TODO: Handle any result
    let f = quote! {
        unsafe extern "C" fn #f_mozjs_name(context: *mut ::mozjs::jsapi::JSContext, argc: u32, vp: *mut ::mozjs::jsapi::Value) -> bool {
            let args = ::mozjs::jsapi::CallArgs::from_vp(vp, argc);
            args.rval().set(::mozjs::jsval::UndefinedValue());

            if args.argc_ == #f_nargs {
                #f_name( #( <#f_iter_targ as ::mozjs::conversions::FromJSValConvertible>::from_jsval(context, ::mozjs::rust::Handle::from_raw(args.get(#f_iter_narg as u32)),()).unwrap().get_success_value().unwrap().to_owned() ),* );
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

    println!("This is what compiler gets: {}", quote!(#f #iitem));
    // ouput mozjs wrapper function and original as-is
    TokenStream::from(quote!(#f #iitem))
}
