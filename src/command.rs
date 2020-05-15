/*
 *  author: Aleksei Kozadaev (2020)
 */

use std::fmt::{self, Display};
use std::io;
use std::process;
use std::time::Instant;
use std::{thread, time};

pub struct Command {
    wrap_shell: bool,
    once: bool,
    timeout_ms: Option<time::Duration>,
    command: String,
    teardown: Option<String>,
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut opts: Vec<String> = vec![format!("command[{}]", self.command)];

        if let Some(timeout_ms) = self.timeout_ms {
            if let Some(teardown) = &self.teardown {
                opts.push(format!("teardown[{}]", teardown));
            }

            opts.push(format!("timeout_ms[{}]", timeout_ms.as_millis()));
        }

        if self.wrap_shell {
            opts.push("wrap_shell".to_string());
        }

        if self.once {
            opts.push("once".to_string());
        }

        write!(f, "{}", opts.join(" "))
    }
}

impl Command {
    pub fn new(
        wrap_shell: bool,
        once: bool,
        timeout_ms: Option<time::Duration>,
        command: String,
        teardown: Option<String>,
    ) -> Self {
        Command {
            wrap_shell,
            once,
            timeout_ms,
            command,
            teardown,
        }
    }

    pub fn run(&self) -> io::Result<Option<i32>> {
        let mut child = if self.wrap_shell {
            process::Command::new("/bin/sh")
                .arg("-c")
                .arg(&self.command)
                .spawn()
        } else {
            let cmd: Vec<&str> = self.command.split_whitespace().collect();
            process::Command::new(cmd[0]).args(&cmd[1..]).spawn()
        }?;

        let status = self.wait_timeout(&mut child);

        if self.once {
            if let Ok(Some(code)) = status {
                process::exit(code);
            } else {
                process::exit(1);
            }
        } else {
            status
        }
    }

    fn wait_timeout(&self, child: &mut process::Child) -> io::Result<Option<i32>> {
        let start = Instant::now();

        loop {
            match child.try_wait() {
                Ok(Some(status)) => match status.code() {
                    Some(code) => return Ok(Some(code)),
                    None => return Ok(None),
                },

                Ok(None) => {
                    // still running - check for timeout_ms
                    if let Some(timeout) = self.timeout_ms {
                        if Instant::now().duration_since(start) > timeout {
                            self.handle_timeout(child);
                        }
                    }
                }

                Err(e) => return Err(e),
            }

            thread::sleep(time::Duration::from_millis(100));
        }
    }

    fn handle_timeout(&self, child: &mut process::Child) {
        // still running - check for timeout_ms
        if let Some(teardown) = &self.teardown {
            process::Command::new("/bin/sh")
                .arg("-c")
                .arg(&teardown)
                .env("CMD_PID", child.id().to_string())
                .status()
                .ok();
        } else {
            child.kill().ok();
        }
    }
}
