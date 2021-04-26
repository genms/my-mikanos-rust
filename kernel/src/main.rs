#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(asm)]

use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;

mod asm;
mod console;
mod font;
mod frame_buffer_config;
mod graphics;
mod hankaku;
mod pci;
mod utils;

use frame_buffer_config::FrameBufferConfig;
use graphics::*;
use utils::fmt::Wrapper;

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

macro_rules! printk {
    ($($x:expr),*) => {
        {
            let mut buf = [0u8; 1024];
            write!(Wrapper::new(&mut buf), $($x),*).expect("printk!");
            let txt = str::from_utf8(&buf).unwrap();
            unsafe {
                CONSOLE.put_string(txt);
            }
        }
    };
}

const DESKTOP_BG_COLOR: PixelColor = PixelColor::new(45, 118, 237);
const DESKTOP_FG_COLOR: PixelColor = PixelColor::new(255, 255, 255);

//const MOUSE_CURSOR_WIDTH: i32 = 15;
//const MOUSE_CURSOR_HEIGHT: i32 = 24;
const MOUSE_CURSOR_SHAPE: [&str; 24] = [
    "@              ",
    "@@             ",
    "@.@            ",
    "@..@           ",
    "@...@          ",
    "@....@         ",
    "@.....@        ",
    "@......@       ",
    "@.......@      ",
    "@........@     ",
    "@.........@    ",
    "@..........@   ",
    "@...........@  ",
    "@............@ ",
    "@......@@@@@@@@",
    "@......@       ",
    "@....@@.@      ",
    "@...@ @.@      ",
    "@..@   @.@     ",
    "@.@    @.@     ",
    "@@      @.@    ",
    "@       @.@    ",
    "         @.@   ",
    "         @@@   ",
];

static mut FRAME_BUFFER_CONFIG: Option<&'static FrameBufferConfig> = None;
static mut PIXEL_WRITER: Option<&dyn PixelWriter> = None;

static mut CONSOLE: console::Console = console::Console::new(DESKTOP_FG_COLOR, DESKTOP_BG_COLOR);

#[no_mangle]
pub extern "C" fn KernelMain(frame_buffer_config: &'static FrameBufferConfig) -> ! {
    unsafe {
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

    printk!("Welcome to MikanOS in Rust!\n");

    for (dy, row) in MOUSE_CURSOR_SHAPE.iter().enumerate() {
        for (dx, u) in row.as_bytes().iter().enumerate() {
            match *u as char {
                '@' => {
                    pixel_writer.write(200 + dx as i32, 100 + dy as i32, &PixelColor::new(0, 0, 0));
                }
                '.' => {
                    pixel_writer.write(
                        200 + dx as i32,
                        100 + dy as i32,
                        &PixelColor::new(255, 255, 255),
                    );
                }
                _ => {}
            };
        }
    }

    match pci::scan_all_bus() {
        Ok(()) => printk!("scan_all_bus: Ok\n"),
        Err(err) => printk!("scan_all_bus: {}\n", err),
    };

    for i in unsafe { 0..pci::NUM_DEVICE } {
        let dev = unsafe { pci::DEVICES[i] };
        let vendor_id = pci::read_vendor_id(dev.bus, dev.device, dev.function);
        let class_code = pci::read_class_code(dev.bus, dev.device, dev.function);
        printk!(
            "{}.{}.{}: vend {:04x}, class {:08x}, head {:02x}\n",
            dev.bus,
            dev.device,
            dev.function,
            vendor_id,
            class_code,
            dev.header_type
        );
    }

    loop {
        hlt()
    }
}
