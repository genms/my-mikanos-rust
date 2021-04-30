use crate::graphics;
use crate::hankaku;

fn get_font(c: char) -> Result<*mut u8, char> {
    unsafe {
        let index = 16 * c as usize;
        if index >= &hankaku::_binary_hankaku_bin_size as *const u8 as usize {
            return Err(c);
        }
        let start = &hankaku::_binary_hankaku_bin_start as *const u8 as *mut u8;
        Ok(start.offset(index as isize))
    }
}

fn get_font_slice(c: char) -> Result<&'static [u8; 16], char> {
    let font = get_font(c)?;
    Ok(unsafe { &*(font as *const [u8; 16]) })
}

pub fn write_ascii(
    writer: &graphics::PixelWriter,
    x: i32,
    y: i32,
    c: char,
    color: &graphics::PixelColor,
) {
    let font = match get_font_slice(c) {
        Ok(ptr) => ptr,
        Err(_) => {
            return;
        }
    };

    for (dy, row) in font.iter().enumerate() {
        for dx in 0..8i32 {
            if (row << dx) & 0x80u8 != 0 {
                writer.write(x + dx, y + dy as i32, color);
            }
        }
    }
}

pub fn write_string(
    writer: &graphics::PixelWriter,
    x: i32,
    y: i32,
    s: &str,
    color: &graphics::PixelColor,
) {
    for (i, c) in s.chars().enumerate() {
        write_ascii(writer, x + 8 * i as i32, y, c, color);
    }
}
