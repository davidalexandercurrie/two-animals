use env_logger::Builder;
use log::LevelFilter;
use std::env;
use std::io::Write;

pub fn init_logger() {
    let mut builder = Builder::new();
    
    // Check if RUST_LOG is set, otherwise use default
    if env::var("RUST_LOG").is_ok() {
        builder.parse_env("RUST_LOG");
    } else {
        // Default to info level for the server crate
        builder.filter_module("server", LevelFilter::Info);
    }
    
    // Custom format for cleaner output
    builder.format(|buf, record| {
        let level = record.level();
        let target = record.target();
        
        // For info level, use minimal formatting
        if level == log::Level::Info && !target.contains("debug") {
            writeln!(buf, "» {}", record.args())
        } else {
            // For debug/trace, include more context
            writeln!(
                buf,
                "[{} {}] {}",
                chrono::Local::now().format("%H:%M:%S"),
                level,
                record.args()
            )
        }
    });
    
    builder.init();
}

// Convenience macros for game events
#[macro_export]
macro_rules! game_event {
    ($($arg:tt)*) => {
        log::info!($($arg)*)
    };
}

#[macro_export]
macro_rules! game_debug {
    ($($arg:tt)*) => {
        log::debug!($($arg)*)
    };
}