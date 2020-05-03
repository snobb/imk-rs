/*
 *  author: Aleksei Kozadaev (2020)
 */

extern crate chrono;
extern crate getopts;

#[cfg(target_os = "linux")]
extern crate inotify;

#[cfg(any(
    target_os = "macos",
    target_os = "openbsd",
    target_os = "freebsd",
    target_os = "netbsd"
))]
extern crate nix;

pub mod command;
pub mod config;
pub mod log;
pub mod walker;

pub mod fswatch {
    #[cfg(target_os = "linux")]
    pub mod linux;

    #[cfg(any(
        target_os = "macos",
        target_os = "openbsd",
        target_os = "freebsd",
        target_os = "netbsd"
    ))]
    pub mod bsd;
}
