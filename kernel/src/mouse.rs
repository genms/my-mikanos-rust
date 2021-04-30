#![allow(dead_code)]

use crate::graphics::*;

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

pub struct MouseCursor<'a> {
    pixel_writer: &'a PixelWriter,
    erase_color: PixelColor,
    position: Vector2D<i32>,
}

impl<'a> MouseCursor<'a> {
    pub fn new(
        pixel_writer: &'a PixelWriter,
        erase_color: PixelColor,
        position: Vector2D<i32>,
    ) -> Self {
        let mouse_cursor = MouseCursor {
            pixel_writer,
            erase_color,
            position,
        };
        draw_mouse_cursor(mouse_cursor.pixel_writer, &mouse_cursor.position);
        mouse_cursor
    }

    pub fn move_relative(&mut self, displacement: &Vector2D<i32>) {
        erase_mouse_cursor(&self.pixel_writer, &self.position, &self.erase_color);
        self.position += *displacement;
        draw_mouse_cursor(&self.pixel_writer, &self.position);
    }

    pub fn refresh(&mut self) {
        erase_mouse_cursor(&self.pixel_writer, &self.position, &self.erase_color);
        draw_mouse_cursor(&self.pixel_writer, &self.position);
    }
}

fn draw_mouse_cursor(pixel_writer: &PixelWriter, position: &Vector2D<i32>) {
    for (dy, row) in MOUSE_CURSOR_SHAPE.iter().enumerate() {
        for (dx, c) in row.chars().enumerate() {
            match c {
                '@' => {
                    pixel_writer.write(
                        position.x + dx as i32,
                        position.y + dy as i32,
                        &PixelColor::new(0, 0, 0),
                    );
                }
                '.' => {
                    pixel_writer.write(
                        position.x + dx as i32,
                        position.y + dy as i32,
                        &PixelColor::new(255, 255, 255),
                    );
                }
                _ => {}
            };
        }
    }
}

fn erase_mouse_cursor(
    pixel_writer: &PixelWriter,
    position: &Vector2D<i32>,
    erase_color: &PixelColor,
) {
    for (dy, row) in MOUSE_CURSOR_SHAPE.iter().enumerate() {
        for (dx, c) in row.chars().enumerate() {
            if c != ' ' {
                pixel_writer.write(position.x + dx as i32, position.y + dy as i32, erase_color);
            }
        }
    }
}
