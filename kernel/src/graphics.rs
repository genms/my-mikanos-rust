use core::ops::{Add, AddAssign};
use crate::FRAME_BUFFER_CONFIG;

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

pub struct RGBResv8BitPerColorPixelWriter {}
pub struct BGRResv8BitPerColorPixelWriter {}

pub trait PixelWriter {
    fn pixel_at(&self, x: i32, y: i32) -> *mut u8 {
        unsafe {
            let frame_buffer_config = FRAME_BUFFER_CONFIG.unwrap();
            frame_buffer_config
                .frame_buffer
                .offset(4 * (frame_buffer_config.pixels_per_scan_line as i32 * y + x) as isize)
        }
    }

    fn read_mut(&self, x: i32, y: i32) -> &mut [u8; 4] {
        let p = self.pixel_at(x, y);
        unsafe { &mut *(p as *mut [u8; 4]) }
    }

    fn write(&self, x: i32, y: i32, c: &PixelColor);
}

impl PixelWriter for RGBResv8BitPerColorPixelWriter {
    fn write(&self, x: i32, y: i32, c: &PixelColor) {
        let pixel = self.read_mut(x, y);
        (*pixel)[0] = c.r;
        (*pixel)[1] = c.g;
        (*pixel)[2] = c.b;
    }
}

impl PixelWriter for BGRResv8BitPerColorPixelWriter {
    fn write(&self, x: i32, y: i32, c: &PixelColor) {
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

impl<T> Add for Vector2D<T> where T: Add<Output=T> + Copy + Clone {
    type Output = Vector2D<T>;

    fn add(self, other: Self) -> Self::Output {
        Vector2D::<T> {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T> AddAssign for Vector2D<T> where T: Add<Output=T> + Copy + Clone {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

pub fn draw_rectangle(
    writer: &dyn PixelWriter,
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
    writer: &dyn PixelWriter,
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
