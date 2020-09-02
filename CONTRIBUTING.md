# Contributing to coreutils
🎉👍 First off, thanks for taking the time to contribute! 👍🎉

The following is a set of guidelines for contributing to `coreutils`. These are mostly guidelines, not rules. Use your best judgment and feel free to propose changes to this document in a pull request.

If you want to implement a tool from scratch, surely read the [**Creating a Tool**](#creating-a-tool) section for orientation on where to start from.

## Table of contents
- [Contributing to coreutils](#contributing-to-coreutils)
  - [Table of contents](#table-of-contents)
  - [Code of Conduct](#code-of-conduct)
  - [Reporting Bugs Or Suggesting Enhancements](#reporting-bugs-or-suggesting-enhancements)
  - [Pull Requests](#pull-requests)
  - [Creating a Tool](#creating-a-tool)
  - [Rust Style Guide](#rust-style-guide)
  - [Git Commit Messages](#git-commit-messages)
  - [Useful References](#useful-references)
    - [Man Pages Online](#man-pages-online)
    - [Rust Current Version Status On Tier 3](#rust-current-version-status-on-tier-3)

## Code of Conduct
The Code of Conduct for this repository [here](./CODE_OF_CONDUCT.md)

## Reporting Bugs Or Suggesting Enhancements
In the [Issues tab](https://github.com/GrayJack/coreutils/issues), click `New issue`, select `Bug report` or `Feature request`, then add the required information and submit it.

## Pull Requests
Always direct the Pull Request to the `dev` branch, try to be as descriptive as possible.

## Creating a Tool
To start a tool, copy the template:
```sh
cp .template -r <TOOL_NAME>
```

Now you can start editing files inside of the new directory (_TOOL_NAME_)

Here is a list of steps you should follow (not necessarily in order):
- Replace _"template"_ occurrences in [`build.rs`](.template/build.rs) by the TOOL_NAME.
- Make changes in [`Cargo.toml`](.template/Cargo.toml) indicated fields.
- Configure argument parsing with `clap` at [`cli.rs`](.template/src/cli.rs), here are some rules:
  - Every `Arg` needs a `.help()`.
  - About the flags:
    - Every flag needs, `.long()` and `.short()` options
    - If the option is specified by the [POSIX standard](https://pubs.opengroup.org/onlinepubs/9699919799/idx/utilities.html), use the short flag option specified.
    - For long options and short options that are not specified by the [POSIX standard](https://pubs.opengroup.org/onlinepubs/9699919799/idx/utilities.html), the flag should match other famous coreutils implementations names (if needed, use _clap_'s `.visible_alias()` or `.visible_aliases()` to fill this requirement) renaming long options can be discussed in the PR.
- Every help message should start with upper case letter and end with a dot.
- Exit code is defined by the [POSIX standard](https://pubs.opengroup.org/onlinepubs/9699919799/idx/utilities.html) (extra customization might be required for extensions).
- We recommend that you create a Pull Request with check boxes (see an [example](https://github.com/GrayJack/coreutils/pull/121)).
- Update the README with the new status of the tool and contributors section.
- Create your PR.

You can create an in-progress PR, as well as a completed one.

Use the PR to debate, ask questions and ask for reviews (don't forget to be respectful and follow the [Code of Conduct](./CODE_OF_CONFUCT.md)).


## Rust Style Guide
- Don't use nightly-only features.
- The code must be formatted by `rustfmt` using `rustfmt.toml` configuration, otherwise the CI will fail.
  - Install the _toolchain_ with: `rustup toolchain install nightly`, format your code with `cargo +nightly fmt` and be sure that `cargo +nightly fmt -- --check` doesn't print anything.
- Main function should be the first block after global use statements and module statements (not module blocks).
- Documentation should always be included when needed, for both functions, methods, modules, etc.
- Tests, when possible, should always be included/updated with your changes.
- Always comment what you're doing if it's not obvious, should be before the code that need explaining.
- Try to be conservative about dependencies, only add if it is very necessary or if it will add good amount of ergonomics with few sub-dependencies. (you can check the dependency tree using `cargo-tree` crate)
  - As for dependencies versions, use `"~<Version>"` for crates below 1.0 and `"^<Version>"` for crates above 1.0.
- Avoid unsafe Rust in the tools code, if necessary, add a function in `coreutils_core` crate with necessary abstractions.

## Git Commit Messages
- First line should start with the name of the tool that is being modified, followed by `:`.
- The second line is empty.
- Reference issues and pull requests goes on the description after the second line.

**Git Commit Messages Template:**

Format if you're editing submodule:
```
<TOOL/LIB>: <SUBMODULE>: <SUBSUBMODULE>: <DESCRIPTION>
```

If there's no submodule:
```
<TOOL/LIB>: <DESCRIPTION>
```

**Examples:**:

```
Mkdir: Added --mode option
```

```
Mkdir: add nested use imports and log error messages to stderr
```

```
Core: mkfifo: simplify the logic of mkfifo function
```

## Useful References
### Man Pages Online
- [POSIX Utilities Specification](https://pubs.opengroup.org/onlinepubs/9699919799/idx/utilities.html)
- [Linux](https://www.linux.org/docs/index.html)
- [FreeBSD](https://www.freebsd.org/cgi/man.cgi)
- [NetBSD](https://netbsd.gw.com/cgi-bin/man-cgi?)
- [OpenBSD](https://man.openbsd.org/)
- [DragonflyBSD](http://man.dragonflybsd.org/?)
- [Illumos](https://illumos.org/man/)
- [Solaris]()

### Rust Current Version Status On Tier 3
- [OpenBSD](http://openports.se/lang/rust)
- [DragonflyBSD](https://github.com/DragonFlyBSD/DPorts/tree/master/lang/rust)
- [Haiku](https://depot.haiku-os.org/#!/pkg/rust_bin/haikuports/1/36/0/-/1/x86_64?bcguid=bc115-DPXR)
