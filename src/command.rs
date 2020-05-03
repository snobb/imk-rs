/*
 *  author: Aleksei Kozadaev (2020)
 */

use std::fmt::{self, Display};
use std::process;
use std::time::Instant;
use std::{thread, time};

pub struct Command {
    pub command: String,
    kill_timeout: Option<u64>,
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kill_timeout {
            Some(v) => write!(f, "command[{}], kill_timeout[{}]", self.command, v),
            None => write!(f, "command[{}]", self.command),
        }
    }
}

impl Command {
    pub fn new(command: String, kill_timeout: Option<u64>) -> Self {
        Command {
            command,
            kill_timeout,
        }
    }

    pub fn run(&self) -> process::ExitStatus {
        let mut child = process::Command::new("/bin/sh")
            .arg("-c")
            .arg(&self.command)
            .spawn()
            .expect("failed to start the comman");

        if let Some(timeout) = self.kill_timeout {
            wait_timeout(&mut child, time::Duration::from_millis(timeout))
        } else {
            // no kill_timeout - wait infinitely
            child.wait().expect("unable to wait for process")
        }
    }
}

fn wait_timeout(child: &mut process::Child, timeout: time::Duration) -> process::ExitStatus {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status,

            Ok(None) => {
                // still running - check for timeout
                if Instant::now().duration_since(start) > timeout {
                    child.kill().expect("unable to kill process");
                }
            }

            Err(e) => {
                eprintln!("error: {}", e);
                child.kill().expect("unable to kill process");
            }
        }

        thread::sleep(time::Duration::from_millis(100));
    }
}
