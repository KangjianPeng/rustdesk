use cocoa::{
    base::{id, NO},
    foundation::{NSArray, NSURL},
};
use objc::runtime::{Object, Sel, BOOL};
use std::ffi::CStr;

#[no_mangle]
extern "C" fn handle_open_urls(_: &Object, _: Sel, _: id, urls: id) {
    unsafe {
        for i in 0..urls.count() {
            let url = CStr::from_ptr(urls.objectAtIndex(i).absoluteString().UTF8String())
                .to_string_lossy()
                .into_owned();
            hbb_common::log::debug!("URL received: {}", url);
            std::thread::spawn(move || crate::handle_url_scheme(url));
        }
    }
}

#[no_mangle]
extern "C" fn service_should_handle_reopen(_: &Object, _: Sel, _: id, _: BOOL) -> BOOL {
    hbb_common::log::debug!("Invoking the main rustdesk process");
    std::thread::spawn(|| crate::handle_url_scheme(String::new()));
    NO
}
