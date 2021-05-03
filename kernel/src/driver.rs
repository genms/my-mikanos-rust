use crate::printk;
use cstr_core::{c_char, CStr};

type MouseObserverFn = extern "C" fn(i8, i8);
type XhcHandle = i32;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[repr(C)]
pub enum LogLevel {
    kError = 3,
    kWarn = 4,
    kInfo = 6,
    kDebug = 7,
}

extern "C" {
    pub fn SetLogLevel(level: LogLevel);
    pub fn UsbInitXhc(xhc_mmio_base: u64) -> XhcHandle;
    pub fn UsbConfigurePort(xhc_handle: XhcHandle, mouse_observer: MouseObserverFn);
    pub fn UsbReceiveEvent(xhc_handle: XhcHandle);

    pub fn GetLog() -> *const c_char;
    pub fn ClearLog();
}

pub fn print_log() {
    unsafe {
        let s = CStr::from_ptr(GetLog()).to_str().unwrap();
        if s.len() > 0 {
            printk!("{}", s);
            ClearLog();
        }
    }
}
