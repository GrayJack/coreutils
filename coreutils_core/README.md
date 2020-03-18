# Coreutils Core

It's a library used by [this](https://github.com/GrayJack/coreutils) implementation of coreutils in Rust programming language.

Think of it as a crate to create coreutils tools.

Here lies several abstraction for Unix/Unix-like structures in the OS supported by the project.
 * Backup module that handles creating backups
 * Input module to get user confirmation
 * mktemp: Wrappers for libc mkstemp(3) and mkdtemp(3)
 * mkfifo: Wrapper for libc mkfifo(3) and maybe expandable for other FIFO related functions
 * OS module with abstractions for more os specific stuff
    * Audit structs and syscall for FreeBSD and MacOS
    * group: Module with structures and methods to handle groups information
    * load: Safe abstraction to system function to get load of the system
    * login_name: Safe abstractions to system function to get caller login name
    * passwd: Module that holds structures and methods to handle system user information
    * process: Safe abstraction related to process handling
    * routing_table: Routing table abstractions for OpenBSD
    * time: OS functions to handle system time (set system time)
    * tty: Abstractions to check if a FileDescriptor is a TTY and ways to get the TTY name
    * utmp and utmpx: Types representing account database on unix and methods to use them
    * utsname: Types to aquire system information

It also re-export major needed things from [bstr](https://github.com/BurntSushi/bstr), a crate with a string type for Rust that is not required to be valid UTF-8, as well as [time](https://github.com/time-rs/time) and [libc](https://github.com/rust-lang/libc)
