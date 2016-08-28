#![feature(alloc_system)]

#[macro_use]
extern crate objc;
extern crate alloc_system;
extern crate libc;
extern crate block;

use std::cell::RefCell;

use objc::runtime;
use objc::runtime::Class;

#[cfg(any(target_arch = "x86_64"))]
#[link(name = "trampoline")]
extern "C" {
    fn trampoline();
}

#[no_mangle]
pub unsafe extern "C" fn get_implementation(obj: *const objc::runtime::Object,
                                            sel: *const objc::runtime::Sel)
                                            -> objc::runtime::Imp {
    let class = msg_send![obj, class];
    return class_getMethodImplementation(class, sel);
}

pub fn trampoline_hook(obj: *const objc::runtime::Object) {
    unsafe {
        let cls = Class::get((*obj).class().name()).unwrap();
        // TODO: change to proper name for proxy class
        let proxy = runtime::objc_allocateClassPair(cls, "proxy_NSObject".as_ptr() as *const i8, 0);

        for method in cls.instance_methods().iter() {
            // TODO: filter out NSObject methods since we want to target application methods but
            // they are nifty for testing right now
            let selector: runtime::Sel = method.name();
            let types = method_getTypeEncoding(*method);
            runtime::class_addMethod(proxy, selector, trampoline as unsafe extern "C" fn(), types);
        }

        // TODO: replace initialize?

        let class_method = runtime::class_getInstanceMethod(proxy, sel!(class));
        let proxy_block = block::ConcreteBlock::new(move || cls);
        let proxy_block = proxy_block.copy();
        class_replaceMethod(proxy,
                            sel!(class),
                            imp_implementationWithBlock(proxy_block),
                            method_getTypeEncoding(class_method));

        runtime::objc_registerClassPair(proxy);

        object_setClass(obj, proxy);
    }
}

pub fn trampoline_unhook(obj: *const objc::runtime::Object) {
    unsafe {
        object_setClass(obj, msg_send![obj, class]);
    }
}

thread_local! {
    static EVENTS: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

#[no_mangle]
pub unsafe extern "C" fn trampoline_start(obj: *const objc::runtime::Object,
                                          sel: objc::runtime::Sel) {
    let class: &objc::runtime::Class = msg_send![obj, class];
    let class_name = class.name();
    let sel_name = sel.name();
    let event = format!("-[{} {}]", class_name, sel_name);
    EVENTS.with(|e| {
        (*e.borrow_mut()).push(event);
    });
    println!("trampoline_start: -[{} {}]", class_name, sel_name);
}

#[no_mangle]
pub unsafe extern "C" fn trampoline_end() {
    let event = EVENTS.with(|e| (*e.borrow_mut()).pop());
    if let Some(e) = event {
        println!("trampoline_end: {}", e);
    }
}

#[link(name = "objc", kind = "dylib")]
extern "C" {
    pub fn method_getTypeEncoding(method: *const objc::runtime::Method) -> *const libc::c_char;
    pub fn object_setClass(obj: *const objc::runtime::Object, class: *const objc::runtime::Class);
    pub fn class_replaceMethod(class: *const objc::runtime::Class,
                               sel: objc::runtime::Sel,
                               imp: objc::runtime::Imp,
                               types: *const libc::c_char)
                               -> objc::runtime::Imp;
    pub fn class_getMethodImplementation(obj: *const objc::runtime::Object,
                                         sel: *const objc::runtime::Sel)
                                         -> objc::runtime::Imp;

    // TODO: make generic
    pub fn imp_implementationWithBlock(block: block::RcBlock<(), &objc::runtime::Class>)
                                       -> objc::runtime::Imp;
}

#[cfg(test)]
mod tests {
    use objc::runtime::{Class, Object, BOOL, YES};

    #[test]
    fn it_works() {
        unsafe {
            let cls = Class::get("NSObject").unwrap();
            let obj: *mut Object = msg_send![cls, new];

            super::trampoline_hook(obj);

            let hash: usize = msg_send![obj, hash];
            assert!(hash > 0);
            let is_kind: BOOL = msg_send![obj, isKindOfClass:cls];
            assert!(is_kind == YES);
            // Even void methods must have their return type annotated
            let _: () = msg_send![obj, release];
        }
    }
}
