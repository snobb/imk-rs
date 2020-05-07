/*
 * author: Aleksei Kozadaev (2020)
 */

use chrono::prelude::Local;

pub fn get_time() -> String {
    Local::now().format("%H:%M:%S").to_string()
}

#[macro_export]
macro_rules! log_info {
    ($fmt:expr, $($arg:tt)*) => ({
        println!(":: [{}] {}", ::log::get_time(), format_args!($fmt, $($arg)*));
    })
}

#[macro_export]
macro_rules! log_error {
    ($fmt:expr, $($arg:tt)*) => ({
        eprintln!("!! [{}] {}", ::log::get_time(), format_args!($fmt, $($arg)*));
    })
}
