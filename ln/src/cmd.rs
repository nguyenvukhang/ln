use std::process::{Command, Stdio};

macro_rules! FMT {
    ($($x:expr),*) => {
        concat!("--format=", $("\u{2}", $x),*)
    }
}
pub(crate) const SP: &str = "\u{2}";

/// Gets the base `git log` command.
pub(crate) fn git_log() -> Command {
    let mut git = Command::new("git");
    git.args([
        "log",
        "--graph",
        "--color=always",
        FMT!("%h", "%ar", "%s", "%C(auto)%D"),
    ]);
    git
}

/// Gets the `less` command. The `-R` flag to support color in the
/// output it scrolls. The `-F` flag tells `less` to quit if the
/// content is less than that of one screen.
pub(crate) fn less() -> Command {
    let mut less = Command::new("less");
    less.arg("-RF").stdin(Stdio::piped());
    less
}
