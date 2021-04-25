extern "C" {
    pub fn IoOut32(addr: u16, data: u32);
    pub fn IoIn32(addr: u16) -> u32;
}
