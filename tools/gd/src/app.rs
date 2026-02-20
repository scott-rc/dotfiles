use crate::git::diff::DiffFile;

#[derive(Debug, Clone)]
pub enum DiffSource {
    WorkingTree,
    Staged,
    Commit(String),
    Range(String, String),
}

impl DiffSource {
    /// Git diff args for this source (includes `--unified=999999`).
    pub fn diff_args(&self) -> Vec<String> {
        let mut args = vec!["diff".into(), "--unified=999999".into()];
        match self {
            Self::WorkingTree => {}
            Self::Staged => args.push("--staged".into()),
            Self::Commit(r) => {
                args.push(format!("{r}~1"));
                args.push(r.clone());
            }
            Self::Range(l, r) => {
                args.push(l.clone());
                args.push(r.clone());
            }
        }
        args
    }

    /// Args for `git show` to get the new-side content, or None for `WorkingTree`.
    pub fn show_args(&self, path: &str) -> Option<Vec<String>> {
        match self {
            Self::WorkingTree => None,
            Self::Staged => Some(vec!["show".into(), format!(":{path}")]),
            Self::Commit(r) | Self::Range(_, r) => {
                Some(vec!["show".into(), format!("{r}:{path}")])
            }
        }
    }
}

pub struct FileList {
    pub entries: Vec<DiffFile>,
    pub current: usize,
}

impl FileList {
    pub fn new(entries: Vec<DiffFile>) -> Self {
        Self {
            entries,
            current: 0,
        }
    }

    pub fn current(&self) -> &DiffFile {
        &self.entries[self.current]
    }

    /// Advance to next file. Returns true if changed.
    pub fn next(&mut self) -> bool {
        if self.current + 1 < self.entries.len() {
            self.current += 1;
            true
        } else {
            false
        }
    }

    /// Go to previous file. Returns true if changed.
    pub fn prev(&mut self) -> bool {
        if self.current > 0 {
            self.current -= 1;
            true
        } else {
            false
        }
    }
}

/// Resolve CLI args into a `DiffSource`.
pub fn resolve_source(staged: bool, source: &[String]) -> DiffSource {
    if staged {
        return DiffSource::Staged;
    }
    match source {
        [] => DiffSource::WorkingTree,
        [arg] => {
            if arg.contains("..") {
                let parts: Vec<&str> = arg.splitn(2, "..").collect();
                DiffSource::Range(parts[0].into(), parts[1].into())
            } else {
                DiffSource::Commit(arg.clone())
            }
        }
        [left, right, ..] => DiffSource::Range(left.clone(), right.clone()),
    }
}
