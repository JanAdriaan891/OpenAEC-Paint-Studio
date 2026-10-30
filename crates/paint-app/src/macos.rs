//! Finder Open With bridge. winit does not forward Cocoa's openFiles callback.
use objc2::{
    runtime::{AnyClass, AnyObject, Sel},
    sel, MainThreadMarker,
};
use objc2_app_kit::{NSApplication, NSApplicationDelegateReply};
use objc2_foundation::{NSArray, NSString};
use std::{path::PathBuf, sync::Mutex};
static FILES: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

#[link(name = "objc")]
extern "C" {
    fn class_addMethod(
        class: *const AnyClass,
        selector: Sel,
        implementation: unsafe extern "C" fn(),
        types: *const std::ffi::c_char,
    ) -> i8;
}
unsafe extern "C" fn open_files(
    _: *mut AnyObject,
    _: Sel,
    app: *mut NSApplication,
    files: *mut NSArray<NSString>,
) {
    // Cocoa invokes this method on the main thread with live retained arguments.
    let Some(files) = files.as_ref() else {
        return;
    };
    if let Ok(mut pending) = FILES.lock() {
        pending.extend(files.iter().map(|name| PathBuf::from(name.to_string())));
    }
    if let Some(app) = app.as_ref() {
        app.replyToOpenOrPrint(NSApplicationDelegateReply::Success);
    }
}
pub fn install() {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    let Some(delegate) = app.delegate() else {
        return;
    };
    let object: &AnyObject = (*delegate).as_ref();
    // Add only the missing callback to winit's delegate. Never replace its class,
    // event loop methods or an existing openFiles implementation.
    unsafe {
        let implementation = std::mem::transmute::<
            unsafe extern "C" fn(*mut AnyObject, Sel, *mut NSApplication, *mut NSArray<NSString>),
            unsafe extern "C" fn(),
        >(open_files);
        class_addMethod(
            object.class(),
            sel!(application:openFiles:),
            implementation,
            c"v@:@@".as_ptr(),
        );
    }
}
pub fn take() -> Option<PathBuf> {
    FILES.lock().ok()?.pop()
}
