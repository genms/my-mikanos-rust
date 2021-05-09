#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(asm)]

mod asm;
mod console;
mod driver;
mod error;
mod font;
mod frame_buffer_config;
mod graphics;
mod hankaku;
mod logger;
mod mouse;
mod pci;
mod utils;

use bit_field::BitField;
use core::panic::PanicInfo;
use cty::uint64_t;
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use log::{Level, LevelFilter};

use font::*;
use frame_buffer_config::FrameBufferConfig;
use graphics::*;
use logger::Logger;

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        hlt()
    }
}

fn hlt() {
    unsafe {
        asm!("hlt");
    }
}

#[macro_export]
macro_rules! printk {
    ($($x:expr),*) => {
        {
            use core::fmt::Write;

            let mut buf = [0u8; 1024];
            write!($crate::utils::fmt::Wrapper::new(&mut buf), $($x),*).unwrap();
            $crate::_printk(&buf);
        }
    };
}

fn _printk(buf: &[u8]) {
    let txt = core::str::from_utf8(buf).unwrap();
    let console = unsafe { CONSOLE.as_mut().unwrap() };
    console.put_string(txt);
}

extern "C" fn mouse_observer(displacement_x: i8, displacement_y: i8) {
    let mouse_cursor = unsafe { MOUSE_CURSOR.as_mut().unwrap() };
    mouse_cursor.move_relative(&Vector2D::new(displacement_x as i32, displacement_y as i32));
}

fn switch_ehci_to_xhci(xhc_dev: &pci::Device) {
    let intel_ehc_exist = || -> bool {
        for i in unsafe { 0..pci::NUM_DEVICE } {
            let device = unsafe { &pci::DEVICES[i] };
            if device.class_code.match_interface(0x0c, 0x03, 0x20) /* EHCI */ &&
                            0x8086 == pci::read_vendor_id_from_dev(device)
            {
                return true;
            }
        }
        false
    }();
    if !intel_ehc_exist {
        return;
    }

    let superspeed_ports = pci::read_conf_reg(xhc_dev, 0xdc); // USB3PRM
    pci::write_conf_reg(xhc_dev, 0xd8, superspeed_ports); // USB3_PSSEN
    let ehci_to_xhci_ports = pci::read_conf_reg(xhc_dev, 0xd4); // XUSB2PRM
    pci::write_conf_reg(xhc_dev, 0xd0, ehci_to_xhci_ports); // XUSB2PR
    debug!(
        "switch_ehci_to_xhci: SS = {:02}, xHCI = {:02x}\n",
        superspeed_ports, ehci_to_xhci_ports
    );
}

const DESKTOP_BG_COLOR: PixelColor = PixelColor::new(45, 118, 237);
const DESKTOP_FG_COLOR: PixelColor = PixelColor::new(255, 255, 255);

static mut LOGGER: Logger = Logger::new(Level::Debug);
static mut PIXEL_WRITER: Option<PixelWriter> = None;
static mut CONSOLE: Option<console::Console> = None;
static mut MOUSE_CURSOR: Option<mouse::MouseCursor> = None;

