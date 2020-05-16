/*
 * author: Aleksei Kozadaev (2020)
 */

use getopts::Options;
use std::env;
use std::fmt::{self, Display};
use std::process::exit;
use std::time::{self, Duration};

use command::Command;
use walker::Walker;

#[derive(Default)]
pub struct Config {
    pub threshold: Duration,
    recurse: bool,
    command: Command,
    files: Vec<String>,
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!(
        "Usage: {} -c <cmd> [-d <cmd>] [-h] [-k <kill-timeout>] [-o] [-r] [-s] [-t <threshold>] <files>",
        program
    );

    let note = "Please use quotes around the command if it is composed of multiple words";
    println!("{}\n{}\n", opts.usage(&brief), note);
}

impl Config {
    pub fn new() -> Self {
        let args: Vec<String> = env::args().collect();
        let program = &args[0];

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
                print_usage(program, opts);
                eprintln!("error: {}\n", e);
                exit(1);
            }
        };

        if matches.opt_present("h") {
            print_usage(&program, opts);
            exit(0);
        }

        let kill_timeout = if matches.opt_present("k") {
            match matches.opt_str("k").unwrap().parse::<u64>() {
                Ok(v) => Some(time::Duration::from_millis(v)),
                Err(_) => None,
            }
        } else {
            None
        };

        let once = matches.opt_present("o");
        let wrap_shell = matches.opt_present("s");

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
            let t = matches.opt_str("t").unwrap().parse::<u64>().unwrap_or(0);
            Duration::from_secs(t)
        } else {
            Duration::from_secs(0)
        };

        let recurse = matches.opt_present("r");

        let files = if !matches.free.is_empty() {
            if recurse {
                let mut enriched: Vec<String> = vec![];
                if let Err(e) = Walker::new(&mut enriched).process(&matches.free) {
                    eprintln!("error: could not recurse: {}", e);
                    exit(1);
                };
                enriched
            } else {
                matches.free
            }
        } else {
            eprintln!("error: files/directories must be specified",);
            exit(1);
        };

        Config {
            threshold,
            recurse,
            command,
            files,
        }
    }

    pub fn command(&self) -> &Command {
        &self.command
    }

    pub fn files(&self) -> &Vec<String> {
        &self.files
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut opts: Vec<String> = vec![];

        if self.threshold > Duration::from_secs(0) {
            opts.push(format!("threshold[{}]", self.threshold.as_secs()));
        }

        if self.recurse {
            opts.push("recurse".to_string());
        }

        opts.push(format!("files{:?}", self.files));

        write!(f, "{} {}", self.command, opts.join(" "))
    }
}
