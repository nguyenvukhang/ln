use std::process::{Command, Stdio};

macro_rules! FMT {
    ($($x:expr),*) => {
        concat!("--format=", $("\u{2}", $x),*)
    }
}
pub const SP: &str = "\u{2}";

/// Gets the base `git log` command.
pub fn git_log() -> Command {
    let mut git = Command::new("git");
    git.args(["log", "--graph", "--color=always", FMT_ARGS]);
    git
}

/// Git log PRETTY FORMATS options.
/// %h  : abbreviated commit hash
/// %ar : author date, relative
/// %s  : subject
/// %D  : ref names without the " (", ")" wrapping.
const FMT_ARGS: &str = FMT!("%h", "%ar", "%s", "%C(auto)%D");

/// parse for `sha`, `time`, `subject`, `refs`.
#[inline]
pub fn parse_line(line: &str) -> (&str, &str, &str, &str) {
    let mut z = line.split(SP);
    (z.next().unwrap(), z.next().unwrap(), z.next().unwrap(), z.next().unwrap())
}

/// Gets the `less` command. The `-R` flag to support color in the
/// output it scrolls. The `-F` flag tells `less` to quit if the
/// content is less than that of one screen.
pub fn less() -> Command {
    let mut less = Command::new("less");
    less.arg("-RF").stdin(Stdio::piped());
    less
}
