use crate::FRAME_BUFFER_CONFIG;

pub struct PixelColor(pub u8, pub u8, pub u8);

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
        (*pixel)[0] = c.0;
        (*pixel)[1] = c.1;
        (*pixel)[2] = c.2;
    }
}

impl PixelWriter for BGRResv8BitPerColorPixelWriter {
    fn write(&self, x: i32, y: i32, c: &PixelColor) {
        let pixel = self.read_mut(x, y);
        (*pixel)[0] = c.2;
        (*pixel)[1] = c.1;
        (*pixel)[2] = c.0;
    }
}

pub struct Vector2D<T>(pub T, pub T);

pub fn draw_rectangle(
    writer: &dyn PixelWriter,
    pos: &Vector2D<i32>,
    size: &Vector2D<i32>,
    c: &PixelColor,
) {
    for dx in 0..size.0 {
        writer.write(pos.0 + dx, pos.1, c);
        writer.write(pos.0 + dx, pos.1 + size.1 - 1, c);
    }
    for dy in 1..(size.1 - 1) {
        writer.write(pos.0, pos.1 + dy, c);
        writer.write(pos.0 + size.0 - 1, pos.1 + dy, c);
    }
}

pub fn fill_rectangle(
    writer: &dyn PixelWriter,
    pos: &Vector2D<i32>,
    size: &Vector2D<i32>,
    c: &PixelColor,
) {
    for dy in 0..size.1 {
        for dx in 0..size.0 {
            writer.write(pos.0 + dx, pos.1 + dy, c);
        }
    }
}
