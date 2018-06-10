/*
 *  main.rs
 *  author: Aleksei Kozadaev (2018)
 */

extern crate inotify;
extern crate getopts;
extern crate chrono;

use std::env;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::process::{Command, Stdio};

use getopts::Options;
use chrono::prelude::{Local};

use inotify::{
    EventMask,
    WatchMask,
    WatchDescriptor,
    Inotify,
};

struct Imk<'a> {
    inotify: Inotify,
    command: String,
    threshold: Duration,
    files: &'a Vec<String>
}

impl<'a> Imk<'a> {
    fn new(command: &String, threshold: u64, files: &'a Vec<String>) -> Imk<'a> {
        Imk {
            command: command.clone(),
            threshold: Duration::from_secs(threshold),
            files: files,
            inotify: Inotify::init().expect("Failed to initialize inotify")
        }
    }

    fn run_command(&self) {
        Command::new("/bin/sh")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .arg("-c")
            .arg(&self.command)
            .spawn()
            .unwrap();
    }

    fn add_watch(&mut self, file: &String, wd_store: &mut HashMap<WatchDescriptor, String>) {
        let watch_mask = WatchMask::MOVE_SELF | WatchMask::MODIFY | WatchMask::ONESHOT;

        match self.inotify.add_watch(PathBuf::from(file), watch_mask) {
            Ok(wd) => {
                wd_store.insert(wd, file.clone());
            },

            Err(e) => {
                eprintln!("Could not add inotify watch for {}: {}", file, e.to_string());
            }
        }
    }

    fn inotify_dispatch(&mut self) -> () {
        let mut wd_store:HashMap<WatchDescriptor, String> = HashMap::new();

        println!(":: [{}] start monitoring: command[{}], threshold[{}], files[{:?}]",
                 get_time(), self.command, self.threshold.as_secs(), self.files);

        for file in self.files {
            self.add_watch(&file, &mut wd_store);
        }

        let mut buffer = [0u8; 4096];
        let mut last = Instant::now() - self.threshold;

        loop {
            let events = self.inotify
                .read_events_blocking(&mut buffer)
                .expect("Failed to read inotify events");

            for event in events {
                let wd = event.wd;

                let file_name = match wd_store.remove(&wd) {
                    Some(f) => f,
                    None => {
                        continue;
                    }
                };

                if event.mask.contains(EventMask::MOVE_SELF) || event.mask.contains(EventMask::MODIFY) {
                    if last.elapsed() >= self.threshold {
                        println!(":: [{}] ======== {} =======", get_time(), file_name);
                        self.run_command();
                        last = Instant::now();
                    }
                }

                self.add_watch(&file_name, &mut wd_store);
            }
        }
    }
}

fn get_time() -> String {
    Local::now().format("%H:%M:%S").to_string()
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} -c <cmd> -t <threshold> <files>", program);
    let note = "Please use quotes around the command if it is composed of multiple words";
    print!("{}\n\n{}\n", opts.usage(&brief), note);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("c", "command",
                "command to execute when file is modified",
                "<COMMAND>");

    opts.optopt("t", "threshold",
                "number of seconds to skip after the last executed command (default: 0)",
                "<THRESHOLD>");

    opts.optflag("h", "help",
                 "display this help text and exit");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => {
            eprintln!("{}", f.to_string());
            return;
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let threshold = if matches.opt_present("t") {
        matches.opt_str("t").unwrap()
            .parse::<u64>().unwrap_or(0)
    } else {
        0u64
    };

    let command: String = if matches.opt_present("c") {
        matches.opt_str("c").unwrap()
    } else {
        eprintln!("command must be specified");
        return;
    };

    let files = if !matches.free.is_empty() {
        matches.free
    } else {
        eprintln!("files/directories must be specified");
        return;
    };

    Imk::new(&command, threshold, &files)
        .inotify_dispatch();
}
