use cty::{uint32_t, uint8_t};

#[repr(C)]
#[derive(Copy, Clone)]
pub enum PixelFormat {
    PixelRGBResv8BitPerColor,
    PixelBGRResv8BitPerColor,
}

#[repr(C)]
pub struct FrameBufferConfig {
    frame_buffer: *const uint8_t,
    pixels_per_scan_line: uint32_t,
    horizontal_resolution: uint32_t,
    vertical_resolution: uint32_t,
    pixel_format: PixelFormat,
}

impl FrameBufferConfig {
    pub fn frame_buffer(&self) -> *mut u8 {
        self.frame_buffer as *mut u8
    }

    pub fn pixels_per_scan_line(&self) -> u32 {
        self.pixels_per_scan_line as u32
    }

    pub fn horizontal_resolution(&self) -> u32 {
        self.horizontal_resolution as u32
    }

    pub fn vertical_resolution(&self) -> u32 {
        self.vertical_resolution as u32
    }

    pub fn pixel_format(&self) -> PixelFormat {
        self.pixel_format
    }
}
