mod cmd;

use cmd::{LogLine, SP};

use std::io::{BufRead, BufReader, Write};
use std::process::Stdio;

const HEIGHT_RATIO: f32 = 0.7;

/// Light Gray.
const L: &str = "\x1b[38;5;246m";

/// Dark Gray.
const D: &str = "\x1b[38;5;240m";

/// Prints one line in the `git log` output.
#[inline]
fn print_git_log_line<W: Write>(line: &str, mut f: W) {
    macro_rules! w {($($x:tt)+)=>{{let _=std::writeln!(f,$($x)*);}}}
    let Some((g, line)) = line.split_once(SP) else {
        // entire line is just the graph visual.
        return w!("{line}");
    };
    let ll @ LogLine { sha, subj, refs, .. } = LogLine::from(line);
    let (n, u) = ll.get_time();

    if refs.len() > 3 {
        w!("{g}\x1b[33m{sha} {D}{{{refs}{D}}} \x1b[m{subj} {D}({L}{n}{u}{D})\x1b[m");
    } else {
        w!("{g}\x1b[33m{sha} \x1b[m{subj} {D}({L}{n}{u}{D})\x1b[m");
    }
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
    println!("{:?}", cmd::git_dir());
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
// vim:fmr=<<,>>
