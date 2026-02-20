#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DiffFile {
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub status: FileStatus,
    pub hunks: Vec<DiffHunk>,
}

impl DiffFile {
    /// The display path (new_path for renames/adds, old_path for deletes).
    pub fn path(&self) -> &str {
        self.new_path
            .as_deref()
            .or(self.old_path.as_deref())
            .unwrap_or("(unknown)")
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DiffHunk {
    pub old_start: u32,
    pub new_start: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Context,
    Added,
    Deleted,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub kind: LineKind,
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

/// Parse `git diff` output into per-file diffs (supports multiple hunks per file).
pub fn parse(raw: &str) -> Vec<DiffFile> {
    if raw.is_empty() {
        return Vec::new();
    }

    // Split on file boundaries. The first chunk before any "diff --git" is empty/header.
    let mut files = Vec::new();
    let mut chunks: Vec<&str> = Vec::new();
    let mut current_start = None;

    for (i, line) in raw.lines().enumerate() {
        if line.starts_with("diff --git ") {
            if let Some(start) = current_start {
                chunks.push(&raw[start..line_offset(raw, i)]);
            }
            current_start = Some(line_offset(raw, i));
        }
    }
    if let Some(start) = current_start {
        chunks.push(&raw[start..]);
    }

    for chunk in chunks {
        if let Some(file) = parse_file(chunk) {
            files.push(file);
        }
    }

    files
}

fn line_offset(raw: &str, line_idx: usize) -> usize {
    raw.lines()
        .take(line_idx)
        .map(|l| l.len() + 1) // +1 for newline
        .sum()
}

fn parse_file(chunk: &str) -> Option<DiffFile> {
    let lines: Vec<&str> = chunk.lines().collect();

    // Skip binary diffs
    if lines.iter().any(|l| l.starts_with("Binary files ")) {
        return None;
    }

    let mut old_path = None;
    let mut new_path = None;
    let mut hunk_start = None;

    for (i, line) in lines.iter().enumerate() {
        if let Some(p) = line.strip_prefix("--- ") {
            old_path = strip_prefix_path(p);
        } else if let Some(p) = line.strip_prefix("+++ ") {
            new_path = strip_prefix_path(p);
        } else if line.starts_with("@@ ") {
            hunk_start = Some(i);
            break;
        }
    }

    let status = match (&old_path, &new_path) {
        (None, Some(_)) => FileStatus::Added,
        (Some(_), None) => FileStatus::Deleted,
        (Some(o), Some(n)) if o != n => FileStatus::Renamed,
        _ => FileStatus::Modified,
    };

    let mut hunks = Vec::new();
    if let Some(first) = hunk_start {
        // Find all hunk boundaries and parse each
        let mut hunk_starts = vec![first];
        for (i, line) in lines.iter().enumerate().skip(first + 1) {
            if line.starts_with("@@ ") {
                hunk_starts.push(i);
            }
        }

        for (idx, &start) in hunk_starts.iter().enumerate() {
            let end = hunk_starts
                .get(idx + 1)
                .copied()
                .unwrap_or(lines.len());
            if let Some(hunk) = parse_hunk(&lines[start..end]) {
                hunks.push(hunk);
            }
        }
    }

    Some(DiffFile {
        old_path,
        new_path,
        status,
        hunks,
    })
}

fn strip_prefix_path(p: &str) -> Option<String> {
    if p == "/dev/null" {
        None
    } else {
        // Strip a/ or b/ prefix
        Some(
            p.strip_prefix("a/")
                .or_else(|| p.strip_prefix("b/"))
                .unwrap_or(p)
                .to_string(),
        )
    }
}

fn parse_hunk(lines: &[&str]) -> Option<DiffHunk> {
    let header = lines.first()?;
    let (old_start, new_start) = parse_hunk_header(header)?;

    let mut diff_lines = Vec::new();
    let mut old_no = old_start;
    let mut new_no = new_start;

    for &line in &lines[1..] {
        if line.starts_with("@@ ") {
            break;
        }
        if line == "\\ No newline at end of file" {
            continue;
        }

        let (kind, content) = if let Some(rest) = line.strip_prefix('+') {
            (LineKind::Added, rest)
        } else if let Some(rest) = line.strip_prefix('-') {
            (LineKind::Deleted, rest)
        } else if let Some(rest) = line.strip_prefix(' ') {
            (LineKind::Context, rest)
        } else {
            // Bare line (no prefix) — treat as context
            (LineKind::Context, line)
        };

        let (old_lineno, new_lineno) = match kind {
            LineKind::Context => {
                let o = old_no;
                let n = new_no;
                old_no += 1;
                new_no += 1;
                (Some(o), Some(n))
            }
            LineKind::Added => {
                let n = new_no;
                new_no += 1;
                (None, Some(n))
            }
            LineKind::Deleted => {
                let o = old_no;
                old_no += 1;
                (Some(o), None)
            }
        };

        diff_lines.push(DiffLine {
            kind,
            content: content.to_string(),
            old_lineno,
            new_lineno,
        });
    }

    Some(DiffHunk {
        old_start,
        new_start,
        lines: diff_lines,
    })
}

/// Parse `@@ -old_start,old_count +new_start,new_count @@` header.
fn parse_hunk_header(line: &str) -> Option<(u32, u32)> {
    // Format: @@ -1,5 +1,7 @@
    let after_at = line.strip_prefix("@@ ")?;
    let parts: Vec<&str> = after_at.splitn(3, ' ').collect();
    if parts.len() < 2 {
        return None;
    }

    let old = parts[0].strip_prefix('-')?;
    let new = parts[1].strip_prefix('+')?;

    let old_start = old.split(',').next()?.parse::<u32>().ok()?;
    let new_start_str = new.split(',').next()?;
    // new_start_str might have trailing @@ if no comma
    let new_start = new_start_str
        .trim_end_matches('@')
        .trim()
        .parse::<u32>()
        .ok()?;

    Some((old_start, new_start))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_modification() {
        let diff = "\
diff --git a/foo.rs b/foo.rs
index 1234..5678 100644
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,4 @@
 line1
+added
 line2
 line3
";
        let files = parse(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].status, FileStatus::Modified);
        assert_eq!(files[0].path(), "foo.rs");
        assert_eq!(files[0].hunks.len(), 1);

        let lines = &files[0].hunks[0].lines;
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0].kind, LineKind::Context);
        assert_eq!(lines[0].old_lineno, Some(1));
        assert_eq!(lines[0].new_lineno, Some(1));
        assert_eq!(lines[1].kind, LineKind::Added);
        assert_eq!(lines[1].old_lineno, None);
        assert_eq!(lines[1].new_lineno, Some(2));
        assert_eq!(lines[1].content, "added");
    }

