use std::sync::Once;
use env_logger::{Builder, Target};
use std::io::Write;
use chrono::Local;
use colored::Colorize;
use log::{debug, error, info, Level, trace, warn};
use tokio::signal;

static INIT: Once = Once::new();

pub fn init_log_once() {
    INIT.call_once(|| {
        let mut builder = Builder::from_default_env();
        builder
            .target(Target::Stdout)
            .format(|buf, record| {
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                let level = match record.level() {
                    Level::Error => record.level().to_string().red(),
                    Level::Warn => record.level().to_string().yellow(),
                    Level::Info => record.level().to_string().green(),
                    Level::Debug => record.level().to_string().blue(),
                    Level::Trace => record.level().to_string().purple(),
                };
                let file = record.file().unwrap_or("unknown");
                let line = record.line().unwrap_or(0);
                writeln!(buf, "{} [{}:{}][{}]--> {}", timestamp,  file, line,level, record.args())
            })
            .init();
    });
}

#[test]
fn test() {
    unsafe { std::env::set_var("RUST_LOG", "trace"); }
    init_log_once();
    info!("This is an info log");
    warn!("This is a warn log");
    debug!("This is a debug log");
    trace!("This is a trace log");
    error!("This is an error log");
}