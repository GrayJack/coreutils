#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum FileDescriptor {
    StdIn = 0,
    StdOut = 1,
    StdErr = 2,
}
