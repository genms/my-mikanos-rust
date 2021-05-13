//! PCI バス制御のプログラムを集めたファイル．
#![allow(dead_code)]

use crate::asm;
use crate::error::{Code, Error};
use crate::make_error;
use bit_field::BitField;
use core::fmt;
use modular_bitfield::prelude::*;

/// CONFIG_ADDRESS レジスタの IO ポートアドレス
const CONFIG_ADDRESS: u16 = 0x0cf8;
/// CONFIG_DATA レジスタの IO ポートアドレス
const CONFIG_DATA: u16 = 0x0cfc;

/// PCI デバイスのクラスコード
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

    /// ベースクラスが等しい場合に真を返す
    pub fn match_base(&self, b: u8) -> bool {
        b == self.base
    }

    /// ベースクラスとサブクラスが等しい場合に真を返す
    pub fn match_sub(&self, b: u8, s: u8) -> bool {
        self.match_base(b) && s == self.sub
    }

    /// ベース，サブ，インターフェースが等しい場合に真を返す
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

/// PCI デバイスを操作するための基礎データを格納する
///
/// バス番号，デバイス番号，ファンクション番号はデバイスを特定するのに必須．
/// その他の情報は単に利便性のために加えてある．
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

/// ScanAllBus() により発見された PCI デバイスの一覧
static mut DEVICES: [Device; 32] = [NULL_DEVICE; 32];
pub fn device() -> &'static [Device] {
    unsafe { &DEVICES[0..NUM_DEVICE] }
}

pub fn get_device(idx: usize) -> &'static Device {
    unsafe { &DEVICES[idx] }
}

/// devices の有効な要素の数
static mut NUM_DEVICE: usize = 0;
pub fn num_device() -> usize {
    unsafe { NUM_DEVICE }
}

/// CONFIG_ADDRESS 用の 32 ビット整数を生成する
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

/// devices[num_device] に情報を書き込み num_device をインクリメントする．
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

/// 指定のファンクションを devices に追加する．
/// もし PCI-PCI ブリッジなら，セカンダリバスに対し ScanBus を実行する
fn scan_function(bus: u8, device: u8, function: u8) -> Result<(), Error> {
    let class_code = read_class_code(bus, device, function);
    let header_type = read_header_type(bus, device, function);
    let dev = Device::new(bus, device, function, header_type, class_code.clone());
    add_device(dev)?;

    if class_code.match_sub(0x06, 0x04) {
        // standard PCI-PCI bridge
        let bus_numbers = read_bus_numbers(bus, device, function);
        let secondary_bus = bus_numbers.get_bits(8..=15) as u8;
        return scan_bus(secondary_bus);
    }

    Ok(())
}

/// 指定のデバイス番号の各ファンクションをスキャンする．
/// 有効なファンクションを見つけたら ScanFunction を実行する．
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

/// 指定のバス番号の各デバイスをスキャンする．
/// 有効なデバイスを見つけたら ScanDevice を実行する．
fn scan_bus(bus: u8) -> Result<(), Error> {
    for device in 0..32 {
        if read_vendor_id(bus, device, 0) == 0xffff {
            continue;
        }
        scan_device(bus, device)?;
    }
    Ok(())
}

/// 指定された MSI ケーパビリティ構造を読み取る
///
/// * `dev` - MSI ケーパビリティを読み込む PCI デバイス
/// * `cap_addr` - MSI ケーパビリティレジスタのコンフィグレーション空間アドレス
fn read_msi_capability(dev: &Device, cap_addr: u8) -> MsiCapability {
    let header_data = read_conf_reg(dev, cap_addr);
    let header = MsiCapabilityHeader::from_bytes(header_data.to_ne_bytes().clone());
    let msg_addr = read_conf_reg(dev, cap_addr + 4);

    let msg_upper_addr;
    let msg_data_addr;
    if header.addr_64_capable() {
        msg_upper_addr = read_conf_reg(dev, cap_addr + 8);
        msg_data_addr = cap_addr + 12;
    } else {
        msg_upper_addr = 0;
        msg_data_addr = cap_addr + 8;
    }
    let msg_data = read_conf_reg(dev, msg_data_addr);

    let mask_bits;
    let pending_bits;
    if header.per_vector_mask_capable() {
        mask_bits = read_conf_reg(dev, msg_data_addr + 4);
        pending_bits = read_conf_reg(dev, msg_data_addr + 8);
    } else {
        mask_bits = 0;
        pending_bits = 0;
    }

    MsiCapability {
        header,
        msg_addr,
        msg_upper_addr,
        msg_data,
        mask_bits,
        pending_bits,
    }
}

