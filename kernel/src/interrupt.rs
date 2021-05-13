//! 割り込み用のプログラムを集めたファイル．
#![allow(dead_code)]

use bit_field::BitField;
use cty::{uint16_t, uint32_t, uint64_t};
use modular_bitfield::prelude::*;

#[repr(C)]
#[derive(BitfieldSpecifier, Debug)]
#[bits = 4]
pub enum DescriptorType {
    Upper8Bytes = 0,
    Ldt = 2,
    TSSAvailable = 9,
    TSSBusy = 11,
    CallGate = 12,
    InterruptGate = 14,
    TrapGate = 15,
}

#[repr(packed)]
#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct InterruptDescriptorAttribute {
    interrupt_stack_table: B3,
    #[skip]
    __: B5,
    descriptor_type: DescriptorType,
    #[skip]
    __: B1,
    descriptor_privilege_level: B2,
    present: bool,
}

#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct InterruptDescriptor {
    offset_low: uint16_t,
    segment_selector: uint16_t,
    attr: InterruptDescriptorAttribute,
    offset_middle: uint16_t,
    offset_high: uint32_t,
    reserved: uint32_t,
}

impl InterruptDescriptor {
    const fn default() -> Self {
        InterruptDescriptor {
            offset_low: 0,
            segment_selector: 0,
            attr: InterruptDescriptorAttribute::from_bytes([0u8; 2]),
            offset_middle: 0,
            offset_high: 0,
            reserved: 0,
        }
    }
}

static mut IDT: [InterruptDescriptor; 256] = [InterruptDescriptor::default(); 256];
pub fn idt() -> &'static mut [InterruptDescriptor] {
    unsafe { &mut IDT }
}

pub fn make_idt_attr(
    descriptor_type: DescriptorType,
    descriptor_privilege_level: u8,
    present: bool,
    interrupt_stack_table: u8,
) -> InterruptDescriptorAttribute {
    InterruptDescriptorAttribute::new()
        .with_interrupt_stack_table(interrupt_stack_table)
        .with_descriptor_type(descriptor_type)
        .with_descriptor_privilege_level(descriptor_privilege_level)
        .with_present(present)
}

pub fn set_idt_entry(
    desc: &mut InterruptDescriptor,
    attr: InterruptDescriptorAttribute,
    offset: u64,
    segment_selector: u16,
) {
    desc.attr = attr;
    desc.offset_low = offset.get_bits(0..=15) as u16;
    desc.offset_middle = offset.get_bits(16..=31) as u16;
    desc.offset_high = offset.get_bits(32..=63) as u32;
    desc.segment_selector = segment_selector;
}

pub mod vector {
    pub enum Number {
        XHCI = 0x40,
    }
}

#[repr(packed)]
pub struct InterruptFrame {
    rip: uint64_t,
    cs: uint64_t,
    rflags: uint64_t,
    rsp: uint64_t,
    ss: uint64_t,
}

pub fn notify_end_of_interrupt() {
    let end_of_interrupt = 0xfee000b0usize as *mut u32;
    unsafe {
        core::ptr::write_volatile(end_of_interrupt, 0);
    }
}
