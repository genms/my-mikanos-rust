use cty::{uint16_t, uint32_t, uint64_t};

extern "C" {
    pub fn IoOut32(addr: uint16_t, data: uint32_t);
    pub fn IoIn32(addr: uint16_t) -> uint32_t;
    pub fn GetCS() -> uint16_t;
    pub fn LoadIDT(limit: uint16_t, offset: uint64_t);
}
