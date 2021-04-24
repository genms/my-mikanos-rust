use crate::FRAME_BUFFER_CONFIG;

pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
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
