//! コンソール描画のプログラムを集めたファイル．
use crate::font;
use crate::graphics;

pub struct Console<'a> {
    pixel_writer: &'a graphics::PixelWriter,
    fg_color: graphics::PixelColor,
    bg_color: graphics::PixelColor,
    buffer: [[u8; Console::COLUMNS + 1]; Console::ROWS],
    cursor_row: usize,
    cursor_column: usize,
}

impl<'a> Console<'a> {
    pub const ROWS: usize = 25;
    pub const COLUMNS: usize = 80;

    pub fn new(
        pixel_writer: &'a graphics::PixelWriter,
        fg_color: graphics::PixelColor,
        bg_color: graphics::PixelColor,
    ) -> Self {
        Console {
            pixel_writer,
            fg_color,
            bg_color,
            buffer: [[0; Console::COLUMNS + 1]; Console::ROWS],
            cursor_row: 0,
            cursor_column: 0,
        }
    }

    pub fn put_string(&mut self, s: &str) {
        for c in s.chars() {
            if c == '\0' {
                break;
            } else if c == '\n' {
                self.newline();
            } else {
                if self.cursor_column >= Self::COLUMNS - 1 {
                    self.newline();
                }

                font::write_ascii(
                    self.pixel_writer,
                    (8 * self.cursor_column) as i32,
                    (16 * self.cursor_row) as i32,
                    c,
                    &self.fg_color,
                );
                self.buffer[self.cursor_row][self.cursor_column] = c as u8;
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
                    self.pixel_writer.write(x as i32, y as i32, &self.bg_color);
                }
            }
            for row in 0..(Self::ROWS - 1) {
                let rng = (row + 1)..=(row + 1);
                self.buffer.copy_within(rng, row);
                font::write_bytes(
                    self.pixel_writer,
                    0,
                    (16 * row) as i32,
                    &self.buffer[row],
                    &self.fg_color,
                );
            }
            let buffer_last_row = &mut self.buffer[Self::ROWS - 1];
            buffer_last_row.fill(0);
        }
    }
}