/// 指定された MSI ケーパビリティ構造に書き込む
///
/// * `dev` - MSI ケーパビリティを読み込む PCI デバイス
/// * `cap_addr` - MSI ケーパビリティレジスタのコンフィグレーション空間アドレス
/// * `msi_cap` - 書き込む値
fn write_msi_capability(dev: &Device, cap_addr: u8, msi_cap: &MsiCapability) {
    write_conf_reg(
        dev,
        cap_addr,
        u32::from_ne_bytes(msi_cap.header.clone().into_bytes()),
    );
    write_conf_reg(dev, cap_addr + 4, msi_cap.msg_addr);

    let msg_data_addr;
    if msi_cap.header.addr_64_capable() {
        write_conf_reg(dev, cap_addr + 8, msi_cap.msg_upper_addr);
        msg_data_addr = cap_addr + 12;
    } else {
        msg_data_addr = cap_addr + 8;
    }

    write_conf_reg(dev, msg_data_addr, msi_cap.msg_data);

    if msi_cap.header.per_vector_mask_capable() {
        write_conf_reg(dev, msg_data_addr + 4, msi_cap.mask_bits);
        write_conf_reg(dev, msg_data_addr + 8, msi_cap.pending_bits);
    }
}

/// 指定された MSI レジスタを設定する
fn configure_msi_register(
    dev: &Device,
    cap_addr: u8,
    msg_addr: u32,
    msg_data: u32,
    num_vector_exponent: u32,
) -> Result<(), Error> {
    let mut msi_cap = read_msi_capability(dev, cap_addr);

    if msi_cap.header.multi_msg_capable() <= num_vector_exponent as u8 {
        msi_cap
            .header
            .set_multi_msg_enable(msi_cap.header.multi_msg_capable());
    } else {
        msi_cap
            .header
            .set_multi_msg_enable(num_vector_exponent as u8);
    }

    msi_cap.header.set_msi_enable(true);
    msi_cap.msg_addr = msg_addr;
    msi_cap.msg_data = msg_data;
    write_msi_capability(dev, cap_addr, &msi_cap);
    Ok(())
}

/// 指定された MSI レジスタを設定する
fn configure_msix_register(
    _dev: &Device,
    _cap_addr: u8,
    _msg_addr: u32,
    _msg_data: u32,
    _num_vector_exponent: u32,
) -> Result<(), Error> {
    Err(make_error!(Code::NotImplemented))
}

/// CONFIG_ADDRESS に指定された整数を書き込む
pub fn write_address(address: u32) {
    unsafe {
        asm::IoOut32(CONFIG_ADDRESS, address);
    }
}

/// CONFIG_DATA に指定された整数を書き込む
pub fn write_data(value: u32) {
    unsafe {
        asm::IoOut32(CONFIG_DATA, value);
    }
}

/// CONFIG_DATA から 32 ビット整数を読み込む
pub fn read_data() -> u32 {
    unsafe { asm::IoIn32(CONFIG_DATA) }
}

/// ベンダ ID レジスタを読み取る（全ヘッダタイプ共通）
pub fn read_vendor_id(bus: u8, device: u8, function: u8) -> u16 {
    write_address(make_address(bus, device, function, 0x00));
    read_data().get_bits(0..=15) as u16
}

/// ベンダ ID レジスタを読み取る（全ヘッダタイプ共通）
#[inline]
pub fn read_vendor_id_from_dev(dev: &Device) -> u16 {
    read_vendor_id(dev.bus, dev.device, dev.function)
}

/// デバイス ID レジスタを読み取る（全ヘッダタイプ共通）
pub fn read_device_id(bus: u8, device: u8, function: u8) -> u16 {
    write_address(make_address(bus, device, function, 0x00));
    read_data().get_bits(16..=31) as u16
}

/// ヘッダタイプレジスタを読み取る（全ヘッダタイプ共通）
pub fn read_header_type(bus: u8, device: u8, function: u8) -> u8 {
    write_address(make_address(bus, device, function, 0x0c));
    read_data().get_bits(16..=23) as u8
}

/// クラスコードレジスタを読み取る（全ヘッダタイプ共通）
pub fn read_class_code(bus: u8, device: u8, function: u8) -> ClassCode {
    write_address(make_address(bus, device, function, 0x08));
    let reg = read_data();
    ClassCode::new(
        reg.get_bits(24..=31) as u8,
        reg.get_bits(16..=23) as u8,
        reg.get_bits(8..=15) as u8,
    )
}

/// バス番号レジスタを読み取る（ヘッダタイプ 1 用）
///
/// 返される 32 ビット整数の構造は次の通り．
///   - 23:16 : サブオーディネイトバス番号
///   - 15:8  : セカンダリバス番号
///   - 7:0   : リビジョン番号
pub fn read_bus_numbers(bus: u8, device: u8, function: u8) -> u32 {
    write_address(make_address(bus, device, function, 0x18));
    read_data()
}

/// 単一ファンクションの場合に真を返す．
pub fn is_single_function_device(header_type: u8) -> bool {
    !header_type.get_bit(7)
}

