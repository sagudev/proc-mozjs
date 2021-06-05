extern crate proc_macro2;
extern crate quote;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate wasm_bindgen_backend as backend;
extern crate wasm_bindgen_shared as shared;

pub use crate::parser::BindgenAttrs;
use crate::parser::MacroParse;
use backend::{Diagnostic, TryToTokens};
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::TokenStreamExt;
use syn::parse::{Parse, ParseStream, Result as SynResult};

mod parser;

/// Takes the parsed input from a `#[mozjs]` macro and returns the generated bindings
pub fn expand(attr: TokenStream, input: TokenStream) -> Result<TokenStream, Diagnostic> {
    parser::reset_attrs_used();
    let item = syn::parse2::<syn::Item>(input)?;
    let opts = syn::parse2(attr)?;

    let mut tokens = proc_macro2::TokenStream::new();
    let mut program = backend::ast::Program::default();
    item.macro_parse(&mut program, (Some(opts), &mut tokens))?;
    program.try_to_tokens(&mut tokens)?;

    // If we successfully got here then we should have used up all attributes
    // and considered all of them to see if they were used. If one was forgotten
    // that's a bug on our end, so sanity check here.
    parser::assert_all_attrs_checked();

    Ok(tokens)
}

/// Takes the parsed input from a `#[mozjs]` macro and returns the generated bindings
pub fn expand_class_marker(
    attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, Diagnostic> {
    parser::reset_attrs_used();
    let mut item = syn::parse2::<syn::ImplItemMethod>(input)?;
    let opts: ClassMarker = syn::parse2(attr)?;

    let mut program = backend::ast::Program::default();
    item.macro_parse(&mut program, (&opts.class, &opts.js_class))?;
    parser::assert_all_attrs_checked(); // same as above

    // This is where things are slightly different, we are being expanded in the
    // context of an impl so we can't inject arbitrary item-like tokens into the
    // output stream. If we were to do that then it wouldn't parse!
    //
    // Instead what we want to do is to generate the tokens for `program` into
    // the header of the function. This'll inject some no_mangle functions and
    // statics and such, and they should all be valid in the context of the
    // start of a function.
    //
    // We manually implement `ToTokens for ImplItemMethod` here, injecting our
    // program's tokens before the actual method's inner body tokens.
    let mut tokens = proc_macro2::TokenStream::new();
    tokens.append_all(item.attrs.iter().filter(|attr| match attr.style {
        syn::AttrStyle::Outer => true,
        _ => false,
    }));
    item.vis.to_tokens(&mut tokens);
    item.sig.to_tokens(&mut tokens);
    let mut err = None;
    item.block.brace_token.surround(&mut tokens, |tokens| {
        if let Err(e) = program.try_to_tokens(tokens) {
            err = Some(e);
        }
        tokens.append_all(item.attrs.iter().filter(|attr| match attr.style {
            syn::AttrStyle::Inner(_) => true,
            _ => false,
        }));
        tokens.append_all(&item.block.stmts);
    });

    if let Some(err) = err {
        return Err(err);
    }

    Ok(tokens)
}

struct ClassMarker {
    class: syn::Ident,
    js_class: String,
}

impl Parse for ClassMarker {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let class = input.parse::<syn::Ident>()?;
        input.parse::<Token![=]>()?;
        let js_class = input.parse::<syn::LitStr>()?.value();
        Ok(ClassMarker { class, js_class })
    }
}


// Option types (represents js default value when none) must be at end
/* #[proc_macro_attribute]
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
    // name of function in string
    let f_name_str = f_name.to_string();
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
                    format!("{} requires exactly {} argument", #f_name_str, #f_nargs).as_ptr() as *const libc::c_char,
                );
                false
            }
        }
    };

    println!("This is what compiler gets: {}", quote!(#f #iitem));
    // ouput mozjs wrapper function and original as-is
    TokenStream::from(quote!(#f #iitem))
} */