    #[test]
    fn parse_new_file() {
        let diff = "\
diff --git a/new.txt b/new.txt
new file mode 100644
index 0000000..1234567
--- /dev/null
+++ b/new.txt
@@ -0,0 +1,2 @@
+hello
+world
";
        let files = parse(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].status, FileStatus::Added);
        assert_eq!(files[0].old_path, None);
        assert_eq!(files[0].new_path, Some("new.txt".into()));
    }

    #[test]
    fn parse_deleted_file() {
        let diff = "\
diff --git a/old.txt b/old.txt
deleted file mode 100644
index 1234567..0000000
--- a/old.txt
+++ /dev/null
@@ -1,2 +0,0 @@
-goodbye
-world
";
        let files = parse(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].status, FileStatus::Deleted);
        assert_eq!(files[0].old_path, Some("old.txt".into()));
        assert_eq!(files[0].new_path, None);
    }

    #[test]
    fn parse_multiple_files() {
        let diff = "\
diff --git a/a.txt b/a.txt
--- a/a.txt
+++ b/a.txt
@@ -1,1 +1,2 @@
 first
+second
diff --git a/b.txt b/b.txt
--- a/b.txt
+++ b/b.txt
@@ -1,2 +1,1 @@
 keep
-remove
";
        let files = parse(diff);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path(), "a.txt");
        assert_eq!(files[1].path(), "b.txt");
    }

    #[test]
    fn parse_multiple_hunks() {
        let diff = "\
diff --git a/foo.rs b/foo.rs
index 1234..5678 100644
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,4 @@
 line1
+added1
 line2
 line3
@@ -10,3 +11,4 @@
 line10
+added2
 line11
 line12
";
        let files = parse(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].hunks.len(), 2);

        let h1 = &files[0].hunks[0];
        assert_eq!(h1.old_start, 1);
        assert_eq!(h1.new_start, 1);
        assert_eq!(h1.lines.len(), 4);

        let h2 = &files[0].hunks[1];
        assert_eq!(h2.old_start, 10);
        assert_eq!(h2.new_start, 11);
        assert_eq!(h2.lines.len(), 4);
        assert_eq!(h2.lines[1].kind, LineKind::Added);
        assert_eq!(h2.lines[1].content, "added2");
    }

    #[test]
    fn parse_empty_input() {
        assert!(parse("").is_empty());
    }

    #[test]
    fn line_numbers_track_correctly() {
        let diff = "\
diff --git a/f.rs b/f.rs
--- a/f.rs
+++ b/f.rs
@@ -1,4 +1,4 @@
 ctx1
-old
+new
 ctx2
 ctx3
";
        let files = parse(diff);
        let lines = &files[0].hunks[0].lines;
        // ctx1: old=1, new=1
        assert_eq!((lines[0].old_lineno, lines[0].new_lineno), (Some(1), Some(1)));
        // -old: old=2, new=None
        assert_eq!((lines[1].old_lineno, lines[1].new_lineno), (Some(2), None));
        // +new: old=None, new=2
        assert_eq!((lines[2].old_lineno, lines[2].new_lineno), (None, Some(2)));
        // ctx2: old=3, new=3
        assert_eq!((lines[3].old_lineno, lines[3].new_lineno), (Some(3), Some(3)));
    }
}
