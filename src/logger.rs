use std::{
    env,
    fs::File,
    panic::{self, AssertUnwindSafe},
};

use log::LevelFilter;
use simplelog::{Config, ConfigBuilder, WriteLogger};

const LOG_FILE: &str = "erfps2.log";

pub fn init() {
    let result = try_init();

    if let Err(e) = result {
        eprintln!("failed to initialize simplelog: {e}");
    }
}

pub fn try_init() -> eyre::Result<()> {
    let log_file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(LOG_FILE)?;

    WriteLogger::init(
        level_from_env("RUST_LOG", LevelFilter::Info),
        log_format_config(),
        log_file,
    )?;

    Ok(())
}

pub fn set_panic_hook() {
    panic::set_hook(Box::new(|info| {
        let log_panic = |location: &str| {
            log::error!(
                "thread panicked: {}\n    at {}",
                info.payload_as_str().unwrap_or("no panic message"),
                location,
            )
        };

        match info.location() {
            Some(location) => log_panic(&format!("{}:{}", location.file(), location.line())),
            None => log_panic("-"),
        };

        log::logger().flush();
    }));
}

#[macro_export]
macro_rules! log_unwind {
    ($($t:tt)*) => {
        $crate::logger::do_log_unwind(|| $($t)*, file!(), line!(), stringify!($($t)*))
    };
}

pub fn do_log_unwind<F: FnOnce() -> R, R>(f: F, file: &str, line: u32, expr: &str) -> R {
    match panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(res) => res,
        Err(err) => {
            let msg = err
                .downcast::<&'static str>()
                .map(|boxed| *boxed)
                .unwrap_or("no panic message");

            log::error!("expression panicked: {msg}\n    at {file}:{line}:\n    {expr}");
            log::logger().flush();

            std::process::abort();
        }
    }
}

fn level_from_env(env: &str, default: LevelFilter) -> LevelFilter {
    let level = env::var(env).map(|s| s.to_ascii_lowercase());

    match level.as_ref().map(String::as_str) {
        Ok("trace") => LevelFilter::Trace,
        Ok("debug") => LevelFilter::Debug,
        Ok("info") => LevelFilter::Info,
        Ok("warn") => LevelFilter::Warn,
        Ok("error") => LevelFilter::Error,
        Ok("off") => LevelFilter::Off,
        _ => default,
    }
}

fn log_format_config() -> Config {
    ConfigBuilder::new()
        .set_time_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Off)
        .set_time_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Off)
        .build()
}