/// PCI デバイスをすべて探索し devices に格納する
///
/// バス 0 から再帰的に PCI デバイスを探索し，devices の先頭から詰めて書き込む．
/// 発見したデバイスの数を num_devices に設定する．
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

/// 指定された PCI デバイスの 32 ビットレジスタを読み取る
pub fn read_conf_reg(dev: &Device, reg_addr: u8) -> u32 {
    write_address(make_address(dev.bus, dev.device, dev.function, reg_addr));
    read_data()
}

/// 指定された PCI デバイスの 32 ビットレジスタに書き込む
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

/// PCI ケーパビリティレジスタの共通ヘッダ
#[repr(packed)]
#[bitfield]
pub struct CapabilityHeader {
    cap_id: B8,
    next_ptr: B8,
    cap: B16,
}

pub const CAPABILITY_MSI: u8 = 0x05;
pub const CAPABILITY_MSIX: u8 = 0x11;

/// 指定された PCI デバイスの指定されたケーパビリティレジスタを読み込む
///
/// * `dev` - ケーパビリティを読み込む PCI デバイス
/// * `addr` - ケーパビリティレジスタのコンフィグレーション空間アドレス
pub fn read_capability_header(dev: &Device, addr: u8) -> CapabilityHeader {
    let header_data = read_conf_reg(dev, addr);
    CapabilityHeader::from_bytes(header_data.to_ne_bytes().clone())
}

#[repr(packed)]
#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct MsiCapabilityHeader {
    cap_id: B8,
    next_ptr: B8,
    msi_enable: bool,
    multi_msg_capable: B3,
    multi_msg_enable: B3,
    addr_64_capable: bool,
    per_vector_mask_capable: bool,
    #[skip]
    __: B7,
}

/// MSI ケーパビリティ構造
///
/// MSI ケーパビリティ構造は 64 ビットサポートの有無などで亜種が沢山ある．
/// この構造体は各亜種に対応するために最大の亜種に合わせてメンバを定義してある．
#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct MsiCapability {
    header: MsiCapabilityHeader,
    msg_addr: u32,
    msg_upper_addr: u32,
    msg_data: u32,
    mask_bits: u32,
    pending_bits: u32,
}

/// MSI または MSI-X 割り込みを設定する
///
/// * `dev` - 設定対象の PCI デバイス
/// * `msg_addr` - 割り込み発生時にメッセージを書き込む先のアドレス
/// * `msg_data` - 割り込み発生時に書き込むメッセージの値
/// * `num_vector_exponent` - 割り当てるベクタ数（2^n の n を指定）
pub fn configure_msi(
    dev: &Device,
    msg_addr: u32,
    msg_data: u32,
    num_vector_exponent: u32,
) -> Result<(), Error> {
    let mut cap_addr: u8 = read_conf_reg(dev, 0x34).get_bits(0..=7) as u8;
    let mut msi_cap_addr: u8 = 0;
    let mut msix_cap_addr: u8 = 0;
    while cap_addr != 0 {
        let header = read_capability_header(dev, cap_addr);
        if header.cap_id() == CAPABILITY_MSI {
            msi_cap_addr = cap_addr;
        } else if header.cap_id() == CAPABILITY_MSIX {
            msix_cap_addr = cap_addr;
        }
        cap_addr = header.next_ptr();
    }

    if msi_cap_addr != 0 {
        configure_msi_register(dev, msi_cap_addr, msg_addr, msg_data, num_vector_exponent)
    } else if msix_cap_addr != 0 {
        configure_msix_register(dev, msi_cap_addr, msg_addr, msg_data, num_vector_exponent)
    } else {
        Err(make_error!(Code::NoPCIMSI))
    }
}

#[derive(PartialEq, Eq)]
pub enum MsiTriggerMode {
    Edge = 0,
    Level = 1,
}

#[derive(PartialEq, Eq)]
pub enum MsiDeliveryMode {
    Fixed = 0b000,
    LowestPriority = 0b001,
    Smi = 0b010,
    Nmi = 0b100,
    Init = 0b101,
    ExtInt = 0b111,
}

pub fn configure_msi_fixed_destination(
    dev: &Device,
    apic_id: u8,
    trigger_mode: MsiTriggerMode,
    delivery_mode: MsiDeliveryMode,
    vector: u8,
    num_vector_exponent: u32,
) -> Result<(), Error> {
    let mut msg_addr: u32 = 0xfee00000;
    msg_addr.set_bits(12..=19, apic_id as u32);

    let mut msg_data: u32 = 0;
    msg_data.set_bits(0..=7, vector as u32);
    msg_data.set_bits(8..=10, delivery_mode as u32);
    if trigger_mode == MsiTriggerMode::Level {
        msg_data.set_bit(14, true);
        msg_data.set_bit(15, true);
    }

    configure_msi(dev, msg_addr, msg_data, num_vector_exponent)
}
