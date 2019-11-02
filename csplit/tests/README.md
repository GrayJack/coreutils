# Test for `csplit`

This directory contains test cases that can be used to compare this
implementation of `csplit` with your default
implementation. Typically, you can execute the test by running

```bash
cd coreutils/csplit/tests
bash runall.sh ../../target/debug/csplit
```

It is also possible to run a specific test by providing it on the
command line, for example:

```bash
cd coreutils/csplit/tests
bash runall.sh ../../target/debug/csplit test-1.sh
```

Each test is a shell script where the environment variable `CSPLIT` is
set to either the default installation of `csplit` or another
implementation of `csplit` (such as the implementation that we are
developing here).

The current directory of the test is set to a temporary directory so
any files produced will be written there.

In addition to any files the script produces, two extra files will be
written by the test runner: `stdout.txt`, `stderr.txt`, and
`exit.txt`. The `stdout.txt` and `stderr.txt` contain the output
written to `stdout` and `stderr` respectively, and `exit.txt` contain
the exit code of the application.

# Notes

* You have to stand in this directory when executing the `runall.sh`
* A difference between the executions does not necessarily indicate a
  failure.
  * Error messages might be different between the two utilities.
  * The byte count output might differ if there is an error.
  * Use your judgement to decide what is relevant.
