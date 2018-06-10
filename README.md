IMK
============
Simple file watcher similar to fswatch or inotify-wait.

Usage:
------
```bash
$ ./imk -h
Usage: ./target/debug/inotify -c <cmd> -t <threshold> <files>

Options:
    -c, --command <COMMAND>
                        command to execute when file is modified
    -t, --threshold <THRESHOLD>
                        number of seconds to skip after the last executed
                        command (default: 0)
    -h, --help          display this help text and exit


Please use quotes around the command if it is composed of multiple words
```

To monitor all .c files and run make run the following:

```bash
$ ./imk -c 'cargo build' src/*.rs
start monitoring: command[cargo build], threshold[0], files[["src/main.rs"]]
   Compiling imk v0.1.0 (file:///imk-rs.git)
    Finished dev [unoptimized + debuginfo] target(s) in 1.7 secs
```

If any of the monitored files are modified, the command will be executed.
