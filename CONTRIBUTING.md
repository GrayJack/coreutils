# Contributing to coreutils
🎉👍 First off, thanks for taking the time to contribute! 👍🎉

The following is a set of guidelines for contributing to coreutils. These are mostly guidelines, not rules. Use your best judgment, and feel free to propose changes to this document in a pull request.

## Table of contents

[Code of Conduct](#code-of-conduct)

[How Can I Contribute?](#how-can-i-contribute)
  * [Reporting Bugs](#reporting-bugs)
  * [Suggesting Enhancements](#suggesting-enhancements)
  * [Pull Requests](#pull-requests)

[Styleguides](#styleguides)
  * [Git Commit Messages](#git-commit-messages)
  * [Tools Styleguide](#tools-styleguide)
  * [Rust Styleguide](#rust-styleguide)
  * [Documentation Styleguide](#documentation-styleguide)

[References](#references)
  * [Man Pages (online)](#man-pages-online)

## Code of Conduct
Just don't be an pain in anyone's butt 😸

## How Can I Contribute?
### Reporting Bugs
In the Issues tracker, click in the button `New issue`, then select the `Bug Report` option. After that, add the required information about the bug in the template, and submit.

### Suggesting Enhancements
In the Issues tracker, click in the button `New issue`, then select the `Feature request` option. After that, add the required information about the bug in the template, and submit.

### Pull Requests
After forking and making your changes, always make a Pull Request to the `master` branch with a proper message saying what is changed.

## Styleguides
### Git Commit Messages
* First line should always start with the name of the tool that the code is being modified, followed by `:`.
* Reference issues and pull requests liberally after the first line.

**Git Commit Messages Template**
```
<TOOL/LIB>: <SUBMODULE>: <SUBSUBMODULE>: <MESSAGE>
```

**Git Commit Messages Example**
```
Core: Groups: Blablabla bla bla
```
```
Id: Implement flag '-a'
```

### Tools Styleguide
* Always use `clap` with `yaml` support to create the tools arguments.
* Always create a `build.rs` that creates every shell completions.
* Every argument have to have the `help` text.
* Every parameter have to have the `long` and `short` options.
* Every tool messages should follow this format: `<tool>: <message>` or `<tool>: <message>: <submessage>`
* Every tool should have exit code the same as the original tools

### Rust Styleguide
* Don't use nightly only features.
* The code must be formatted with this repository `rustfmt` configuration `rustfmt.toml`, otherwise the CI will fail.
  * Install the toolchain with: `rustup toolchain install nightly` and then format your code with `cargo +nightly fmt`. Be sure that `cargo +nightly fmt -- --check` doesn't print anything.
* Documentation should always be included when needed, for both functions, methods, modules, etc.
* Tests, when possible, should always be included/updated with your changes.
* Always comment what you're doing if it's not obvious, should be before the code that need explaining.

## References
### Manual pages online
* [Linux](https://www.linux.org/docs/index.html)
* [FreeBSD](https://www.freebsd.org/cgi/man.cgi)
* [NetBSD](https://netbsd.gw.com/cgi-bin/man-cgi?)
* [OpenBSD](https://man.openbsd.org/)
* [DragonflyBSD](http://man.dragonflybsd.org/?)
* [Illumos](https://illumos.org/man/)
* [Solaris]()
