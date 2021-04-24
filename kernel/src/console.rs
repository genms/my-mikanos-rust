use core::str;

use crate::font;
use crate::graphics;

pub struct Console {
    fg_color: graphics::PixelColor,
    bg_color: graphics::PixelColor,
    buffer: [[u8; Console::COLUMNS + 1]; Console::ROWS],
    cursor_row: usize,
    cursor_column: usize,
}

impl Console {
    pub const ROWS: usize = 25;
    pub const COLUMNS: usize = 80;

    pub const fn new(fg_color: graphics::PixelColor, bg_color: graphics::PixelColor) -> Self {
        Console {
            fg_color,
            bg_color,
            buffer: [[0; Console::COLUMNS + 1]; Console::ROWS],
            cursor_row: 0,
            cursor_column: 0,
        }
    }

    pub fn put_string(&mut self, s: &str) {
        let pixel_writer = unsafe { crate::PIXEL_WRITER.unwrap() };

        for byte in s.bytes() {
            if byte == '\0' as u8 {
                break;
            } else if byte == '\n' as u8 {
                self.newline();
            } else if self.cursor_column < Self::COLUMNS - 1 {
                font::write_ascii(
                    pixel_writer,
                    (8 * self.cursor_column) as i32,
                    (16 * self.cursor_row) as i32,
                    byte as char,
                    &self.fg_color,
                );
                self.buffer[self.cursor_row][self.cursor_column] = byte;
                self.cursor_column += 1;
            }
        }
    }

    fn newline(&mut self) {
        let pixel_writer = unsafe { crate::PIXEL_WRITER.unwrap() };

        self.cursor_column = 0;
        if self.cursor_row < Self::ROWS - 1 {
            self.cursor_row += 1;
        } else {
            for y in 0..(16 * Self::ROWS) {
                for x in 0..(8 * Self::COLUMNS) {
                    pixel_writer.write(x as i32, y as i32, &self.bg_color);
                }
            }
            for row in 0..(Self::ROWS - 1) {
                let rng = (row + 1)..(row + 2);
                self.buffer.copy_within(rng, row);
                let txt = str::from_utf8(&self.buffer[row]).unwrap();
                font::write_string(pixel_writer, 0, (16 * row) as i32, txt, &self.fg_color);
            }
            let buffer_last_row = &mut self.buffer[Self::ROWS - 1];
            buffer_last_row.fill(0);
        }
    }
}
