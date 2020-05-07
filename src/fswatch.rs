/*
 *  author: Aleksei Kozadaev (2020)
 */

use std::collections::HashMap;
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

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

            Err(e) => ::log_error!("error: failed to add file watch for {}: {}", file, e),
        }
    }

    pub fn dispatch(&mut self) {
        let mut wd_store: HashMap<WatchDescriptor, String> = HashMap::new();

        ::log_info!(
            "start monitoring: {}, threshold[{}], files{:?}",
            self.command,
            self.threshold.as_secs(),
            self.files,
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
                    ::log_info!("===== {} =====", file_name);
                    match self.command.run() {
                        Ok(status) => {
                            match status.code() {
                                Some(code) => {
                                    ::log_info!("===== {} [exit code {}] =====", file_name, code)
                                }

                                None => ::log_info!("===== {} [terminated] =====", file_name),
                            }

                            stdout().flush().unwrap();

                            last = Instant::now();
                        }

                        Err(e) => ::log_error!("error: failed to run the command: {}", e),
                    }
                }

                self.add_watch(&file_name, &mut wd_store);
            }
        }
    }
}
