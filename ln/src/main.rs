use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

macro_rules!write  {($f:ident,$($x:tt)+)=>{std::write  !($f,$($x),*).unwrap()}}
macro_rules!writeln{($f:ident,$($x:tt)+)=>{std::writeln!($f,$($x),*).unwrap()}}

const GRAY1: &str = "\x1b[38;5;240m";
const GRAY0: &str = "\x1b[38;5;246m";

const SP: &str = "\u{2}";
const FMT_ARGS: [&str; 4] = ["%h", "%ar", "%s", "%C(auto)%D"];

/// parse for `sha`, `time`, `subject`, `refs`.
#[inline]
fn parse_line(line: &str) -> (&str, &str, &str, &str) {
    let (sha, line) = line.split_once(SP).unwrap();
    let (time, line) = line.split_once(SP).unwrap();
    let (subject, refs) = line.split_once(SP).unwrap();
    (sha, time, subject, refs)
}

/// Prints one line in the `git log` output.
#[inline]
fn handle_git_log_stdout_line<W: Write>(line: &str, mut f: W) {
    let Some((graph, line)) = line.split_once(SP) else {
        // entire line is just the graph visual.
        return writeln!(f, "{line}");
    };
    let (sha, time, subject, refs) = parse_line(line);

    write!(f, "{graph}\x1b[33m{sha} ");
    if refs.len() > 3 {
        write!(f, "{GRAY1}{{{refs}{GRAY1}}} ");
    }
    write!(f, "\x1b[m{subject} {GRAY1}({GRAY0}");
    {
        let (n, t) = time.split_once(' ').unwrap();
        let t =
            if t.starts_with("mo") { 'M' } else { t.chars().next().unwrap() };
        write!(f, "{n}{t}");
    }
    writeln!(f, "{GRAY1})\x1b[m");
}

/// Iterates over the git log and writes the outputs to `f`.
fn run<R: BufRead, W: Write>(mut git_log: R, mut target: W) {
    let mut buffer = String::with_capacity(256);
    loop {
        buffer.clear();
        let line = match git_log.read_line(&mut buffer) {
            Ok(0) | Err(_) => break,
            _ => buffer.trim_end(),
        };
        handle_git_log_stdout_line(line, &mut target);
    }
}

/// Gets the `git log` command. Forwards all arguments passed to this
/// binary on to `git log`.
fn git_log() -> Command {
    let mut git = Command::new("git");
    git.arg("log").args(std::env::args().skip(1)).arg("--graph");
    git.arg(format!("--format={SP}{}", FMT_ARGS.join(SP)));
    git.arg("--color=always");
    git.stdout(Stdio::piped());
    git
}

/// Gets the `less` command. The `-R` flag to support color in the
/// output it scrolls. The `-F` flag tells `less` to quit if the
/// content is less than that of one screen.
fn less() -> Command {
    let mut less = Command::new("less");
    less.arg("-RF").stdin(Stdio::piped());
    less
}

/// Here, we operate under the assumption that we ARE using this in a
/// tty context, and hence always have color on.
fn main() {
    let git_log = git_log().spawn().unwrap().stdout.take().unwrap();
    let git_log = BufReader::new(git_log);

    let Ok(mut less) = less().spawn() else {
        return run(git_log, std::io::stdout());
    };

    run(git_log, less.stdin.take().unwrap());
    let _ = less.wait();
}
