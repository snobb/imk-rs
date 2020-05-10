/*
 *  author: Aleksei Kozadaev (2020)
 */

use std::fmt::{self, Display};
use std::io;
use std::process;
use std::time::Instant;
use std::{thread, time};

pub enum CommandResult<T, E> {
    Killed,
    Status(T),
    Error(E),
}

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

    pub fn run(&self) -> CommandResult<i32, io::Error> {
        let child_result = if self.wrap_shell {
            process::Command::new("/bin/sh")
                .arg("-c")
                .arg(&self.command)
                .spawn()
        } else {
            let cmd: Vec<&str> = self.command.split_whitespace().collect();
            process::Command::new(cmd[0]).args(&cmd[1..]).spawn()
        };

        let mut child = match child_result {
            Ok(child) => child,
            Err(e) => return CommandResult::Error(e),
        };

        let status = self.wait_timeout(&mut child);

        if self.once {
            if let CommandResult::Status(code) = status {
                process::exit(code);
            } else {
                process::exit(1);
            }
        } else {
            status
        }
    }

    fn wait_timeout(&self, child: &mut process::Child) -> CommandResult<i32, io::Error> {
        let start = Instant::now();

        loop {
            match child.try_wait() {
                Ok(Some(status)) => match status.code() {
                    Some(code) => return CommandResult::Status(code),
                    None => return CommandResult::Killed,
                },

                Ok(None) => {
                    if let Some(timeout) = self.timeout_ms {
                        if Instant::now().duration_since(start) > timeout {
                            self.handle_timeout(child);
                        }
                    }
                }

                Err(e) => {
                    return CommandResult::Error(e);
                }
            }

            thread::sleep(time::Duration::from_millis(100));
        }
    }

    fn handle_timeout(&self, child: &mut process::Child) -> CommandResult<i32, io::Error> {
        // still running - check for timeout_ms
        if let Some(teardown) = &self.teardown {
            process::Command::new("/bin/sh")
                .arg("-c")
                .arg(&teardown)
                .env("CMD_PID", child.id().to_string())
                .status()
                .expect("could not run the teardown command");
            CommandResult::Killed
        } else {
            match child.kill() {
                Ok(_) => CommandResult::Killed,
                Err(e) => CommandResult::Error(e),
            }
        }
    }
}
