#![allow(dead_code)]

use crate::asm;
use bit_field::BitField;
use core::fmt;

const CONFIG_ADDRESS: u16 = 0x0cf8;
const CONFIG_DATA: u16 = 0x0cfc;

#[derive(Debug)]
pub enum Error {
    Full,
    Empty,
    LastOfCode,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn make_address(bus: u8, device: u8, function: u8, reg_addr: u8) -> u32 {
    let mut reg_addr_for_address = reg_addr;
    reg_addr_for_address.set_bits(0..=1, 0);

    let mut address = 0u32;
    address
        .set_bit(31, true)
        .set_bits(16..=23, bus as u32)
        .set_bits(11..=15, device as u32)
        .set_bits(8..=10, function as u32)
        .set_bits(0..=7, reg_addr_for_address as u32);
    address
}

fn add_device(bus: u8, device: u8, function: u8, header_type: u8) -> Result<(), Error> {
    unsafe {
        if NUM_DEVICE == DEVICES.len() {
            return Err(Error::Full);
        }

        DEVICES[NUM_DEVICE] = Device {
            bus,
            device,
            function,
            header_type,
        };
        NUM_DEVICE += 1;
    }
    Ok(())
}

fn scan_function(bus: u8, device: u8, function: u8) -> Result<(), Error> {
    let header_type = read_header_type(bus, device, function);
    add_device(bus, device, function, header_type)?;

    let class_code = read_class_code(bus, device, function);
    let base = class_code.get_bits(24..=31) as u8;
    let sub = class_code.get_bits(16..=23) as u8;

    if base == 0x06 && sub == 0x04 {
        let bus_numbers = read_bus_numbers(bus, device, function);
        let secondary_bus = bus_numbers.get_bits(8..=15) as u8;
        return scan_bus(secondary_bus);
    }

    Ok(())
}

fn scan_device(bus: u8, device: u8) -> Result<(), Error> {
    scan_function(bus, device, 0)?;
    if is_single_function_device(read_header_type(bus, device, 0)) {
        return Ok(());
    }

    for function in 1..8 {
        if read_vendor_id(bus, device, function) == 0xffff {
            continue;
        }
        scan_function(bus, device, function)?;
    }
    Ok(())
}

fn scan_bus(bus: u8) -> Result<(), Error> {
    for device in 0..32 {
        if read_vendor_id(bus, device, 0) == 0xffff {
            continue;
        }
        scan_device(bus, device)?;
    }
    Ok(())
}

pub fn write_address(address: u32) {
    unsafe {
        asm::IoOut32(CONFIG_ADDRESS, address);
    }
}

pub fn write_data(value: u32) {
    unsafe {
        asm::IoOut32(CONFIG_DATA, value);
    }
}

pub fn read_data() -> u32 {
    unsafe { asm::IoIn32(CONFIG_DATA) }
}

pub fn read_vendor_id(bus: u8, device: u8, function: u8) -> u16 {
    write_address(make_address(bus, device, function, 0x00));
    read_data().get_bits(0..=15) as u16
}

pub fn read_device_id(bus: u8, device: u8, function: u8) -> u16 {
    write_address(make_address(bus, device, function, 0x00));
    read_data().get_bits(16..=31) as u16
}

pub fn read_header_type(bus: u8, device: u8, function: u8) -> u8 {
    write_address(make_address(bus, device, function, 0x0c));
    read_data().get_bits(16..=23) as u8
}

pub fn read_class_code(bus: u8, device: u8, function: u8) -> u32 {
    write_address(make_address(bus, device, function, 0x08));
    read_data()
}

pub fn read_bus_numbers(bus: u8, device: u8, function: u8) -> u32 {
    write_address(make_address(bus, device, function, 0x18));
    read_data()
}

pub fn is_single_function_device(header_type: u8) -> bool {
    !header_type.get_bit(7)
}

#[derive(Copy, Clone)]
pub struct Device {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub header_type: u8,
}

pub static mut DEVICES: [Device; 32] = [Device {
    bus: 0,
    device: 0,
    function: 0,
    header_type: 0,
}; 32];
pub static mut NUM_DEVICE: usize = 0;

pub fn scan_all_bus() -> Result<(), Error> {
    unsafe {
        NUM_DEVICE = 0;
    }

    let header_type = read_header_type(0, 0, 0);
    if is_single_function_device(header_type) {
        return scan_bus(0);
    }

    for function in 1..8 {
        if read_vendor_id(0, 0, function) == 0xffff {
            continue;
        }
        scan_bus(function)?;
    }
    Ok(())
}
