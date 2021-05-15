use core::fmt;
use cty::{uint32_t, uint64_t, uintptr_t};

#[repr(C)]
pub struct MemoryMap {
    pub buffer_size: uint64_t,
    pub buffer: *const u8,
    pub map_size: uint64_t,
    pub map_key: uint64_t,
    pub descriptor_size: uint64_t,
    pub descriptor_version: uint32_t,
}

#[repr(C)]
pub struct MemoryDescriptor {
    pub md_type: uint32_t,
    pub physical_start: uintptr_t,
    pub virtual_start: uintptr_t,
    pub number_of_pages: uint64_t,
    pub attribute: uint64_t,
}

#[repr(C)]
#[derive(FromPrimitive, PartialEq, Eq, Debug)]
pub enum MemoryType {
    EfiReservedMemoryType,
    EfiLoaderCode,
    EfiLoaderData,
    EfiBootServicesCode,
    EfiBootServicesData,
    EfiRuntimeServicesCode,
    EfiRuntimeServicesData,
    EfiConventionalMemory,
    EfiUnusableMemory,
    EfiACPIReclaimMemory,
    EfiACPIMemoryNVS,
    EfiMemoryMappedIO,
    EfiMemoryMappedIOPortSpace,
    EfiPalCode,
    EfiPersistentMemory,
    EfiMaxMemoryType,
}

impl fmt::Display for MemoryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
