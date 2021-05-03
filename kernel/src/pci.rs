#![allow(dead_code)]

use crate::asm;
use crate::error::{Code, Error};
use crate::make_error;
use bit_field::BitField;
use core::fmt;

const CONFIG_ADDRESS: u16 = 0x0cf8;
const CONFIG_DATA: u16 = 0x0cfc;

#[derive(Debug, Copy, Clone)]
pub struct ClassCode {
    pub base: u8,
    pub sub: u8,
    pub interface: u8,
}

impl ClassCode {
    pub const fn new(base: u8, sub: u8, interface: u8) -> Self {
        ClassCode {
            base,
            sub,
            interface,
        }
    }

    pub fn match_base(&self, b: u8) -> bool {
        b == self.base
    }

    pub fn match_sub(&self, b: u8, s: u8) -> bool {
        self.match_base(b) && s == self.sub
    }

    pub fn match_interface(&self, b: u8, s: u8, i: u8) -> bool {
        self.match_sub(b, s) && i == self.interface
    }
}

impl fmt::Display for ClassCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut u = 0u32;
        u.set_bits(24..=31, self.base as u32);
        u.set_bits(16..=23, self.sub as u32);
        u.set_bits(8..=15, self.interface as u32);
        write!(f, "{:08x}", u)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Device {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub header_type: u8,
    pub class_code: ClassCode,
}

impl Device {
    pub const fn new(
        bus: u8,
        device: u8,
        function: u8,
        header_type: u8,
        class_code: ClassCode,
    ) -> Self {
        Device {
            bus,
            device,
            function,
            header_type,
            class_code,
        }
    }
}

static NULL_DEVICE: Device = Device::new(0, 0, 0, 0, ClassCode::new(0, 0, 0));
pub static mut DEVICES: [Device; 32] = [NULL_DEVICE; 32];
pub static mut NUM_DEVICE: usize = 0;

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

fn add_device(device: Device) -> Result<(), Error> {
    unsafe {
        if NUM_DEVICE == DEVICES.len() {
            return Err(make_error!(Code::Full));
        }

        DEVICES[NUM_DEVICE] = device;
        NUM_DEVICE += 1;
    }
    Ok(())
}

fn scan_function(bus: u8, device: u8, function: u8) -> Result<(), Error> {
    let class_code = read_class_code(bus, device, function);
    let header_type = read_header_type(bus, device, function);
    let dev = Device::new(bus, device, function, header_type, class_code.clone());
    add_device(dev)?;

    if class_code.match_sub(0x06, 0x04) {
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

#[inline]
pub fn read_vendor_id_from_dev(dev: &Device) -> u16 {
    read_vendor_id(dev.bus, dev.device, dev.function)
}

pub fn read_device_id(bus: u8, device: u8, function: u8) -> u16 {
    write_address(make_address(bus, device, function, 0x00));
    read_data().get_bits(16..=31) as u16
}

pub fn read_header_type(bus: u8, device: u8, function: u8) -> u8 {
    write_address(make_address(bus, device, function, 0x0c));
    read_data().get_bits(16..=23) as u8
}

pub fn read_class_code(bus: u8, device: u8, function: u8) -> ClassCode {
    write_address(make_address(bus, device, function, 0x08));
    let reg = read_data();
    ClassCode::new(
        reg.get_bits(24..=31) as u8,
        reg.get_bits(16..=23) as u8,
        reg.get_bits(8..=15) as u8,
    )
}

pub fn read_bus_numbers(bus: u8, device: u8, function: u8) -> u32 {
    write_address(make_address(bus, device, function, 0x18));
    read_data()
}

pub fn is_single_function_device(header_type: u8) -> bool {
    !header_type.get_bit(7)
}

pub fn scan_all_bus() -> Result<(), Error> {
    unsafe {
        NUM_DEVICE = 0;
    }

    let header_type = read_header_type(0, 0, 0);
    if is_single_function_device(header_type) {
        return scan_bus(0);
    }

    for function in 0..8 {
        if read_vendor_id(0, 0, function) == 0xffff {
            continue;
        }
        scan_bus(function)?;
    }
    Ok(())
}

pub fn read_conf_reg(dev: &Device, reg_addr: u8) -> u32 {
    write_address(make_address(dev.bus, dev.device, dev.function, reg_addr));
    read_data()
}

pub fn write_conf_reg(dev: &Device, reg_addr: u8, value: u32) {
    write_address(make_address(dev.bus, dev.device, dev.function, reg_addr));
    write_data(value);
}

pub const fn calc_bar_address(bar_index: u32) -> u8 {
    0x10 + 4 * bar_index as u8
}

pub fn read_bar(device: &Device, bar_index: u32) -> Result<u64, Error> {
    if bar_index >= 6 {
        return Err(make_error!(Code::IndexOutOfRange));
    }

    let addr = calc_bar_address(bar_index);
    let bar = read_conf_reg(device, addr);

    // 32 bit address
    if !bar.get_bit(2) {
        return Ok(bar as u64);
    }

    // 64 bit address
    if bar_index >= 5 {
        return Err(make_error!(Code::IndexOutOfRange));
    }

    let bar_upper = read_conf_reg(device, addr + 4);

    let mut ret = bar as u64;
    ret.set_bits(32..=63, bar_upper as u64);
    Ok(ret)
}
