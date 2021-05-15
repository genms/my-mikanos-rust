//! カーネル本体のプログラムを書いたファイル．
#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(asm)]
#![feature(abi_x86_interrupt)]

mod asm;
mod console;
mod driver;
mod error;
mod font;
mod frame_buffer_config;
mod graphics;
mod hankaku;
mod interrupt;
mod logger;
mod memory_map;
mod mouse;
mod pci;
mod utils;

extern crate num;
#[macro_use]
extern crate num_derive;

use arrayvec::ArrayVec;
use bit_field::BitField;
use core::fmt;
use core::panic::PanicInfo;
use cty::{uint16_t, uint64_t};
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use log::{Level, LevelFilter};

use font::*;
use frame_buffer_config::FrameBufferConfig;
use graphics::*;
use logger::Logger;
use memory_map::*;

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    printk!("Kernel Panic!\n{}", panic_info);
    loop {
        hlt()
    }
}

fn hlt() {
    unsafe {
        asm!("hlt");
    }
}

#[macro_export]
macro_rules! printk {
    ($($x:expr),*) => {
        {
            use core::fmt::Write;

            let mut buf = [0u8; 1024];
            if write!($crate::utils::fmt::Wrapper::new(&mut buf), $($x),*).is_ok() {
                $crate::_printk(&buf);
            }
        }
    };
}

fn _printk(buf: &[u8]) {
    let txt = core::str::from_utf8(buf).unwrap_or("?\n");
    global::console().put_string(txt);
}

extern "C" fn mouse_observer(displacement_x: i8, displacement_y: i8) {
    global::mouse_cursor()
        .move_relative(&Vector2D::new(displacement_x as i32, displacement_y as i32));
}

fn switch_ehci_to_xhci(xhc_dev: &pci::Device) {
    let intel_ehc_exist = || -> bool {
        for dev in pci::device() {
            if dev.class_code.match_interface(0x0c, 0x03, 0x20) /* EHCI */ &&
                            0x8086 == pci::read_vendor_id_from_dev(dev)
            {
                return true;
            }
        }
        false
    }();
    if !intel_ehc_exist {
        return;
    }

    let superspeed_ports = pci::read_conf_reg(xhc_dev, 0xdc); // USB3PRM
    pci::write_conf_reg(xhc_dev, 0xd8, superspeed_ports); // USB3_PSSEN
    let ehci_to_xhci_ports = pci::read_conf_reg(xhc_dev, 0xd4); // XUSB2PRM
    pci::write_conf_reg(xhc_dev, 0xd0, ehci_to_xhci_ports); // XUSB2PR
    debug!(
        "switch_ehci_to_xhci: SS = {:02}, xHCI = {:02x}\n",
        superspeed_ports, ehci_to_xhci_ports
    );
}

#[derive(Debug)]
pub enum MessageType {
    InterruptXHCI,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Message {
    msg_type: MessageType,
}

impl Message {
    const fn new(msg_type: MessageType) -> Self {
        Message { msg_type }
    }
}

extern "x86-interrupt" fn int_handler_xhci(_: *const interrupt::InterruptFrame) {
    global::main_queue().push(Message::new(MessageType::InterruptXHCI));
    interrupt::notify_end_of_interrupt();
}

const DESKTOP_BG_COLOR: PixelColor = PixelColor::new(45, 118, 237);
const DESKTOP_FG_COLOR: PixelColor = PixelColor::new(255, 255, 255);

mod global {
    use crate::*;

    pub(super) static mut LOGGER: Logger = Logger::new(Level::Warn);
    pub fn logger() -> &'static mut Logger {
        unsafe { &mut LOGGER }
    }

