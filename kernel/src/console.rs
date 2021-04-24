use core::str;

use crate::graphics;
use crate::font;

pub struct Console<'a> {
    writer: &'a dyn graphics::PixelWriter,
    fg_color: &'a graphics::PixelColor,
    bg_color: &'a graphics::PixelColor,
    buffer: [[u8; (Console::COLUMNS + 1) as usize]; Console::ROWS as usize],
    cursor_row: i32,
    cursor_column: i32,
}

impl<'a> Console<'a> {
    pub const ROWS: i32 = 25;
    pub const COLUMNS: i32 = 80;

    pub fn new(
        writer: &'static dyn graphics::PixelWriter,
        fg_color: &'a graphics::PixelColor,
        bg_color: &'a graphics::PixelColor,
    ) -> Self {
        Console {
            writer,
            fg_color,
            bg_color,
            buffer: [[0; (Console::COLUMNS + 1) as usize]; Console::ROWS as usize],
            cursor_row: 0,
            cursor_column: 0,
        }
    }

    pub fn put_string(&mut self, s: &str) {
        for byte in s.bytes() {
            if byte == '\0' as u8 {
                break;
            } else if byte == '\n' as u8 {
                self.newline();
            } else if self.cursor_column < Self::COLUMNS - 1 {
                font::write_ascii(
                    self.writer,
                    8 * self.cursor_column,
                    16 * self.cursor_row,
                    byte as char,
                    self.fg_color,
                );
                self.buffer[self.cursor_row as usize][self.cursor_column as usize] = byte;
                self.cursor_column += 1;
            }
        }
    }

    fn newline(&mut self) {
        self.cursor_column = 0;
        if self.cursor_row < Self::ROWS - 1 {
            self.cursor_row += 1;
        } else {
            for y in 0..(16 * Self::ROWS) {
                for x in 0..(8 * Self::COLUMNS) {
                    self.writer.write(x, y, self.bg_color);
                }
            }
            for row in 0..(Self::ROWS - 1) {
                let rng = ((row + 1) as usize)..((row + 2) as usize);
                self.buffer.copy_within(rng, row as usize);
                let txt = str::from_utf8(&self.buffer[row as usize]).unwrap();
                font::write_string(self.writer, 0, 16 * row, txt, self.fg_color);
            }
            let buffer_last_row = &mut self.buffer[(Self::ROWS - 1) as usize];
            buffer_last_row.fill(0);
        }
    }
}
