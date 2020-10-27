# Coreutils in Rust

[![Hits-of-Code](https://hitsofcode.com/github/GrayJack/coreutils)](https://hitsofcode.com/view/github/GrayJack/coreutils)
[![Build Status](https://api.travis-ci.com/GrayJack/coreutils.svg?branch=master)](https://travis-ci.com/GrayJack/coreutils)
[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2FGrayJack%2Fcoreutils.svg?type=shield)](https://app.fossa.io/projects/git%2Bgithub.com%2FGrayJack%2Fcoreutils?ref=badge_shield)
<!-- [![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2FGrayJack%2Fcoreutils.svg?type=large)](https://app.fossa.io/projects/git%2Bgithub.com%2FGrayJack%2Fcoreutils?ref=badge_large) -->


An in-progress safe implementation of `coreutils` aiming for a minimal and yet complete set of utilities. This project aims at the _POSIX_ specification basic requirements, plus common and useful features present in other implementations of the utility.

This project has no intent to be 100% compatible with _GNU's coreutils_, like [Uutils' coreutils](https://github.com/uutils/coreutils), but if it happens to be, it's okay too.

## Minimum Rust Version Policy
This project's minimum supported `rustc` version (_MSRV_) is _1.45.0_.

This will not be conservative until we get to a _1.0_ version. So it can be changed at any point in time.

<!-- In general, this project will try to be conservative with respect to the minimum supported version of Rust, but in case of safety reasons it may bump at any time [e.g. `MaybeUninit` stabilization on 1.36.0 fixing huge problems with `std::mem::uninitialized()`] or improvements that affect positively conditional compilation (we use it a lot). -->

## Compilation tests guarantees
Compilation is tested for Rust Tier 1 and Tier 2 _x86_64 Unix/Unix-like_ platforms (except _Redox_), with _CI_.

All platforms are tested on _MSRV_ and _stable_ _Rust_, and Tier 1 platforms are also tested on _beta_ and _nightly_ _Rust_ (Tier 2 and 3 only guarantee _stable_ full capacity).

Also note that Tier 3 Rust tests can fail before it hits the compilation check, since my only option is to use a _Linux_ system, add the target and do `cargo check --target`, and they often are not available as a target for any Tier 1 platforms. If you use one of these platforms and have the right MSRV requirements and it build successfully, let me know, if it fails, open a issue with the compilation error.

|   Platform   |  Tier  |                                                                CI Status                                                                 |                    Manual Status                     |
| :----------: | :----: | :--------------------------------------------------------------------------------------------------------------------------------------: | :--------------------------------------------------: |
|    Linux     | Tier 1 |        [![Linux](https://github.com/GrayJack/coreutils/workflows/Linux/badge.svg)](https://github.com/GrayJack/coreutils/actions)        | Passing (Manjaro Linux 5.3.2 - 2019-10-28) (8695863) |
|    MacOS     | Tier 1 |        [![MacOS](https://github.com/GrayJack/coreutils/workflows/MacOS/badge.svg)](https://github.com/GrayJack/coreutils/actions)        |                          -                           |
|   FreeBSD    | Tier 2 |      [![FreeBSD](https://github.com/GrayJack/coreutils/workflows/FreeBSD/badge.svg)](https://github.com/GrayJack/coreutils/actions)      |    Passing (FreeBSD 12.0 - 2019-10-28) (8695863)     |
|    NetBSD    | Tier 2 |       [![NetBSD](https://github.com/GrayJack/coreutils/workflows/NetBSD/badge.svg)](https://github.com/GrayJack/coreutils/actions)       |          - (system without minimal version)          |
|   Solaris    | Tier 2 |      [![Solaris](https://github.com/GrayJack/coreutils/workflows/Solaris/badge.svg)](https://github.com/GrayJack/coreutils/actions)      |          - (system without minimal version)          |
|   Fuchsia    | Tier 2 |      [![Fuchsia](https://github.com/GrayJack/coreutils/workflows/Fuchsia/badge.svg)](https://github.com/GrayJack/coreutils/actions)      |                          -                           |
|   OpenBSD    | Tier 3 |      [![OpenBSD](https://github.com/GrayJack/coreutils/workflows/OpenBSD/badge.svg)](https://github.com/GrayJack/coreutils/actions)      | Passing (OpenBSD 6.6 Current - 2019-10-28) (8695863) |
| DragonflyBSD | Tier 3 | [![DragonflyBSD](https://github.com/GrayJack/coreutils/workflows/DragonflyBSD/badge.svg)](https://github.com/GrayJack/coreutils/actions) | Passing (DragonflyBSD 5.6.2 - 2019-10-28) (8695863)  |
|    Haiku     | Tier 3 |        [![Haiku](https://github.com/GrayJack/coreutils/workflows/Haiku/badge.svg)](https://github.com/GrayJack/coreutils/actions)        |                          -                           |

## Compilation
Since not all targets provide full _Unix_ API coverage (they aren't _Unix_ or lack _libc_ crate support), some can provide a `Cargo.toml` that have all utilities that should work on the target.

### Compilation example for
```sh
cp <PLATFORM>.toml Cargo.toml
cargo build --release
```

<!-- ### Install example
```sh
cp <Platform>.toml Cargo.toml
cargo install --path .
``` -->

## Tools
|   Name   | Not Started | Started | Done  |
| :------: | :---------: | :-----: | :---: |
| basename |             |         |   X   |
|   cat    |             |    X    |       |
|  chgrp   |      X      |         |       |
|  chmod   |      X      |         |       |
|  chown   |      X      |         |       |
|  chroot  |             |         |   X   |
|  clear   |             |         |   X   |
|   comm   |      X      |         |       |
|    cp    |      X      |         |       |
|  csplit  |             |    X    |       |
|   cut    |             |         |   X   |
|   date   |             |    X    |       |
|    dd    |      X      |         |       |
|    df    |      X      |         |       |
|   diff   |      X      |         |       |
| dirname  |             |         |   X   |
|    du    |             |         |   X   |
|   echo   |             |         |   X   |
|   env    |             |         |   X   |
|  expand  |             |         |   X   |
|   expr   |      X      |         |       |
|  false   |             |         |   X   |
|  groups  |             |         |   X   |
|   hash   |      X      |         |       |
|   head   |             |         |   X   |
|    id    |             |         |   X   |
| install  |      X      |         |       |
|   join   |      X      |         |       |
|   link   |             |         |   X   |
|    ln    |      X      |         |       |
| logname  |             |         |   X   |
|    ls    |      X      |         |       |
|  mkdir   |             |         |   X   |
|  mktemp  |             |         |   X   |
|  mkfifo  |      X      |         |       |
|    mv    |             |         |   X   |
|   nice   |             |         |   X   |
|    nl    |             |         |   X   |
|  nohup   |             |         |   X   |
|    od    |      X      |         |       |
|  paste   |      X      |         |       |
|  patch   |      X      |         |       |
|  printf  |      X      |         |       |
|   pwd    |             |         |   X   |
|    rm    |             |         |   x   |
|  rmdir   |             |         |   X   |
|   sed    |      X      |         |       |
|   seq    |             |    X    |       |
|   sort   |             |    X    |       |
|  sleep   |             |         |   X   |
|  split   |      X      |         |       |
|   stat   |      X      |         |       |
|   stty   |      X      |         |       |
|   tail   |      X      |         |       |
|   tee    |      X      |         |       |
|   test   |      X      |         |       |
|   time   |      X      |         |       |
|  touch   |             |    X    |       |
|    tr    |      X      |         |       |
|   true   |             |         |   X   |
|  tsort   |      X      |         |       |
|   tty    |             |         |   X   |
|  uname   |             |         |   X   |
| unexpand |             |         |   X   |
|   uniq   |             |    X    |       |
|  unlink  |             |         |   X   |
|  uptime  |             |         |   X   |
|  users   |             |         |   X   |
|    wc    |             |         |   X   |
|   who    |             |         |   X   |
|  whoami  |             |         |   X   |
|   yes    |             |         |   X   |


## Licensing
This software is licensed under the [Mozilla Public License, v. 2.0](./LICENSE) (MPL). If a copy of the MPL was not distributed with this file, you can obtain one at http://mozilla.org/MPL/2.0/.

## Contributing
We appreciate contributions, please check [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines on how to contribute to the project with issue reports, git commits messages, etc.

The document also gives orientation on where to start if you wanna implement a pending tool from scratch.

## Contributors
Without them this project would not be what it is today.

 * Ashwin-A-K
 * [@bojan88](https://github.com/bojan88) - _Bojan Đurđević_
 * [@Celeo](https://github.com/Celeo) - _Celeo_
 * [@FedericoPonzi](https://github.com/FedericoPonzi) - _Federico Ponzi_
 * [@Larisho](https://github.com/Larisho) - _Gab David_
 * [@silverweed](https://github.com/silverweed) - _Giacomo Parolini_
 * [@marcospb19](https://github.com/marcospb19) - _João M. Bezerra_
 * [@kegesch](https://github.com/kegesch) - _Jonas Geschke_
 * Ladysamantha
 * [@mkindahl](https://github.com/mkindahl) - _Mats Kindah_
 * [@MichelKansou](https://github.com/MichelKansou) - _Michel Kansou_
 * [@twe4ked](https://github.com/twe4ked) - _Odin Dutton_
 * [@rodrigocam](https://github.com/rodrigocam) - _Rodrigo Oliveira Campos_
 * [@Albibek](https://github.com/Albibek) - _Sergey Noskov_
 * [@palfrey](https://github.com/palfrey) - _Tom Parker-Shemilt_
 * [@tobbez](https://github.com/tobbez) - _Torbjörn Lönnemark_
 * [@vaibhav-y](https://github.com/vaibhav-y) - _Vaibhav Yenamandra_
 * [@muskuloes](https://github.com/muskuloes) - _Victor Tuekam_
