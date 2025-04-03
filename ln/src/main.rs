mod cmd;

use cmd::SP;

use std::io::{BufRead, BufReader, Write};
use std::process::Stdio;

macro_rules!write  {($f:ident,$($x:tt)+)=>{{let _=std::write  !($f,$($x)*);}}}
macro_rules!writeln{($f:ident,$($x:tt)+)=>{{let _=std::writeln!($f,$($x)*);}}}

const GRAY1: &str = "\x1b[38;5;240m";
const GRAY0: &str = "\x1b[38;5;246m";

const HEIGHT_RATIO: f32 = 0.7;

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
fn print_git_log_line<W: Write>(line: &str, mut f: W) {
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

// Gets the upper bound on number of lines to print on a bounded run.
fn get_line_limit() -> u32 {
    let (_, lines) = crossterm::terminal::size().unwrap();
    (lines as f32 * HEIGHT_RATIO) as u32
}

/// Iterates over the git log and writes the outputs to `f`.
fn run<R: BufRead, W: Write>(is_bounded: bool, mut log: R, mut target: W) {
    let mut buffer = String::with_capacity(256);
    let mut limit = if is_bounded { get_line_limit() } else { u32::MAX };

    while limit > 0 {
        buffer.clear();
        let line = match log.read_line(&mut buffer) {
            Ok(0) | Err(_) => break,
            _ => buffer.trim_end(),
        };
        print_git_log_line(line, &mut target);
        limit -= 1;
    }
}

/// Here, we operate under the assumption that we ARE using this in a
/// tty context, and hence always have color on.
fn main() {
    let mut git_log = cmd::git_log();
    git_log.stdout(Stdio::piped());

    let mut is_bounded = false;
    for arg in std::env::args_os().skip(1) {
        if arg == "--bound" {
            is_bounded = true;
            continue;
        }
        git_log.arg(arg);
    }

    let mut git_log_p = git_log.spawn().unwrap(); // process
    let git_log_s = git_log_p.stdout.take().unwrap(); // stdout
    let git_log_r = BufReader::new(git_log_s); // reader

    match cmd::less().spawn() {
        Ok(mut less) => {
            // `less` found: pass the git log output to less.
            run(is_bounded, git_log_r, less.stdin.take().unwrap());
            let _ = less.wait();
        }
        Err(_) => {
            // `less` not found: just run normal git log and print to stdout.
            run(is_bounded, git_log_r, std::io::stdout());
        }
    }
}
