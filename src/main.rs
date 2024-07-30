use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MAX_COMMENT_LEN: usize = 60;
const FORMAT: &str = concat!("--format=", "::::%h::::%d::::%s::::%at");

#[derive(Debug)]
struct Commit<'a> {
    graph: &'a str,
    hash: &'a str,
    comment: &'a str,
    refs: Option<String>,
    timestamp: u64,
}

fn parse_commit(line: &str) -> Option<Commit> {
    let mut iter = line.splitn(5, "::::");
    let graph = iter.next()?.trim_end();
    let hash = iter.next()?.trim_end();
    let refs = iter.next()?.trim();
    let comment = iter.next()?;
    let timestamp = iter.next()?;
    Some(Commit {
        graph,
        hash,
        refs: (!refs.is_empty())
            .then(|| refs.replace("origin/", "*").replace("->", "→")),
        comment,
        timestamp: timestamp.parse().unwrap(),
    })
}

#[rustfmt::skip]
fn print_readable_duration(d: Duration)  {
    let mut n = d.as_secs();
    if n < 60 { return print!("{n}s") } n /= 60;
    if n < 60 { return print!("{n}m") } n /= 60;
    if n < 24 { return print!("{n}h") } n /= 24;
    if n < 7  { return print!("{n}d") } print!("{}w", n / 7)
}

fn main() {
    let mut cmd = Command::new("git");
    // cmd.args(["-C", "/Users/khang/repos/math"]); // for debugging
    cmd.args(["log", "--graph", FORMAT]);
    cmd.arg("--color=always"); // for colors in the graph visualization
    cmd.args(std::env::args().skip(1));

    cmd.stdout(Stdio::piped());
    let output = match cmd.output() {
        Ok(v) => v,
        Err(e) => return println!("Error running git: {e}"),
    };
    let git_log_text = match std::str::from_utf8(&output.stdout) {
        Ok(v) => v,
        Err(e) => return println!("Error parsing git log: {e}"),
    };

    let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(v) => v.as_secs(),
        Err(e) => return println!("Error getting system time: {e}"),
    };
    let elapsed = |secs: u64| Duration::from_secs(now - secs);

    let mut stdout = std::io::stdout().lock();

    for line in git_log_text.lines() {
        let c = match parse_commit(line) {
            Some(v) => v,
            None => {
                let _ = writeln!(stdout, "{line}");
                continue;
            }
        };
        let Commit { graph, hash, comment, refs, timestamp } = c;

        let _ = write!(stdout, "{graph} \x1b[33m{hash}\x1b[0m");

        let _ = match comment.len() {
            n if n <= MAX_COMMENT_LEN => write!(stdout, " {comment}"),
            _ => write!(stdout, " {}...", &comment[..MAX_COMMENT_LEN - 3]),
        };

        let _ = write!(stdout, " \x1b[38;5;241m(\x1b[38;5;246m");
        print_readable_duration(elapsed(timestamp));
        let _ = write!(stdout, "\x1b[38;5;241m)\x1b[0m");

        let _ = match refs {
            Some(refs) => writeln!(stdout, " \x1b[37m{refs}\x1b[0m"),
            None => writeln!(stdout),
        };
    }
}
