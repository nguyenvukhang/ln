use std::collections::HashSet;

macro_rules! next {
    ($z:expr) => {
        $z.next().unwrap()
    };
}

pub struct LogLine<'a> {
    pub sha: &'a str,
    pub time: &'a str,
    pub subj: &'a str,
    pub refs: &'a str,
}

pub const SP: &str = "\u{2}";

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

    /// Whether or not this log line contains any git refs/branches.
    pub fn has_refs(&self) -> bool {
        self.refs.len() > 3
    }

    pub fn is_verified(&self, verified: Option<&mut HashSet<&str>>) -> bool {
        let Some(verified) = verified else { return false };
        truncate_verified_shas(verified, self.sha.len());
        verified.contains(self.sha)
    }
}

/// Truncate all verified SHAs to match the currently displayed SHAs.
fn truncate_verified_shas(verified: &mut HashSet<&str>, len: usize) {
    let Some(v_sha_len) = verified.iter().next().map(|v| v.len()) else { return };
    if v_sha_len < len {
        // Verified SHA lengths should be the full 40 chars, while the displayed
        // SHA lengths should be usually 7 or 8.
        panic!("Impossible.");
    }
    if v_sha_len > len {
        let mut buf = Vec::with_capacity(verified.len());
        buf.extend(verified.drain());
        buf.iter_mut().for_each(|v| *v = &v[..len]);
        verified.extend(buf);
    }
}
