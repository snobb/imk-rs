/*
 *  author: Aleksei Kozadaev (2020)
 */

use std::fmt::{self, Display};
use std::io;
use std::process;
use std::time::Instant;
use std::{thread, time};

use log::get_time;

pub struct Command {
    pub command: String,
    wrap_shell: bool,
    once: bool,
    timeout_ms: Option<u64>,
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.timeout_ms {
            Some(v) => write!(
                f,
                "command[{}], wrap-shell[{}], timeout_ms[{}]",
                self.command, self.wrap_shell, v
            ),
            None => write!(
                f,
                "command[{}], wrap-shell[{}]",
                self.command, self.wrap_shell
            ),
        }
    }
}

impl Command {
    pub fn new(command: String, wrap_shell: bool, once: bool, timeout_ms: Option<u64>) -> Self {
        Command {
            command,
            wrap_shell,
            once,
            timeout_ms,
        }
    }

    pub fn run(&self) -> io::Result<process::ExitStatus> {
        let mut child = if self.wrap_shell {
            process::Command::new("/bin/sh")
                .arg("-c")
                .arg(&self.command)
                .spawn()
        } else {
            let cmd: Vec<&str> = self.command.split_whitespace().collect();
            process::Command::new(cmd[0]).args(&cmd[1..]).spawn()
        }?;

        let status: process::ExitStatus;
        if let Some(timeout_ms) = self.timeout_ms {
            status = wait_timeout(&mut child, time::Duration::from_millis(timeout_ms));
        } else {
            status = child.wait().expect("unable to wait for process");
        }

        if self.once {
            process::exit(status.code().unwrap());
        } else {
            Ok(status)
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
                eprintln!("!! [{}] error: {}", get_time(), e);
                child.kill().expect("unable to kill process")
            }
        }

        thread::sleep(time::Duration::from_millis(100));
    }
}
