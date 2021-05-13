use core::fmt;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Code {
    Success,
    Full,
    Empty,
    NoEnoughMemory,
    IndexOutOfRange,
    HostControllerNotHalted,
    InvalidSlotID,
    PortNotConnected,
    InvalidEndpointNumber,
    TransferRingNotSet,
    AlreadyAllocated,
    NotImplemented,
    InvalidDescriptor,
    BufferTooSmall,
    UnknownDevice,
    NoCorrespondingSetupStage,
    TransferFailed,
    InvalidPhase,
    UnknownXHCISpeedID,
    NoWaiter,
    NoPCIMSI,
    LastOfCode, // この列挙子は常に最後に配置する
}

#[derive(Debug)]
pub struct Error {
    code: Code,
    file: &'static str,
    line: u32,
}

impl Error {
    pub fn new(code: Code, file: &'static str, line: u32) -> Self {
        Error { code, file, line }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[macro_export]
macro_rules! make_error {
    ($x:expr) => {{
        Error::new(($x), file!(), line!())
    }};
}
