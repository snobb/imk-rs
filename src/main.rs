/*
 *  author: Aleksei Kozadaev (2020)
 */

extern crate imk;

extern crate getopts;

use getopts::Options;
use std::env;

use imk::command::Command;
use imk::file_walker::Walker;
use imk::fswatch::Watcher;

fn print_usage(program: &str, opts: Options) {
    let brief = format!(
        "Usage: {} -c <cmd> [-r] [-s] [-o] [-k <kill-timeout>] [-t <threshold>] <files>",
        program
    );
    let note = "Please use quotes around the command if it is composed of multiple words";
    print!("{}\n{}\n\n", opts.usage(&brief), note);
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
        "t",
        "threshold",
        "number of seconds to skip after the last executed command (default: 0)",
        "<THRESHOLD>",
    );

    opts.optopt(
        "k",
        "kill-timeout",
        "kill the command after timeout (default: 0ms)",
        "<KILL-TIMEOUT>",
    );

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

    opts.optflag("o", "once", "run command once and exit on event.");
    opts.optflag("h", "help", "display this help text and exit");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            print_usage(&program, opts);
            eprintln!("{}\n", f.to_string());
            return;
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let kill_timeout = if matches.opt_present("k") {
        match matches.opt_str("k").unwrap().parse::<u64>() {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    } else {
        None
    };

    let wrap_shell = matches.opt_present("s");
    let once = matches.opt_present("o");

    let command = Command::new(
        matches.opt_str("c").unwrap(),
        wrap_shell,
        once,
        kill_timeout,
    );

    let threshold = if matches.opt_present("t") {
        matches.opt_str("t").unwrap().parse::<u64>().unwrap_or(0)
    } else {
        0
    };

    let files = if !matches.free.is_empty() {
        &matches.free
    } else {
        eprintln!("files/directories must be specified");
        return;
    };

    if matches.opt_present("r") {
        match Walker::new().process(files) {
            Ok(recursed) => {
                Watcher::new(command, threshold, &recursed).dispatch();
            }

            Err(err) => eprintln!("Error: {}", err),
        }
    } else {
        Watcher::new(command, threshold, files).dispatch();
    }
}
