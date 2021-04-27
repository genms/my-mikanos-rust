use crate::printk;
use log::{Level, Metadata, Record};

pub struct Logger {
    pub log_level: Level,
}

impl Logger {
    pub const fn new(log_level: Level) -> Self {
        Logger { log_level }
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.log_level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            printk!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}
