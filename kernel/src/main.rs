#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(asm)]

use core::panic::PanicInfo;
use core::str;
use core::fmt::Write;

mod console;
mod font;
mod frame_buffer_config;
mod graphics;
mod hankaku;
mod utils;

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

static mut FRAME_BUFFER_CONFIG: Option<&'static frame_buffer_config::FrameBufferConfig> = None;
static mut PIXEL_WRITER: Option<&dyn graphics::PixelWriter> = None;

#[no_mangle]
pub extern "C" fn KernelMain(
    frame_buffer_config: &'static frame_buffer_config::FrameBufferConfig,
) -> ! {
    unsafe {
        FRAME_BUFFER_CONFIG = Some(frame_buffer_config);
        PIXEL_WRITER = match frame_buffer_config.pixel_format {
            frame_buffer_config::PixelFormat::PixelRGBResv8BitPerColor => {
                Some(&graphics::RGBResv8BitPerColorPixelWriter {})
            }
            frame_buffer_config::PixelFormat::PixelBGRResv8BitPerColor => {
                Some(&graphics::BGRResv8BitPerColorPixelWriter {})
            }
        };
    }

    let pixel_writer = unsafe { PIXEL_WRITER.unwrap() };

    let bg_color = graphics::PixelColor {
        r: 255,
        g: 255,
        b: 255,
    };
    for x in 0..frame_buffer_config.horizontal_resolution as i32 {
        for y in 0..frame_buffer_config.vertial_resolution as i32 {
            pixel_writer.write(x, y, &bg_color);
        }
    }

    let font_color = graphics::PixelColor { r: 0, g: 0, b: 0 };
    let mut console = console::Console::new(pixel_writer, &font_color, &bg_color);

    for i in 0..27 {
        let mut buf = [0 as u8; 128];
        write!(Wrapper::new(&mut buf), "line {}\n", i).expect("write!");
        let txt = str::from_utf8(&buf).unwrap();
        console.put_string(txt);
    }

    loop {
        hlt()
    }
}
