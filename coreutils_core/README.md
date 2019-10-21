# Coreutils Core

It's a library used by [this](https://github.com/GrayJack/coreutils) implementation of coreutils in Rust programming language.

Here lies several abstraction for Unix/Unix-like structures in the OS supported by the project.
 * `Passwd`: A struct that holds the information of a system user
 * `Group`: This struct holds information about a system group
 * `Groups`: A collection of `Group`
 * `UtsName`: A struct that holds general system information
 * tty module: helper function about tty and `TtyName` to get the ttyname
 * Routing table abstractions for OpenBSD
 * Audit structs and syscall for FreeBSD and MacOS
 * mktemp: Wrappers for libc mkstemp(3) and mkdtemp(3)

It also re-export major needed things from [bstr](https://github.com/BurntSushi/bstr), a crate with a string type for Rust that is not required to be valid UTF-8.
