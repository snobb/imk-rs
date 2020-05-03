/*
 *  author: Aleksei Kozadaev (2020)
 */

use config::Config;

use std::collections::HashMap;
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use std::process;

use nix::sys::event::{kevent, kevent_ts, kqueue, EventFilter, EventFlag, FilterFlag, KEvent};

pub struct Watcher<'a> {
    config: &'a Config,
    kq: RawFd,
}

impl<'a> Watcher<'a> {
    pub fn new(config: &'a Config) -> Self {
        Watcher {
            config,
            kq: kqueue().expect("cannot initialise kqueue"),
        }
    }

    pub fn add_watch(&self, filename: &str) -> RawFd {
        let file = File::open(filename).unwrap();
        let fd = file.as_raw_fd();
        ::log_info!(":::FILE::: {} {}", filename, fd);

        let watch_mask = FilterFlag::NOTE_WRITE | FilterFlag::NOTE_DELETE | FilterFlag::NOTE_RENAME;
        let ev_mask = EventFlag::EV_ADD | EventFlag::EV_ONESHOT;
        let filter = EventFilter::EVFILT_VNODE;

        let ev = KEvent::new(fd as usize, filter, ev_mask, watch_mask, 0, 0);

        let target = vec![ev];

        if let Err(e) = kevent(self.kq, &target, &mut Vec::new(), 0) {
            ::log_error!("could not add watcher: {}", e);
            process::exit(1);
        }

        fd
    }

    pub fn dispatch(&mut self) {
        let mut fd_store: HashMap<RawFd, String> = HashMap::new();

        ::log_info!("start monitoring: {}", self.config);

        for file in self.config.files() {
            fd_store.insert(self.add_watch(file), file.to_string());
        }

        loop {
            let source = vec![KEvent::new(
                0,
                EventFilter::EVFILT_VNODE,
                EventFlag::empty(),
                FilterFlag::empty(),
                0,
                0,
            )];

            if let Err(e) = kevent_ts(self.kq, &source, &mut Vec::new(), None) {
                ::log_error!("kevent_ts: {}", e);
                process::exit(1);
            }

            for ev in source.iter() {
                ::log_info!(":::EVENT::: {} {:?}", ev.data(), ev);
            }
        }
    }
}
