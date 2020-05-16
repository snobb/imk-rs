/*
 *  author: Aleksei Kozadaev (2020)
 */

extern crate imk;

use imk::config::Config;
use imk::fswatch::Watcher;

fn main() {
    let config = Config::new();
    Watcher::new(&config).dispatch();
}
