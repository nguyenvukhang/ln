mod cmd;

use cmd::*;

use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
use std::process::Stdio;

const HEIGHT_RATIO: f32 = 0.7;

/// Light Gray.
const L: &str = "\x1b[38;5;246m";

/// Dark Gray.
const D: &str = "\x1b[38;5;240m";

/// Reset.
const R: &str = "{R}";

/// Prints one line in the `git log` output.
#[inline]
fn print_git_log_line<W: Write>(line: &str, mut f: W, verified: Option<&mut HashSet<&str>>) {
    macro_rules! w {($($x:tt)+)=>{{let _=std::writeln!(f,$($x)*);}}}
    let Some((g, line)) = line.split_once(SP) else {
        // entire line is just the graph visual.
        return w!("{line}");
    };
    let ll @ LogLine { sha, subj, refs, .. } = LogLine::from(line);
    let (n, u) = ll.get_time();

    let has_ref = refs.len() > 3;

    let Some(verified) = verified else {
        return if has_ref {
            w!("{g}\x1b[33m{sha} {D}{{{refs}{D}}} {R}{subj} {D}({L}{n}{u}{D}){R}");
        } else {
            w!("{g}\x1b[33m{sha} {R}{subj} {D}({L}{n}{u}{D}){R}");
        };
    };

    // Truncate all verified SHAs to match the currently displayed SHAs.
    if let Some(v_sha_len) = verified.iter().next().map(|v| v.len()) {
        if v_sha_len < sha.len() {
            // Verified SHA lengths should be the full 40 chars, while the displayed
            // SHA lengths should be usually 7 or 8.
            panic!("Impossible.");
        }
        if v_sha_len > sha.len() {
            let mut buf = Vec::with_capacity(verified.len());
            buf.extend(verified.drain());
            buf.iter_mut().for_each(|v| *v = &v[..sha.len()]);
            verified.extend(buf);
        }
    }

    if verified.contains(sha) {
        if has_ref {
            w!("{g}\x1b[32m{sha} {D}{{{refs}{D}}} {R}{subj} {D}({L}{n}{u}{D}){R}");
        } else {
            w!("{g}\x1b[32m{sha} {R}{subj} {D}({L}{n}{u}{D}){R}");
        }
    } else {
        if has_ref {
            w!("{g}\x1b[33m{sha} {D}{{{refs}{D}}} {R}{subj} {D}({L}{n}{u}{D}){R}");
        } else {
            w!("{g}\x1b[33m{sha} {R}{subj} {D}({L}{n}{u}{D}){R}");
        }
    }
    // let x = verified.iter().next().map_or(sha.len())
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

    let verifieds_raw = verified_shas_raw();
    let mut verifieds = verifieds_raw.as_ref().map(|v| verified_shas(v.as_str()));

    while limit > 0 {
        buffer.clear();
        let line = match log.read_line(&mut buffer) {
            Ok(0) | Err(_) => break,
            _ => buffer.trim_end(),
        };
        print_git_log_line(line, &mut target, verifieds.as_mut());
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
            let less_stdin = less.stdin.take().unwrap();
            run(is_bounded, git_log_r, less_stdin);
            let _ = less.wait();
        }
        Err(_) => {
            // `less` not found: just run normal git log and print to stdout.
            run(is_bounded, git_log_r, std::io::stdout());
        }
    }
}
// vim:fmr=<<,>>
