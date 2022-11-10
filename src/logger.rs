use log;

/// Customize logger
pub struct Logger;

static LOGGER: Logger = Logger;

impl Logger {
    /// Install logger
    pub fn init() -> Result<(), log::SetLoggerError> {
        log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::Info))
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            //println!("{} - {}", record.level(), record.args());
            eprintln!("{}|{}: {}", record.level(), record.target(), record.args());
        }
    }

    fn flush(&self) {}
}
