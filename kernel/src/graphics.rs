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
            FRAME_BUFFER_CONFIG
                .frame_buffer
                .offset(4 * (FRAME_BUFFER_CONFIG.pixels_per_scan_line as i32 * y + x) as isize)
        }
    }

    fn write(&self, x: i32, y: i32, c: &PixelColor);
}

impl PixelWriter for RGBResv8BitPerColorPixelWriter {
    fn write(&self, x: i32, y: i32, c: &PixelColor) {
        let p = self.pixel_at(x, y);
        unsafe {
            let pg = p.offset(1);
            let pb = p.offset(2);
            *p = c.r;
            *pg = c.g;
            *pb = c.b;
        }
    }
}

impl PixelWriter for BGRResv8BitPerColorPixelWriter {
    fn write(&self, x: i32, y: i32, c: &PixelColor) {
        let p = self.pixel_at(x, y);
        unsafe {
            let pg = p.offset(1);
            let pr = p.offset(2);
            *p = c.b;
            *pg = c.g;
            *pr = c.r;
        }
    }
}
