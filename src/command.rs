/*
 *  author: Aleksei Kozadaev (2020)
 */

use std::fmt::{self, Display};
use std::process;

pub struct Command {
    pub command: String,
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.command)
    }
}

impl Command {
    pub fn new(command: String) -> Self {
        Command { command }
    }

    pub fn run_command(&self) -> process::ExitStatus {
        let mut child = process::Command::new("/bin/sh")
            .arg("-c")
            .arg(&self.command)
            .spawn()
            .expect("failed to start the comman");

        child.wait().expect("failed to wait on child")
    }
}
