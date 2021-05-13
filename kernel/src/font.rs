//! フォント描画のプログラムを集めたファイル.
use bit_field::BitField;
use core::str;

use crate::graphics;
use crate::hankaku;

pub fn write_ascii(
    writer: &graphics::PixelWriter,
    x: i32,
    y: i32,
    c: char,
    color: &graphics::PixelColor,
) {
    let font = match hankaku::get_font_slice(c) {
        Ok(ptr) => ptr,
        Err(_) => {
            return;
        }
    };

    for (dy, row) in font.iter().enumerate() {
        for dx in 0..=7 {
            if row.get_bit(7 - dx) {
                writer.write(x + dx as i32, y + dy as i32, color);
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

pub fn write_bytes(
    writer: &graphics::PixelWriter,
    x: i32,
    y: i32,
    bytes: &[u8],
    color: &graphics::PixelColor,
) {
    let s = str::from_utf8(bytes).unwrap_or("?\n");
    write_string(writer, x, y, s, color);
}
