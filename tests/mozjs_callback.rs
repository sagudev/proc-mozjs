/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

/* Clone from https://github.com/servo/rust-mozjs/blob/master/tests/callback.rs */

#[macro_use]
extern crate mozjs;
extern crate libc;

use mozjs::glue::EncodeStringToUTF8;
use mozjs::jsapi::CallArgs;
use mozjs::jsapi::JSAutoRealm;
use mozjs::jsapi::JSContext;
use mozjs::jsapi::JS_DefineFunction;
use mozjs::jsapi::JS_NewGlobalObject;
use mozjs::jsapi::JS_ReportErrorASCII;
use mozjs::jsapi::OnNewGlobalHookOption;
use mozjs::jsapi::Value;
use mozjs::jsval::UndefinedValue;
use mozjs::rust::{JSEngine, RealmOptions, Runtime, SIMPLE_GLOBAL_CLASS};

use std::ffi::CStr;
use std::ptr;
use std::str;

#[test]
fn callback() {
    let engine = JSEngine::init().unwrap();
    let runtime = Runtime::new(engine.handle());
    let context = runtime.cx();
    let h_option = OnNewGlobalHookOption::FireOnNewGlobalHook;
    let c_option = RealmOptions::default();

    unsafe {
        let global = JS_NewGlobalObject(
            context,
            &SIMPLE_GLOBAL_CLASS,
            ptr::null_mut(),
            h_option,
            &*c_option,
        );
        rooted!(in(context) let global_root = global);
        let global = global_root.handle();
        let _ac = JSAutoRealm::new(context, global.get());
        let function = JS_DefineFunction(
            context,
            global.into(),
            b"puts\0".as_ptr() as *const libc::c_char,
            Some(puts),
            1,
            0,
        );
        assert!(!function.is_null());
        let javascript = "puts('Test Iñtërnâtiônàlizætiøn ┬─┬ノ( º _ ºノ) ');";
        rooted!(in(context) let mut rval = UndefinedValue());
        let _ = runtime.evaluate_script(global, javascript, "test.js", 0, rval.handle_mut());
    }
}

unsafe extern "C" fn puts(context: *mut JSContext, argc: u32, vp: *mut Value) -> bool {
    let args = CallArgs::from_vp(vp, argc);
    args.rval().set(UndefinedValue());

    if args.argc_ == 1 {
        let arg = mozjs::rust::Handle::from_raw(args.get(0));
        let js = mozjs::rust::ToString(context, arg);
        rooted!(in(context) let message_root = js);
        EncodeStringToUTF8(context, message_root.handle().into(), |message| {
            let message = CStr::from_ptr(message);
            let message = str::from_utf8(message.to_bytes()).unwrap();
            assert_eq!(message, "Test Iñtërnâtiônàlizætiøn ┬─┬ノ( º _ ºノ) ");
            println!("{}", message);
        });
        true
    } else {
        JS_ReportErrorASCII(
            context,
            b"puts() requires exactly 1 argument\0".as_ptr() as *const libc::c_char,
        );
        false
    }
}
