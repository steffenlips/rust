use std::io::Write;
use std::thread::JoinHandle;
use std::{
    fs::{File, OpenOptions},
    sync::{mpsc, Mutex},
    thread,
};

use chrono::{SecondsFormat, Utc};
use log::{error, Level, LevelFilter, Metadata, Record};
use once_cell::sync::OnceCell;

///////////////////////////////////////////////////////////////////////////////
struct Logger {}
static LOGGER: OnceCell<Logger> = OnceCell::new();
///////////////////////////////////////////////////////////////////////////////
struct LoggerOptions {
    console: bool,
    file: Option<File>,
    thread: Option<thread::JoinHandle<()>>,
    sender: Option<Mutex<mpsc::Sender<String>>>,
}
static LOGGER_OPTIONS: OnceCell<Mutex<LoggerOptions>> = OnceCell::new();
///////////////////////////////////////////////////////////////////////////////
impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() as u8 <= log::max_level() as u8
    }
    ///////////////////////////////////////////////////////////////////////////////
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let now = Utc::now();
            let level_str = match record.level() {
                Level::Error => "ERR",
                Level::Warn => "WRN",
                Level::Info => "INF",
                Level::Debug => "DBG",
                Level::Trace => "TRC",
            };
            let msg = format!(
                "{} [{}] [{}] {}",
                &now.to_rfc3339_opts(SecondsFormat::Millis, true),
                level_str,
                record.metadata().target(),
                record.args()
            );

            if !cfg!(test) {
                match || -> Result<(), String> {
                    LOGGER_OPTIONS
                        .get()
                        .ok_or("[asynclogger] Options not available")?
                        .lock()
                        .or_else(|err| Err(format!("[asynclogger] {}", err)))?
                        .sender
                        .as_ref()
                        .ok_or("[asynclogger] Logger thread not correctly initialized")?
                        .lock()
                        .or_else(|err| Err(format!("[asynclogger] {}", err)))?
                        .send(msg)
                        .or_else(|err| Err(format!("[asynclogger] {}", err)))?;

                    Ok(())
                }() {
                    Ok(()) => (),
                    Err(err) => {
                        println!("{}", err);
                    }
                }
            } else {
                println!("{}", msg);
            }
        }
    }
    ///////////////////////////////////////////////////////////////////////////////
    fn flush(&self) {}
}
///////////////////////////////////////////////////////////////////////////////
impl LoggerOptions {
    fn initialize_thread(&mut self) {
        let (log_sender, log_receiver) = mpsc::channel();
        self.sender = Some(Mutex::new(log_sender));
        self.thread = Some(std::thread::spawn(move || loop {
            let message: String = log_receiver.recv().unwrap();
            if message.find("###").is_some() {
                break;
            }
            match LOGGER_OPTIONS.get() {
                Some(options) => {
                    let locked = options.lock().unwrap();
                    if locked.console {
                        println!("{}", message.to_string());
                    }
                    if locked.file.is_some() {
                        let mut file = locked.file.as_ref().unwrap();
                        writeln!(file, "{}", message.to_string()).unwrap();
                    }
                }
                None => (),
            }
        }))
    }
    ///////////////////////////////////////////////////////////////////////////////
    pub fn wait(&mut self) -> Option<JoinHandle<()>> {
        self.thread.take()
    }
    ///////////////////////////////////////////////////////////////////////////////
    fn set_log_file(&mut self, log_file: &str) {
        self.file = Some(
            OpenOptions::new()
                .append(false)
                .write(true)
                .create(true)
                .open(log_file)
                .unwrap(),
        );
        self.file.as_ref().unwrap().set_len(0).unwrap();
    }
}
///////////////////////////////////////////////////////////////////////////////
pub fn configure_logging(filter: log::LevelFilter, console: bool, log_file: &str) {
    let result = cfg!(test);
    if !result {
        match LOGGER.get() {
            Some(_) => (),
            None => {
                LOGGER.set(Logger {}).ok();
                log::set_logger(LOGGER.get().unwrap())
                    .map(|()| log::set_max_level(LevelFilter::Trace))
                    .ok();
            }
        }

        log::set_max_level(filter);

        // initialize if not exists
        match LOGGER_OPTIONS.get() {
            Some(_) => (),
            None => {
                let mut options = LoggerOptions {
                    console: true,
                    file: None,
                    sender: None,
                    thread: None,
                };
                options.initialize_thread();
                LOGGER_OPTIONS.set(Mutex::new(options)).ok();
            }
        }

        // set new options
        match LOGGER_OPTIONS.get() {
            Some(options) => {
                let mut lock = options.lock().unwrap();
                lock.console = console;

                if log_file.len() > 0 {
                    lock.set_log_file(log_file);
                }
            }
            None => (),
        }
    }
}

pub fn flush_logger() {
    let result = cfg!(test);
    if !result {
        error!("###");
        // initialize if not exists
        match LOGGER_OPTIONS.get() {
            Some(options) => {
                let mut lock = options.lock().unwrap();
                let handle = lock.wait().unwrap();
                drop(lock);
                handle.join().unwrap();
            }
            None => (),
        }
    }
}
