/*
 *  author: Aleksei Kozadaev (2020)
 */

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use chrono::prelude::Local;
use inotify::{EventMask, Inotify, WatchDescriptor, WatchMask};

use command::Command;

pub struct Watcher<'a> {
    inotify: Inotify,
    command: Command,
    threshold: Duration,
    files: &'a [String],
}

impl<'a> Watcher<'a> {
    pub fn new(command: Command, threshold: u64, files: &'a [String]) -> Self {
        Watcher {
            threshold: Duration::from_secs(threshold),
            command,
            files,
            inotify: Inotify::init().expect("Failed to initialize inotify"),
        }
    }

    fn add_watch(&mut self, file: &str, wd_store: &mut HashMap<WatchDescriptor, String>) {
        let watch_mask = WatchMask::MOVE_SELF | WatchMask::MODIFY | WatchMask::ONESHOT;

        match self.inotify.add_watch(PathBuf::from(file), watch_mask) {
            Ok(wd) => {
                wd_store.insert(wd, file.to_string());
            }

            Err(e) => {
                eprintln!(
                    "Could not add inotify watch for {}: {}",
                    file,
                    e.to_string()
                );
            }
        }
    }

    pub fn inotify_dispatch(&mut self) {
        let mut wd_store: HashMap<WatchDescriptor, String> = HashMap::new();

        println!(
            ":: [{}] start monitoring: {}, threshold[{}], files{:?}",
            get_time(),
            self.command,
            self.threshold.as_secs(),
            self.files
        );

        for file in self.files {
            self.add_watch(&file, &mut wd_store);
        }

        let mut buffer = [0u8; 4096];
        let mut last = Instant::now() - self.threshold;

        loop {
            let events = self
                .inotify
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

                if (event.mask.contains(EventMask::MOVE_SELF)
                    || event.mask.contains(EventMask::MODIFY))
                    && last.elapsed() >= self.threshold
                {
                    println!(":: [{}] ======== {} =======", get_time(), file_name);
                    let status = self.command.run_command();

                    match status.code() {
                        Some(code) => println!(
                            ":: [{}] ======= {} [exit code {}] =======",
                            get_time(),
                            file_name,
                            code
                        ),
                        None => println!(
                            ":: [{}] ======= {} [terminated] =======",
                            get_time(),
                            file_name
                        ),
                    }

                    last = Instant::now();
                }

                self.add_watch(&file_name, &mut wd_store);
            }
        }
    }
}

fn get_time() -> String {
    Local::now().format("%H:%M:%S").to_string()
}
