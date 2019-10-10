# Coreutils in Rust

[![Hits-of-Code](https://hitsofcode.com/github/GrayJack/coreutils)](https://hitsofcode.com/view/github/GrayJack/coreutils)
[![Build Status](https://api.travis-ci.com/GrayJack/coreutils.svg?branch=master)](https://travis-ci.com/GrayJack/coreutils)
[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2FGrayJack%2Fcoreutils.svg?type=shield)](https://app.fossa.io/projects/git%2Bgithub.com%2FGrayJack%2Fcoreutils?ref=badge_shield)


An attempt to make safe coreutils aiming a minimal and yet complete set of utilities. This project aims to have **at least** the common features between several implementations of the utility.

This project has no intent to be 100% compatible with GNU's coreutils, like [Uutils' coreutils](https://github.com/uutils/coreutils), but if happens to be, that's ok too.

## Contributing
Check the [CONTRIBUTING.md](./CONTRIBUTING.md) file for the guidelines to contribute to the project, including issue reports, git commits messages, etc.

## Minimum Rust Version Policy
This project's minimum supported `rustc` version (MSRV) is 1.37.0.

In general, this project will try to be conservative with respect to the minimum supported version of Rust, but in case of safety reasons it may bump at any time. [e.g. `MaybeUninit` stabilization on 1.36.0 fixing huge problems with `std::mem::uninitialized()`]

## Compilation tests guarantees
The compilations is tested for Rust x86_64 Unix/Unix-like platforms, with exception of redox target, with Github Actions.

The Tier1 platforms are tested on MSRV, stable, beta and nightly Rust, while Tier2 and Tier3 platforms are tested on MSRV and stable only, since they are no guarantees to be available in beta and nightly Rust.

Also note that Tier3 Rust Platform will probably fail before hits the compilation check, since my only options is to use a Linux OS, add the target and do `cargo check --target`, and they often are not available as a target for any Tier1 platforms. If you use one of these platforms and have the right MSRV requirements and it build successfully, let me know, if it fails, open a issue with the compilation error.

|   Platform    |  Tier  | CI Status | Manual Status |
|:-------------:|:------:|:---------:|:-------------:|
|  Linux        | Tier1  | [![Linux](https://github.com/GrayJack/coreutils/workflows/Linux/badge.svg)](https://github.com/GrayJack/coreutils/actions)               | Passing (Manjaro Linux 5.2.17 - 2019-10-01) (202fa79) |
|  MacOS        | Tier1  | [![MacOS](https://github.com/GrayJack/coreutils/workflows/MacOS/badge.svg)](https://github.com/GrayJack/coreutils/actions)               | - |
|  FreeBSD      | Tier2  | [![FreeBSD](https://github.com/GrayJack/coreutils/workflows/FreeBSD/badge.svg)](https://github.com/GrayJack/coreutils/actions)           | Passing (FreeBSD 12.0 - 2019-10-01) (202fa79) |
|  NetBSD       | Tier2  | [![NetBSD](https://github.com/GrayJack/coreutils/workflows/NetBSD/badge.svg)](https://github.com/GrayJack/coreutils/actions)             | - (system without minimal version) |
|  Solaris      | Tier2  | [![Solaris](https://github.com/GrayJack/coreutils/workflows/Solaris/badge.svg)](https://github.com/GrayJack/coreutils/actions)           | - (system without minimal version) |
|  Fuchsia      | Tier2  | [![Fuchsia](https://github.com/GrayJack/coreutils/workflows/Fuchsia/badge.svg)](https://github.com/GrayJack/coreutils/actions)           | - |
|  OpenBSD      | Tier3  | [![OpenBSD](https://github.com/GrayJack/coreutils/workflows/OpenBSD/badge.svg)](https://github.com/GrayJack/coreutils/actions)           | Passing (OpenBSD 6.6 Current - 2019-10-01) (202fa79) |
|  DragonflyBSD | Tier3  | [![DragonflyBSD](https://github.com/GrayJack/coreutils/workflows/DragonflyBSD/badge.svg)](https://github.com/GrayJack/coreutils/actions) | Passing (DragonflyBSD 5.6.2 - 2019-10-01) (202fa79) |
|  Haiku        | Tier3  | [![Haiku](https://github.com/GrayJack/coreutils/workflows/Haiku/badge.svg)](https://github.com/GrayJack/coreutils/actions)               | - |

## Tools
|   Name   | Not Started | Started | Done |
|:--------:|:-----------:|:-------:|:----:|
| basename |             |         |   X  |
|    cat   |      X      |         |      |
|   chgrp  |      X      |         |      |
|   chmod  |      X      |         |      |
|   chown  |      X      |         |      |
|  chroot  |      X      |         |      |
|   clear  |             |         |   X  |
|   comm   |      X      |         |      |
|    cp    |      X      |         |      |
|  csplit  |      X      |         |      |
|    cut   |      X      |         |      |
|   date   |             |    X    |      |
|    dd    |      X      |         |      |
|    df    |      X      |         |      |
|   diff   |      X      |         |      |
|  dirname |             |         |   X  |
|    du    |      X      |         |      |
|   echo   |             |         |   X  |
|    env   |             |         |   X  |
|  expand  |      X      |         |      |
|   expr   |      X      |         |      |
|   false  |             |         |   X  |
|  groups  |             |         |   X  |
|   hash   |      X      |         |      |
|   head   |      X      |         |      |
|    id    |             |         |   X  |
|  install |      X      |         |      |
|   join   |      X      |         |      |
|   link   |      X      |         |      |
|    ln    |      X      |         |      |
|  logname |             |         |   X  |
|    ls    |      X      |         |      |
|   mkdir  |      X      |         |      |
|  mkfifo  |      X      |         |      |
|    mv    |      X      |         |      |
|   nice   |             |         |   X  |
|    nl    |      X      |         |      |
|   nohup  |      X      |         |      |
|    od    |      X      |         |      |
|   paste  |      X      |         |      |
|   patch  |      X      |         |      |
|  printf  |      X      |         |      |
|    pwd   |             |         |   X  |
|    rm    |             |         |   x  |
|   rmdir  |      X      |         |      |
|    sed   |      X      |         |      |
|    seq   |      X      |         |      |
|   sort   |      X      |         |      |
|   sleep  |             |         |   X  |
|   split  |      X      |         |      |
|   stat   |      X      |         |      |
|   stty   |      X      |         |      |
|   tail   |      X      |         |      |
|    tee   |      X      |         |      |
|   test   |      X      |         |      |
|   time   |      X      |         |      |
|   touch  |      X      |         |      |
|    tr    |      X      |         |      |
|   true   |             |         |   X  |
|   tsort  |      X      |         |      |
|    tty   |             |         |   X  |
|   uname  |             |         |   X  |
| unexpand |      X      |         |      |
|   uniq   |      X      |         |      |
|  unlink  |      X      |         |      |
|  uptime  |      X      |         |      |
|   users  |      X      |         |      |
|    wc    |             |         |   X  |
|    who   |      X      |         |      |
|  whoami  |             |         |   X  |
|    yes   |             |         |   X  |


## Licensing
This software is licensed under the [Mozilla Public License, v. 2.0](./LICENSE). If a copy of the MPL was not distributed with this file, you can obtain one at http://mozilla.org/MPL/2.0/.


<!-- [![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2FGrayJack%2Fcoreutils.svg?type=large)](https://app.fossa.io/projects/git%2Bgithub.com%2FGrayJack%2Fcoreutils?ref=badge_large) -->
