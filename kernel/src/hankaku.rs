use cty::uint8_t;

extern "C" {
    static _binary_hankaku_bin_start: uint8_t;
    static _binary_hankaku_bin_end: uint8_t;
    static _binary_hankaku_bin_size: uint8_t;
}

pub fn get_font(c: char) -> Result<*mut u8, char> {
    unsafe {
        let index = 16 * c as usize;
        if index >= &_binary_hankaku_bin_size as *const u8 as usize {
            return Err(c);
        }
        let start = &_binary_hankaku_bin_start as *const u8 as *mut u8;
        Ok(start.offset(index as isize))
    }
}

pub fn get_font_slice(c: char) -> Result<&'static [u8; 16], char> {
    let font = get_font(c)?;
    Ok(unsafe { &*(font as *const [u8; 16]) })
}
