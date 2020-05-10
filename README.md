IMK
============
Simple file watcher similar to fswatch or inotify-wait.

Usage:
------
```bash
$ ./imk -h
Usage: imk -c <cmd> [-d <cmd>] [-h] [-k <kill-timeout>] [-o] [-r] [-s] [-t <threshold>] <files>

Options:
    -c, --command <COMMAND>
                        command to execute when file is modified
    -d, --teardown <COMMAND>
                        command to execute after the command process is killed
                        (required -k)
    -h, --help          display this help text and exit
    -k, --kill-timeout <KILL-TIMEOUT>
                        kill the command after timeout (default: 0ms)
    -o, --once          run command once and exit on event.
    -r, --recurse       if a directory is supplied, add all its
                        sub-directories as well
    -s, --wrap-shell    run the provided command in a shell. Eg. /bin/sh -c
                        <command>.
    -t, --threshold <THRESHOLD>
                        number of seconds to skip after the last executed
                        command (default: 0)

Please use quotes around the command if it is composed of multiple words
```

To monitor all .c files and run make run the following:

```bash
$ imk -rc 'cargo build' src/
:: [21:09:57] start monitoring: cmd[cargo build] recurse files[src/]
:: [21:10:12] === src//main.rs (1) ===
   Compiling imk v0.1.0 (/home/snobb/progs/REPOS/imk-rs.git)
    Finished dev [unoptimized + debuginfo] target(s) in 1.32s
:: [21:10:13] === exit code 0 ===
```

If any of the monitored files are modified, the command will be executed.
