#![allow(dead_code)]

use crate::graphics::*;
use crate::PIXEL_WRITER;

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

pub struct MouseCursor {
    erase_color: PixelColor,
    position: Vector2D<i32>,
}

impl MouseCursor {
    pub const fn new(erase_color: PixelColor, position: Vector2D<i32>) -> Self {
        MouseCursor {
            erase_color,
            position,
        }
    }

    pub fn move_relative(&mut self, displacement: &Vector2D<i32>) {
        erase_mouse_cursor(&self.position, &self.erase_color);
        self.position += *displacement;
        draw_mouse_cursor(&self.position);
    }

    pub fn refresh(&mut self) {
        erase_mouse_cursor(&self.position, &self.erase_color);
        draw_mouse_cursor(&self.position);
    }
}

fn draw_mouse_cursor(position: &Vector2D<i32>) {
    let pixel_writer = unsafe { PIXEL_WRITER.unwrap() };

    for (dy, row) in MOUSE_CURSOR_SHAPE.iter().enumerate() {
        for (dx, b) in row.as_bytes().iter().enumerate() {
            match *b as char {
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

fn erase_mouse_cursor(position: &Vector2D<i32>, erase_color: &PixelColor) {
    let pixel_writer = unsafe { PIXEL_WRITER.unwrap() };

    for (dy, row) in MOUSE_CURSOR_SHAPE.iter().enumerate() {
        for (dx, b) in row.as_bytes().iter().enumerate() {
            if *b as char != ' ' {
                pixel_writer.write(position.x + dx as i32, position.y + dy as i32, erase_color);
            }
        }
    }
}
