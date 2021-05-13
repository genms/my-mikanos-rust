//! 画像描画関連のプログラムを集めたファイル．
use crate::frame_buffer_config::*;
use core::ops::{Add, AddAssign};

pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl PixelColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        PixelColor { r, g, b }
    }
}

pub struct PixelWriter {
    config: &'static FrameBufferConfig,
    write_fn: fn(&Self, i32, i32, &PixelColor),
}

impl PixelWriter {
    pub fn new(config: &'static FrameBufferConfig) -> Self {
        PixelWriter {
            config,
            write_fn: match config.pixel_format() {
                PixelFormat::PixelRGBResv8BitPerColor => Self::write_rgb,
                PixelFormat::PixelBGRResv8BitPerColor => Self::write_bgr,
            },
        }
    }

    pub fn write(&self, x: i32, y: i32, c: &PixelColor) {
        (self.write_fn)(self, x, y, c);
    }

    fn pixel_at(&self, x: i32, y: i32) -> *mut u8 {
        unsafe {
            self.config
                .frame_buffer()
                .offset(4 * (self.config.pixels_per_scan_line() as i32 * y + x) as isize)
        }
    }

    fn read_mut(&self, x: i32, y: i32) -> &'static mut [u8; 4] {
        let p = self.pixel_at(x, y);
        unsafe { &mut *(p as *mut [u8; 4]) }
    }

    fn write_rgb(&self, x: i32, y: i32, c: &PixelColor) {
        let pixel = self.read_mut(x, y);
        (*pixel)[0] = c.r;
        (*pixel)[1] = c.g;
        (*pixel)[2] = c.b;
    }

    fn write_bgr(&self, x: i32, y: i32, c: &PixelColor) {
        let pixel = self.read_mut(x, y);
        (*pixel)[0] = c.b;
        (*pixel)[1] = c.g;
        (*pixel)[2] = c.r;
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Vector2D<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vector2D<T> {
    pub const fn new(x: T, y: T) -> Self {
        Vector2D::<T> { x, y }
    }
}

impl<T> Add for Vector2D<T>
where
    T: Add<Output = T> + Copy + Clone,
{
    type Output = Vector2D<T>;

    fn add(self, other: Self) -> Self::Output {
        Vector2D::<T> {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T> AddAssign for Vector2D<T>
where
    T: Add<Output = T> + Copy + Clone,
{
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

pub fn draw_rectangle(
    writer: &PixelWriter,
    pos: &Vector2D<i32>,
    size: &Vector2D<i32>,
    c: &PixelColor,
) {
    for dx in 0..size.x {
        writer.write(pos.x + dx, pos.y, c);
        writer.write(pos.x + dx, pos.y + size.y - 1, c);
    }
    for dy in 1..(size.y - 1) {
        writer.write(pos.x, pos.y + dy, c);
        writer.write(pos.x + size.x - 1, pos.y + dy, c);
    }
}

pub fn fill_rectangle(
    writer: &PixelWriter,
    pos: &Vector2D<i32>,
    size: &Vector2D<i32>,
    c: &PixelColor,
) {
    for dy in 0..size.y {
        for dx in 0..size.x {
            writer.write(pos.x + dx, pos.y + dy, c);
        }
    }
}