    pub(super) static mut PIXEL_WRITER: Option<PixelWriter> = None;
    pub fn pixel_writer() -> &'static PixelWriter {
        unsafe { PIXEL_WRITER.as_ref().unwrap() }
    }

    pub(super) static mut CONSOLE: Option<console::Console> = None;
    pub fn console() -> &'static mut console::Console<'static> {
        unsafe { CONSOLE.as_mut().unwrap() }
    }

    pub(super) static mut MOUSE_CURSOR: Option<mouse::MouseCursor> = None;
    pub fn mouse_cursor() -> &'static mut mouse::MouseCursor<'static> {
        unsafe { MOUSE_CURSOR.as_mut().unwrap() }
    }

    pub(super) static mut XHC_HANDLE: Option<driver::XhcHandle> = None;
    pub fn xhc_handle() -> driver::XhcHandle {
        unsafe { XHC_HANDLE.unwrap() }
    }

    pub(super) static mut MAIN_QUEUE: ArrayVec<Message, 32> = ArrayVec::<Message, 32>::new_const();
    pub fn main_queue() -> &'static mut ArrayVec<Message, 32> {
        unsafe { &mut MAIN_QUEUE }
    }
}

#[no_mangle]
pub extern "C" fn KernelMain(
    fb_config: &'static FrameBufferConfig,
    memory_map: &'static MemoryMap,
) -> ! {
    let pixel_writer: &PixelWriter;
    unsafe {
        log::set_logger(global::logger())
            .map(|()| log::set_max_level(LevelFilter::Trace))
            .unwrap();

        global::PIXEL_WRITER = Some(PixelWriter::new(fb_config));
        pixel_writer = global::pixel_writer();

        global::CONSOLE = Some(console::Console::new(
            pixel_writer,
            DESKTOP_FG_COLOR,
            DESKTOP_BG_COLOR,
        ));
        global::MOUSE_CURSOR = Some(mouse::MouseCursor::new(
            pixel_writer,
            DESKTOP_BG_COLOR,
            Vector2D::new(400, 300),
        ));
    }

    let frame_width = fb_config.horizontal_resolution() as i32;
    let frame_height = fb_config.vertical_resolution() as i32;
    fill_rectangle(
        pixel_writer,
        &Vector2D::new(0, 0),
        &Vector2D::new(frame_width, frame_height - 50),
        &DESKTOP_BG_COLOR,
    );
    fill_rectangle(
        pixel_writer,
        &Vector2D::new(0, frame_height - 50),
        &Vector2D::new(frame_width, 50),
        &PixelColor::new(1, 8, 17),
    );
    fill_rectangle(
        pixel_writer,
        &Vector2D::new(0, frame_height - 50),
        &Vector2D::new(frame_width / 5, 50),
        &PixelColor::new(80, 80, 80),
    );
    draw_rectangle(
        pixel_writer,
        &Vector2D::new(10, frame_height - 40),
        &Vector2D::new(30, 30),
        &PixelColor::new(160, 160, 160),
    );
    write_string(
        pixel_writer,
        660,
        566,
        "my-mikanos-rust",
        &PixelColor::new(160, 160, 160),
    );

    printk!("Welcome to MikanOS in Rust!\n");

    const AVAILABLE_MEMORY_TYPES: [MemoryType; 3] = [
        MemoryType::EfiBootServicesCode,
        MemoryType::EfiBootServicesData,
        MemoryType::EfiConventionalMemory,
    ];

    printk!("memory_map: {:p}\n", memory_map);
    let mut buffer_ptr = memory_map.buffer;
    while (buffer_ptr as usize as u64) < memory_map.buffer as usize as u64 + memory_map.map_size {
        let desc = unsafe { &*(buffer_ptr as *const MemoryDescriptor) };
        let md_type: MemoryType = num::FromPrimitive::from_u32(desc.md_type).unwrap();
        if AVAILABLE_MEMORY_TYPES.contains(&md_type) {
            printk!(
                "type = {}, phys = {:08x} - {:08x}, pages = {}, attr = {:08x}\n",
                desc.md_type,
                desc.physical_start as u64,
                desc.physical_start as u64 + desc.number_of_pages * 4096 - 1,
                desc.number_of_pages,
                desc.attribute
            );
        }
        unsafe {
            buffer_ptr = buffer_ptr.offset(memory_map.descriptor_size as isize);
        }
    }

    global::mouse_cursor().refresh();

    pci::scan_all_bus().unwrap();
    debug!("scan_all_bus: Ok\n");

    for dev in pci::device() {
        let vendor_id = pci::read_vendor_id(dev.bus, dev.device, dev.function);
        let class_code = pci::read_class_code(dev.bus, dev.device, dev.function);
        debug!(
            "{}.{}.{}: vend {:04x}, class {}, head {:02x}\n",
            dev.bus, dev.device, dev.function, vendor_id, class_code, dev.header_type
        );
    }

    // Intel 製を優先して xHC を探す
    let xhc_dev = || -> Option<&pci::Device> {
        let mut ret = None;
        for dev in pci::device() {
            if dev.class_code.match_interface(0x0c, 0x03, 0x30) {
                ret = Some(dev);

                if 0x8086 == pci::read_vendor_id_from_dev(dev) {
                    return ret;
                }
            }
        }
        ret
    }();
    let xhc_dev = xhc_dev.unwrap_or_else(|| {
        info!("xHC not found\n");
        loop {
            hlt()
        }
    });
    info!(
        "xHC has been found: {}.{}.{}\n",
        xhc_dev.bus, xhc_dev.device, xhc_dev.function,
    );

    let cs = unsafe { asm::GetCS() };
    let idt = interrupt::idt();
    interrupt::set_idt_entry(
        &mut idt[interrupt::vector::Number::XHCI as usize],
        interrupt::make_idt_attr(interrupt::DescriptorType::InterruptGate, 0, true, 0),
        int_handler_xhci as u64,
        cs,
    );
    unsafe {
        asm::LoadIDT(
            (core::mem::size_of_val(idt) - 1) as uint16_t,
            &idt[0] as *const interrupt::InterruptDescriptor as uint64_t,
        );
    }

    let bsp_local_apic_id_addr = 0xfee00020 as *const u32;
    let bsp_local_apic_id = unsafe { (*bsp_local_apic_id_addr).get_bits(24..=31) as u8 };
    pci::configure_msi_fixed_destination(
        xhc_dev,
        bsp_local_apic_id,
        pci::MsiTriggerMode::Level,
        pci::MsiDeliveryMode::Fixed,
        interrupt::vector::Number::XHCI as u8,
        0,
    )
    .unwrap();

    let xhc_bar = pci::read_bar(xhc_dev, 0).unwrap();
    debug!("read_bar: Ok\n");

    let mut xhc_mmio_base = xhc_bar;
    xhc_mmio_base.set_bits(0..=3, 0);
    debug!("xHC mmio_base = {:08x}\n", xhc_mmio_base);

    if 0x8086 == pci::read_vendor_id_from_dev(xhc_dev) {
        switch_ehci_to_xhci(xhc_dev);
    }

    unsafe {
        driver::SetLogLevel(driver::LogLevel::kWarn);

        let xhc_handle = driver::UsbInitXhc(xhc_mmio_base);
        driver::print_log();
        global::XHC_HANDLE = Some(xhc_handle);

        asm!("sti");

        driver::UsbConfigurePort(xhc_handle, mouse_observer);
        driver::print_log();
    }

    loop {
        unsafe {
            asm!("cli");
            let main_queue = global::main_queue();
            if main_queue.len() == 0 {
                asm!("sti");
                asm!("hlt");
                continue;
            }

            let msg: Message = main_queue.remove(0);
            asm!("sti");

            #[allow(unreachable_patterns)]
            match msg.msg_type {
                MessageType::InterruptXHCI => {
                    driver::UsbReceiveEvent(global::xhc_handle());
                    driver::print_log();
                }
                _ => {
                    error!("Unknown message type: {}\n", msg.msg_type);
                }
            }
        }
    }
}