#[no_mangle]
pub extern "C" fn KernelMain(fb_config: &'static FrameBufferConfig) -> ! {
    unsafe {
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(LevelFilter::Trace))
            .unwrap();

        PIXEL_WRITER = Some(PixelWriter::new(fb_config));
        let pixel_writer = PIXEL_WRITER.as_ref().unwrap();

        CONSOLE = Some(console::Console::new(
            pixel_writer,
            DESKTOP_FG_COLOR,
            DESKTOP_BG_COLOR,
        ));
        MOUSE_CURSOR = Some(mouse::MouseCursor::new(
            pixel_writer,
            DESKTOP_BG_COLOR,
            Vector2D::new(400, 300),
        ));
    }

    let pixel_writer = unsafe { PIXEL_WRITER.as_ref().unwrap() };

    let frame_width = fb_config.horizontal_resolution() as i32;
    let frame_height = fb_config.vertical_resolution() as i32;
    fill_rectangle(
        pixel_writer,
        &Vector2D::new(0, 0),
        &Vector2D::new(frame_width, frame_height - 50),
        &DESKTOP_BG_COLOR,
    );
    fill_rectangle(
        pixel_writer,
        &Vector2D::new(0, frame_height - 50),
        &Vector2D::new(frame_width, 50),
        &PixelColor::new(1, 8, 17),
    );
    fill_rectangle(
        pixel_writer,
        &Vector2D::new(0, frame_height - 50),
        &Vector2D::new(frame_width / 5, 50),
        &PixelColor::new(80, 80, 80),
    );
    draw_rectangle(
        pixel_writer,
        &Vector2D::new(10, frame_height - 40),
        &Vector2D::new(30, 30),
        &PixelColor::new(160, 160, 160),
    );
    write_string(
        pixel_writer,
        660,
        566,
        "my-mikanos-rust",
        &PixelColor::new(160, 160, 160),
    );

    let mouse_cursor = unsafe { MOUSE_CURSOR.as_mut().unwrap() };
    mouse_cursor.refresh();

    printk!("Welcome to MikanOS in Rust!\n");

    match pci::scan_all_bus() {
        Ok(()) => debug!("scan_all_bus: Ok\n"),
        Err(err) => debug!("scan_all_bus: {}\n", err),
    };

    for i in unsafe { 0..pci::NUM_DEVICE } {
        let dev = unsafe { &pci::DEVICES[i] };
        let vendor_id = pci::read_vendor_id(dev.bus, dev.device, dev.function);
        let class_code = pci::read_class_code(dev.bus, dev.device, dev.function);
        debug!(
            "{}.{}.{}: vend {:04x}, class {}, head {:02x}\n",
            dev.bus, dev.device, dev.function, vendor_id, class_code, dev.header_type
        );
    }

    // Intel 製を優先して xHC を探す
    let xhc_dev = || -> Option<&pci::Device> {
        let mut ret = None;
        for i in unsafe { 0..pci::NUM_DEVICE } {
            let device = unsafe { &pci::DEVICES[i] };
            if device.class_code.match_interface(0x0c, 0x03, 0x30) {
                ret = Some(device);

                if 0x8086 == pci::read_vendor_id_from_dev(device) {
                    return ret;
                }
            }
        }
        ret
    }();
    if xhc_dev.is_none() {
        loop {
            hlt()
        }
    }
    let xhc_dev = xhc_dev.unwrap();
    info!(
        "xHC has been found: {}.{}.{}\n",
        xhc_dev.bus, xhc_dev.device, xhc_dev.function,
    );

    let xhc_bar = pci::read_bar(xhc_dev, 0);
    if let Err(e) = xhc_bar {
        debug!("read_bar: {}\n", e);
        loop {
            hlt()
        }
    }
    debug!("read_bar: Ok\n");

    let xhc_bar = xhc_bar.unwrap();
    let mut xhc_mmio_base = xhc_bar;
    xhc_mmio_base.set_bits(0..=3, 0);
    debug!("xHC mmio_base = {:08x}\n", xhc_mmio_base);

    if 0x8086 == pci::read_vendor_id_from_dev(xhc_dev) {
        switch_ehci_to_xhci(xhc_dev);
    }

    unsafe {
        driver::SetLogLevel(driver::LogLevel::kDebug);

        let xhc_handle = driver::UsbInitXhc(xhc_mmio_base as uint64_t);
        driver::print_log();

        driver::UsbConfigurePort(xhc_handle, mouse_observer);
        driver::print_log();

        driver::SetLogLevel(driver::LogLevel::kWarn);
        loop {
            driver::UsbReceiveEvent(xhc_handle);
            driver::print_log();
        }
    }
}
