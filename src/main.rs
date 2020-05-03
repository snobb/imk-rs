/*
 *  author: Aleksei Kozadaev (2020)
 */

extern crate imk;

use imk::config::Config;

#[cfg(any(
    target_os = "macos",
    target_os = "openbsd",
    target_os = "freebsd",
    target_os = "netbsd"
))]
use imk::fswatch::bsd::Watcher;

#[cfg(target_os = "linux")]
use imk::fswatch::linux::Watcher;

fn main() {
    let config = Config::new();
    Watcher::new(&config).dispatch();
}
