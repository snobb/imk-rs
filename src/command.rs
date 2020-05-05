/*
 *  author: Aleksei Kozadaev (2020)
 */

use std::fmt::{self, Display};
use std::process;
use std::time::Instant;
use std::{thread, time};

pub struct Command {
    pub command: String,
    wrap_shell: bool,
    timeout_ms: Option<u64>,
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.timeout_ms {
            Some(v) => write!(f, "command[{}], timeout_ms[{}]", self.command, v),
            None => write!(f, "command[{}]", self.command),
        }
    }
}

impl Command {
    pub fn new(command: String, wrap_shell: bool, timeout_ms: Option<u64>) -> Self {
        Command {
            command,
            wrap_shell,
            timeout_ms,
        }
    }

    pub fn run(&self) -> process::ExitStatus {
        let mut child = if self.wrap_shell {
            process::Command::new("/bin/sh")
                .arg("-c")
                .arg(&self.command)
                .spawn()
                .expect("failed to start the comman")
        } else {
            let cmd: Vec<&str> = self.command.split_whitespace().collect();
            process::Command::new(cmd[0])
                .args(&cmd[1..])
                .spawn()
                .expect("failed to start the comman")
        };

        if let Some(timeout_ms) = self.timeout_ms {
            wait_timeout(&mut child, time::Duration::from_millis(timeout_ms))
        } else {
            // no timeout_ms - wait infinitely
            child.wait().expect("unable to wait for process")
        }
    }
}

fn wait_timeout(child: &mut process::Child, timeout_ms: time::Duration) -> process::ExitStatus {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status,

            Ok(None) => {
                // still running - check for timeout_ms
                if Instant::now().duration_since(start) > timeout_ms {
                    child.kill().expect("unable to kill process")
                }
            }

            Err(e) => {
                eprintln!("error: {}", e);
                child.kill().expect("unable to kill process")
            }
        }

        thread::sleep(time::Duration::from_millis(100));
    }
}
