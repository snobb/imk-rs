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
    let brief = format!("Usage: {} -c <cmd> [-r] [-t <threshold>] <files>", program);
    let note = "Please use quotes around the command if it is composed of multiple words";
    print!("{}\n{}\n\n", opts.usage(&brief), note);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt(
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

    opts.optflag(
        "r",
        "recurse",
        "if a directory is supplied, add all its sub-directories as well",
    );

    opts.optflag("h", "help", "display this help text and exit");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            eprintln!("{}", f.to_string());
            return;
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let command: Command = if matches.opt_present("c") {
        let cmd = matches.opt_str("c").unwrap();
        Command::new(cmd)
    } else {
        eprintln!("command must be specified");
        return;
    };

    let threshold = if matches.opt_present("t") {
        matches.opt_str("t").unwrap().parse::<u64>().unwrap_or(0)
    } else {
        0
    };

    let files = if !matches.free.is_empty() {
        matches.free.clone()
    } else {
        eprintln!("files/directories must be specified");
        return;
    };

    if matches.opt_present("r") {
        let mut walker = Walker::new();
        match walker.process(&files) {
            Ok(recursed) => {
                Watcher::new(command, threshold, &recursed).inotify_dispatch();
            }
            Err(err) => eprintln!("Error: {}", err),
        }
    } else {
        Watcher::new(command, threshold, &files).inotify_dispatch();
    }
}
