use std::collections::HashSet;
use std::path::PathBuf;
use std::process::{Command, Stdio};

macro_rules! FMT {
    ($($x:expr),*) => {
        concat!("--format=", $("\u{2}", $x),*)
    }
}

/// Git log PRETTY FORMATS options.
/// %h  : abbreviated commit hash
/// %ar : author date, relative
/// %s  : subject
/// %D  : ref names without the " (", ")" wrapping.
const FMT_ARGS: &str = FMT!("%h", "%ar", "%s", "%C(auto)%D");

/// Gets the base `git log` command.
pub fn git_log() -> Command {
    let mut git = Command::new("git");
    git.args(["log", "--graph", "--color=always", FMT_ARGS]);
    git
}

/// Gets the `less` command. The `-R` flag to support color in the
/// output it scrolls. The `-F` flag tells `less` to quit if the
/// content is less than that of one screen.
pub fn less() -> Command {
    let mut less = Command::new("less");
    less.arg("-rF").stdin(Stdio::piped());
    less
}

/// Gets the base `git log` command.
pub fn git_dir() -> Option<PathBuf> {
    let mut git = Command::new("git");
    let _out = git.args(["rev-parse", "--git-dir"]).output().ok()?;
    let _str = std::str::from_utf8(&_out.stdout).ok()?;
    Some(PathBuf::from(_str.trim()))
}

pub fn verified_shas_raw() -> Option<String> {
    let v_file = git_dir()?.join(".verified");
    std::fs::read_to_string(v_file).ok()
}

pub fn verified_shas(raw: &str) -> HashSet<&str> {
    raw.lines().collect()
}
