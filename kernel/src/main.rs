#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(asm)]

mod asm;
mod console;
mod font;
mod frame_buffer_config;
mod graphics;
mod hankaku;
mod logger;
mod mouse;
mod pci;
mod utils;

use core::panic::PanicInfo;
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use log::{Level, LevelFilter};

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
            write!($crate::utils::fmt::Wrapper::new(&mut buf), $($x),*).expect("printk!");
            let txt = core::str::from_utf8(&buf).unwrap();
            unsafe {
                $crate::CONSOLE.put_string(txt);
            }
        }
    };
}

const DESKTOP_BG_COLOR: PixelColor = PixelColor::new(45, 118, 237);
const DESKTOP_FG_COLOR: PixelColor = PixelColor::new(255, 255, 255);

static mut LOGGER: Logger = Logger::new(Level::Info);
static mut FRAME_BUFFER_CONFIG: Option<&'static FrameBufferConfig> = None;
static mut PIXEL_WRITER: Option<&dyn PixelWriter> = None;
static mut CONSOLE: console::Console = console::Console::new(DESKTOP_FG_COLOR, DESKTOP_BG_COLOR);
static mut MOUSE_CURSOR: mouse::MouseCursor =
    mouse::MouseCursor::new(DESKTOP_BG_COLOR, Vector2D::new(400, 300));

#[no_mangle]
pub extern "C" fn KernelMain(frame_buffer_config: &'static FrameBufferConfig) -> ! {
    unsafe {
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(LevelFilter::Trace))
            .unwrap();

        FRAME_BUFFER_CONFIG = Some(frame_buffer_config);
        PIXEL_WRITER = match frame_buffer_config.pixel_format {
            frame_buffer_config::PixelFormat::PixelRGBResv8BitPerColor => {
                Some(&RGBResv8BitPerColorPixelWriter {})
            }
            frame_buffer_config::PixelFormat::PixelBGRResv8BitPerColor => {
                Some(&BGRResv8BitPerColorPixelWriter {})
            }
        };
    }

    let pixel_writer = unsafe { PIXEL_WRITER.unwrap() };

    let frame_width = frame_buffer_config.horizontal_resolution as i32;
    let frame_height = frame_buffer_config.vertical_resolution as i32;
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

    unsafe {
        MOUSE_CURSOR.refresh();
    }

    printk!("Welcome to MikanOS in Rust!\n");

    match pci::scan_all_bus() {
        Ok(()) => info!("scan_all_bus: Ok\n"),
        Err(err) => info!("scan_all_bus: {}\n", err),
    };

    for i in unsafe { 0..pci::NUM_DEVICE } {
        let dev = unsafe { pci::DEVICES[i] };
        let vendor_id = pci::read_vendor_id(dev.bus, dev.device, dev.function);
        let class_code = pci::read_class_code(dev.bus, dev.device, dev.function);
        info!(
            "{}.{}.{}: vend {:04x}, class {:08x}, head {:02x}\n",
            dev.bus, dev.device, dev.function, vendor_id, class_code, dev.header_type
        );
    }

    loop {
        hlt()
    }
}
