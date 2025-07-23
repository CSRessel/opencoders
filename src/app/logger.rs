use std::fs::OpenOptions;
use std::io::Write;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct LogMessage {
    pub timestamp: String,
    pub level: String,
    pub module: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum LoggerError {
    ChannelClosed,
    FileError(String),
    InitializationFailed(String),
}

#[derive(Debug, Clone)]
pub struct LoggerState {
    pub primary_path: Option<String>,
    pub fallback_path: Option<String>,
    pub last_error: Option<LoggerError>,
    pub messages_sent: u64,
    pub messages_failed: u64,
}

impl LoggerState {
    pub fn new() -> Self {
        Self {
            primary_path: None,
            fallback_path: None,
            last_error: None,
            messages_sent: 0,
            messages_failed: 0,
        }
    }
}

static LOGGER_SENDER: Mutex<Option<mpsc::Sender<LogMessage>>> = Mutex::new(None);
static LOGGER_STATE: Mutex<LoggerState> = Mutex::new(LoggerState {
    primary_path: None,
    fallback_path: None,
    last_error: None,
    messages_sent: 0,
    messages_failed: 0,
});

pub fn init_logger() -> Result<(), LoggerError> {
    let (sender, receiver) = mpsc::channel::<LogMessage>();
    
    // Store the sender for use by logging macros
    {
        let mut guard = LOGGER_SENDER.lock().unwrap();
        *guard = Some(sender);
    }
    
    // Spawn background logging thread
    thread::spawn(move || {
        let mut file_writer = FileWriter::new();
        
        for message in receiver {
            file_writer.write_message(&message);
        }
    });
    
    Ok(())
}

pub fn switch_to_crash_mode() {
    if let Some(sender) = get_sender() {
        let crash_message = LogMessage {
            timestamp: format_timestamp(),
            level: "CRASH".to_string(),
            module: "logger".to_string(),
            message: "Switching to crash mode".to_string(),
        };
        
        let _ = sender.send(crash_message);
    }
}

pub fn get_logger_state() -> LoggerState {
    LOGGER_STATE.lock().unwrap().clone()
}

pub fn get_logger_diagnostics() -> String {
    let state = get_logger_state();
    format!(
        "Logger Diagnostics:\n\
         Primary: {:?}\n\
         Fallback: {:?}\n\
         Messages sent: {}\n\
         Messages failed: {}\n\
         Last error: {:?}",
        state.primary_path,
        state.fallback_path,
        state.messages_sent,
        state.messages_failed,
        state.last_error
    )
}

pub fn get_sender() -> Option<mpsc::Sender<LogMessage>> {
    LOGGER_SENDER.lock().unwrap().clone()
}

fn update_state<F>(updater: F) 
where 
    F: FnOnce(&mut LoggerState)
{
    let mut state = LOGGER_STATE.lock().unwrap();
    updater(&mut *state);
}

struct FileWriter {
    primary_file: Option<std::fs::File>,
    fallback_file: Option<std::fs::File>,
    crash_mode: bool,
}

impl FileWriter {
    fn new() -> Self {
        let mut writer = Self {
            primary_file: None,
            fallback_file: None,
            crash_mode: false,
        };
        
        writer.initialize_primary();
        writer
    }
    
    fn initialize_primary(&mut self) {
        match OpenOptions::new().create(true).append(true).open("opencoders.log") {
            Ok(file) => {
                self.primary_file = Some(file);
                update_state(|state| {
                    state.primary_path = Some("opencoders.log".to_string());
                    state.last_error = None;
                });
            }
            Err(e) => {
                let error = LoggerError::FileError(format!("Failed to open primary log: {}", e));
                update_state(|state| {
                    state.last_error = Some(error);
                });
                self.initialize_fallback();
            }
        }
    }
    
    fn initialize_fallback(&mut self) {
        let crash_path = format!("/tmp/opencoders-crash-{}-{}.log", 
            std::process::id(), 
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        );
        
        match OpenOptions::new().create(true).append(true).open(&crash_path) {
            Ok(mut file) => {
                // Log the fallback initialization
                let fallback_msg = format!("[{}] [WARN] [logger] Using fallback log: {}\n", 
                    format_timestamp(), crash_path);
                let _ = file.write_all(fallback_msg.as_bytes());
                let _ = file.flush();
                
                self.fallback_file = Some(file);
                self.crash_mode = true;
                
                update_state(|state| {
                    state.fallback_path = Some(crash_path);
                    state.last_error = None;
                });
            }
            Err(e) => {
                let error = LoggerError::FileError(format!("Failed to open fallback log: {}", e));
                update_state(|state| {
                    state.last_error = Some(error);
                });
            }
        }
    }
    
    fn write_message(&mut self, message: &LogMessage) {
        let formatted = format!("[{}] [{}] [{}] {}\n", 
            message.timestamp, message.level, message.module, message.message);
        
        let success = if self.crash_mode {
            self.write_to_fallback(&formatted)
        } else {
            self.write_to_primary(&formatted) || {
                // Primary failed, switch to fallback
                self.initialize_fallback();
                self.write_to_fallback(&formatted)
            }
        };
        
        update_state(|state| {
            if success {
                state.messages_sent += 1;
            } else {
                state.messages_failed += 1;
            }
        });
    }
    
    fn write_to_primary(&mut self, message: &str) -> bool {
        if let Some(ref mut file) = self.primary_file {
            if file.write_all(message.as_bytes()).is_ok() && file.flush().is_ok() {
                return true;
            }
        }
        false
    }
    
    fn write_to_fallback(&mut self, message: &str) -> bool {
        if let Some(ref mut file) = self.fallback_file {
            if file.write_all(message.as_bytes()).is_ok() && file.flush().is_ok() {
                return true;
            }
        }
        false
    }
}

pub fn format_timestamp() -> String {
    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH).unwrap();
    let secs = duration.as_secs();
    let nanos = duration.subsec_nanos();
    
    // Convert to date/time components
    let days_since_epoch = secs / 86400;
    let seconds_today = secs % 86400;
    
    // Calculate year (approximate, good enough for logging)
    let mut year = 1970;
    let mut remaining_days = days_since_epoch;
    
    // Simple year calculation (ignoring leap years for simplicity)
    while remaining_days >= 365 {
        if is_leap_year(year) && remaining_days >= 366 {
            remaining_days -= 366;
        } else if !is_leap_year(year) {
            remaining_days -= 365;
        } else {
            break;
        }
        year += 1;
    }
    
    // Calculate month and day (simplified)
    let month_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1;
    let mut day_of_month = remaining_days + 1;
    
    for &days_in_month in &month_days {
        let days_this_month = if month == 2 && is_leap_year(year) { 29 } else { days_in_month };
        if day_of_month <= days_this_month {
            break;
        }
        day_of_month -= days_this_month;
        month += 1;
    }
    
    // Calculate time components
    let hour = seconds_today / 3600;
    let minute = (seconds_today % 3600) / 60;
    let second = seconds_today % 60;
    let millis = nanos / 1_000_000;
    
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z", 
        year, month, day_of_month, hour, minute, second, millis)
}

fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

// Check if terminal is in raw mode (simplified check)
pub fn is_raw_mode() -> bool {
    // This is a simplified check - in a real implementation you might want to
    // track this state more precisely
    std::env::var("TERM").is_ok()
}

#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        if let Some(sender) = $crate::app::logger::get_sender() {
            let message = $crate::app::logger::LogMessage {
                timestamp: $crate::app::logger::format_timestamp(),
                level: $level.to_string(),
                module: module_path!().to_string(),
                message: format!($($arg)*),
            };
            let _ = sender.send(message);
        }
    };
}

#[macro_export]
macro_rules! crash_log {
    ($($arg:tt)*) => {
        $crate::app::logger::switch_to_crash_mode();
        $crate::log!("CRASH", $($arg)*);
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::log!("ERROR", $($arg)*);
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::log!("WARN", $($arg)*);
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::log!("INFO", $($arg)*);
    };
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::log!("DEBUG", $($arg)*);
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {};
}

#[macro_export]
macro_rules! tui_error {
    ($($arg:tt)*) => {
        if !$crate::app::logger::is_raw_mode() {
            eprintln!("[TUI ERROR] {}", format!($($arg)*));
        }
        $crate::log_error!($($arg)*);
    };
}