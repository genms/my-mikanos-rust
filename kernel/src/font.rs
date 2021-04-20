use crate::graphics;
use crate::hankaku;

pub unsafe fn get_font(c: char) -> Result<*mut u8, &'static str> {
    let index = 16 * c as usize;
    if index >= &hankaku::_binary_hankaku_bin_size as *const u8 as usize {
        return Err("get_font");
    }
    let start = &hankaku::_binary_hankaku_bin_start as *const u8 as *mut u8;
    Ok(start.offset(index as isize))
}

pub fn write_ascii(
    writer: &dyn graphics::PixelWriter,
    x: i32,
    y: i32,
    c: char,
    color: &graphics::PixelColor,
) {
    let font = unsafe {
        match get_font(c) {
            Ok(ptr) => ptr,
            Err(_) => {
                return;
            }
        }
    };

    for dy in 0..16i32 {
        for dx in 0..8i32 {
            let row = unsafe { *(font.offset(dy as isize)) };
            if (row << dx) & 0x80u8 != 0 {
                writer.write(x + dx, y + dy, color);
            }
        }
    }
}

pub fn write_string(writer: &dyn graphics::PixelWriter, x: i32, y: i32, s: &str, color: &graphics::PixelColor) {
    for (i, c) in s.chars().enumerate() {
        write_ascii(writer, x + 8 * i as i32, y, c, color);
    }
}
