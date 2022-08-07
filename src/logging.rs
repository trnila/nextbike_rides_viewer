use indicatif::ProgressBar;
use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::sync::{Arc, Weak};

pub struct Logger {
    logger: env_logger::Logger,
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.logger.enabled(metadata)
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            let bar = get_progress_bar();
            match bar {
                Some(p) => {
                    p.suspend(|| self.logger.log(record));
                }
                None => self.logger.log(record),
            };
        }
    }

    fn flush(&self) {
        self.logger.flush()
    }
}

impl Logger {
    pub fn init() {
        let logger = env_logger::Builder::from_default_env().build();
        let max_level = logger.filter();

        log::set_boxed_logger(Box::new(Logger { logger })).unwrap();
        log::set_max_level(max_level);
    }
}

pub struct LoggingAwareProgressBar {
    bar: Arc<ProgressBar>,
}

impl LoggingAwareProgressBar {
    pub fn new(len: u64) -> Self {
        ProgressBar::new(len).into()
    }
}

impl From<ProgressBar> for LoggingAwareProgressBar {
    fn from(bar: ProgressBar) -> Self {
        let bar = Arc::new(bar);
        set_progress_bar(Some(Arc::downgrade(&bar)));
        LoggingAwareProgressBar { bar }
    }
}

impl Drop for LoggingAwareProgressBar {
    fn drop(&mut self) {
        self.bar.finish_and_clear();
        set_progress_bar(None);
    }
}

impl std::ops::Deref for LoggingAwareProgressBar {
    type Target = indicatif::ProgressBar;

    fn deref(&self) -> &Self::Target {
        &self.bar
    }
}

lazy_static! {
    static ref PROGRESS_BAR: RwLock<Option<Weak<ProgressBar>>> = RwLock::new(None);
}

fn set_progress_bar(pb: Option<Weak<ProgressBar>>) {
    *PROGRESS_BAR.write() = pb;
}

fn get_progress_bar() -> Option<Arc<ProgressBar>> {
    PROGRESS_BAR.read().as_ref()?.upgrade()
}
