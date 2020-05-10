/*
 *  author: Aleksei Kozadaev (2020)
 */

extern crate imk;

extern crate getopts;

use getopts::Options;
use std::env;
use std::time;

use imk::command::Command;
use imk::file_walker::Walker;
use imk::fswatch::Watcher;

fn print_usage(program: &str, opts: Options) {
    let brief = format!(
        "Usage: {} -c <cmd> [-d <cmd>] [-h] [-k <kill-timeout>] [-o] [-r] [-s] [-t <threshold>] <files>",
        program
    );
    let note = "Please use quotes around the command if it is composed of multiple words";
    println!("{}\n{}\n", opts.usage(&brief), note);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.reqopt(
        "c",
        "command",
        "command to execute when file is modified",
        "<COMMAND>",
    );

    opts.optopt(
        "d",
        "teardown",
        "command to execute after the command process is killed (required -k)",
        "<COMMAND>",
    );

    opts.optflag("h", "help", "display this help text and exit");

    opts.optopt(
        "k",
        "kill-timeout",
        "kill the command after timeout (default: 0ms)",
        "<KILL-TIMEOUT>",
    );

    opts.optflag("o", "once", "run command once and exit on event.");

    opts.optflag(
        "r",
        "recurse",
        "if a directory is supplied, add all its sub-directories as well",
    );

    opts.optflag(
        "s",
        "wrap-shell",
        "run the provided command in a shell. Eg. /bin/sh -c <command>.",
    );

    opts.optopt(
        "t",
        "threshold",
        "number of seconds to skip after the last executed command (default: 0)",
        "<THRESHOLD>",
    );

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            print_usage(&program, opts);
            eprintln!("error: {}\n", e);
            return;
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let kill_timeout = if matches.opt_present("k") {
        match matches.opt_str("k").unwrap().parse::<u64>() {
            Ok(v) => Some(time::Duration::from_millis(v)),
            Err(_) => None,
        }
    } else {
        None
    };

    let wrap_shell = matches.opt_present("s");
    let once = matches.opt_present("o");

    let teardown = if matches.opt_present("d") {
        Some(matches.opt_str("d").unwrap())
    } else {
        None
    };

    let command = Command::new(
        wrap_shell,
        once,
        kill_timeout,
        matches.opt_str("c").unwrap(),
        teardown,
    );

    let threshold = if matches.opt_present("t") {
        matches.opt_str("t").unwrap().parse::<u64>().unwrap_or(0)
    } else {
        0
    };

    let files = if !matches.free.is_empty() {
        &matches.free
    } else {
        eprintln!("error: files/directories must be specified",);
        return;
    };

    if matches.opt_present("r") {
        match Walker::new().process(files) {
            Ok(recursed) => {
                Watcher::new(command, threshold, &recursed).dispatch();
            }

            Err(e) => eprintln!("error: could not recurse: {}", e),
        }
    } else {
        Watcher::new(command, threshold, files).dispatch();
    }
}
