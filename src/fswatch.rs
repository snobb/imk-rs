/*
 *  author: Aleksei Kozadaev (2020)
 */

use std::collections::HashMap;
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::time::Instant;

use inotify::{EventMask, Inotify, WatchDescriptor, WatchMask};

use config::Config;

pub struct Watcher<'a> {
    config: &'a Config,
    inotify: Inotify,
}

impl<'a> Watcher<'a> {
    pub fn new(config: &'a Config) -> Self {
        Watcher {
            config,
            inotify: Inotify::init().expect("Failed to initialize inotify"),
        }
    }

    fn add_watch(&mut self, file: &str, wd_store: &mut HashMap<WatchDescriptor, String>) {
        let watch_mask = WatchMask::MOVE_SELF | WatchMask::MODIFY | WatchMask::ONESHOT;

        match self.inotify.add_watch(PathBuf::from(file), watch_mask) {
            Ok(wd) => {
                wd_store.insert(wd, file.to_string());
            }

            Err(e) => ::log_error!("failed to add file watch for {}: {}", file, e),
        }
    }

    pub fn dispatch(&mut self) {
        let mut wd_store: HashMap<WatchDescriptor, String> = HashMap::new();

        if self.config.is_immediate() {
            ::log_info!("run command immediately: {}", self.config);
            self.run_command("immediate");
        }

        ::log_info!("start monitoring: {}", self.config);

        for file in self.config.files() {
            self.add_watch(file, &mut wd_store);
        }

        let mut buffer = [0u8; 4096];
        let mut last = Instant::now() - self.config.threshold;

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
                    && last.elapsed() >= self.config.threshold
                {
                    ::log_info!("===== {} =====", file_name);
                    self.run_command(&file_name);
                    last = Instant::now();
                }

                self.add_watch(&file_name, &mut wd_store);
            }
        }
    }

    fn run_command(&self, file_name: &str) {
        match self.config.command().run() {
            Ok(Some(code)) => {
                ::log_info!("===== {} [exit code {}] =====", file_name, code)
            }

            Ok(None) => ::log_info!("===== {} [terminated] =====", file_name),

            Err(e) => ::log_error!("failed to run the command: {}", e),
        }

        stdout().flush().unwrap();
    }
}
