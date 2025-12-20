use wasm_bindgen::prelude::*;


#[allow(unused_macros)]
macro_rules! dbg {
    ($($arg:tt)*) => {
        crate::logging::log(format!("[DEBUG][{}:{}] {}", file!(), line!(), format!($($arg)*)))
    };
}

#[allow(unused_macros)]
macro_rules! warn {
    ($($arg:tt)*) => {
        crate::logging::log_cold(format!("[WARNING][{}:{}] {}", file!(), line!(), format!($($arg)*)))
    };
}

#[allow(unused_macros)]
macro_rules! log_err {
    ($err:expr) => {
        crate::logging::log_cold(format!("{}", $err));
    };
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: String);
}

#[cold]
#[inline(never)]
pub fn log_cold(message: String) {
    crate::logging::log(format!("{}", message));
}

pub fn panic_handler(panic_info: &::std::panic::PanicHookInfo) {
    dbg!("A panic occured!");

    if let Some(location) = panic_info.location() {
        dbg!("{:?}", location);
    }

    if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
        dbg!("Panic details: {s:?}");
    } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
        dbg!("Panic details: {s:?}");
    } else {
        dbg!("panic occurred");
    }
}
