#[macro_use]
extern crate mozjs;
extern crate libc;

use proc_mozjs::{bindfn, jsfn};

// here we import only engine parts
use mozjs::jsapi::JSAutoRealm;
use mozjs::jsapi::JS_DefineFunction;
use mozjs::jsapi::JS_NewGlobalObject;
use mozjs::jsapi::OnNewGlobalHookOption;
use mozjs::jsval::UndefinedValue;
use mozjs::rust::{JSEngine, RealmOptions, Runtime, SIMPLE_GLOBAL_CLASS};

use std::ptr;

#[test]
fn multiarg() {
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
            // Some(____to_be_mozjs_puts),
            bindfn!(puts),
            1,
            0,
        );
        assert!(!function.is_null());
        let javascript = "puts('Test Iñtërnâtiônàlizætiøn ┬─┬ノ( º _ ºノ) ', false);";
        rooted!(in(context) let mut rval = UndefinedValue());
        let _ = runtime.evaluate_script(global, javascript, "test.js", 0, rval.handle_mut());
    }
}

#[jsfn]
fn puts(s: String, new_line: bool, _o: Option<bool>) {
    assert_eq!(s, "Test Iñtërnâtiônàlizætiøn ┬─┬ノ( º _ ºノ) ");
    if new_line {
        println!("{}", s);
    } else {
        print!("{}", s);
    }
}
