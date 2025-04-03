use std::process::{Command, Stdio};

macro_rules! FMT {
    ($($x:expr),*) => {
        concat!("--format=", $("\u{2}", $x),*)
    }
}
macro_rules! next {
    ($z:expr) => {
        $z.next().unwrap()
    };
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

impl<'a> From<&'a str> for LogLine<'a> {
    fn from(z: &'a str) -> Self {
        let mut z = z.split(SP);
        Self { sha: next!(z), time: next!(z), subj: next!(z), refs: next!(z) }
    }
}

impl<'a> LogLine<'a> {
    /// Get `(n, u)` where n is the number and u is the units.
    pub fn get_time(&self) -> (&'a str, char) {
        let (n, u) = self.time.split_once(' ').unwrap();
        (n, if u.starts_with("mo") { 'M' } else { u.chars().next().unwrap() })
    }
}

pub struct LogLine<'a> {
    pub sha: &'a str,
    pub time: &'a str,
    pub subj: &'a str,
    pub refs: &'a str,
}

/// Gets the `less` command. The `-R` flag to support color in the
/// output it scrolls. The `-F` flag tells `less` to quit if the
/// content is less than that of one screen.
pub fn less() -> Command {
    let mut less = Command::new("less");
    less.arg("-RF").stdin(Stdio::piped());
    less
}
