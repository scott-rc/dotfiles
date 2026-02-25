// GitHub Dark-inspired diff color palette.
// Line backgrounds are subtle tints; word-level highlights are brighter.

/// Added line background (subtle green tint)
pub const BG_ADDED: &str = "\x1b[48;2;22;39;27m";
/// Deleted line background (subtle red tint)
pub const BG_DELETED: &str = "\x1b[48;2;50;24;24m";
/// Added word highlight (brighter green)
pub const BG_ADDED_WORD: &str = "\x1b[48;2;38;68;42m";
/// Deleted word highlight (brighter red)
pub const BG_DELETED_WORD: &str = "\x1b[48;2;85;36;36m";

/// Visual selection background (subtle blue-violet tint)
pub const BG_VISUAL: &str = "\x1b[48;2;40;50;80m";

/// Tree cursor background (subtle blue-gray tint)
pub const BG_TREE_CURSOR: &str = "\x1b[48;2;42;53;69m";
/// Cursor line background (subtle blue tint)
pub const BG_CURSOR: &str = "\x1b[48;2;36;46;62m";

/// File header foreground (bold blue)
pub const FG_FILE_HEADER: &str = "\x1b[1;38;2;121;192;255m";
/// Line number gutter (dim gray)
pub const FG_GUTTER: &str = "\x1b[2;38;2;110;118;129m";
/// Separator character (dim gray)
pub const FG_SEP: &str = "\x1b[2;38;2;68;76;86m";
/// File tree text (near-white, no DIM to avoid bleed)
pub const FG_TREE: &str = "\x1b[38;2;201;209;217m";
/// File tree directory names (folder yellow)
pub const FG_TREE_DIR: &str = "\x1b[38;2;224;177;77m";
/// File tree connector lines (gray, matches lsd tree-edge #484f58)
pub const FG_TREE_GUIDE: &str = "\x1b[38;2;72;79;88m";
/// Added line marker foreground
pub const FG_ADDED_MARKER: &str = "\x1b[38;2;63;185;80m";
/// Deleted line marker foreground
pub const FG_DELETED_MARKER: &str = "\x1b[38;2;248;81;73m";
/// Tree status: modified (yellow/orange)
pub const FG_STATUS_MODIFIED: &str = "\x1b[38;2;210;153;34m";
/// Tree status: added (green, matches FG_ADDED_MARKER)
pub const FG_STATUS_ADDED: &str = "\x1b[38;2;63;185;80m";
/// Tree status: deleted (red, matches FG_DELETED_MARKER)
pub const FG_STATUS_DELETED: &str = "\x1b[38;2;248;81;73m";
/// Tree status: renamed (blue/cyan)
pub const FG_STATUS_RENAMED: &str = "\x1b[38;2;121;192;255m";
/// Tree status: untracked (dim gray)
pub const FG_STATUS_UNTRACKED: &str = "\x1b[38;2;110;118;129m";
/// Scrollbar track background (very dim gray)
pub const BG_SCROLLBAR_TRACK: &str = "\x1b[48;2;30;33;38m";
/// Scrollbar viewport thumb background (brighter gray)
pub const BG_SCROLLBAR_THUMB: &str = "\x1b[48;2;55;60;68m";
/// Status bar background (dark blue-gray)
pub const STATUS_BG: &str = "\x1b[48;2;28;33;40m";
/// Status bar foreground (muted gray)
pub const STATUS_FG: &str = "\x1b[38;2;139;148;158m";

pub const RESET: &str = "\x1b[0m";

/// Wrap continuation marker for long lines.
pub fn wrap_marker(color: bool) -> String {
    if color {
        format!("{FG_GUTTER}\u{21aa}{RESET}")
    } else {
        "\u{21aa}".to_string()
    }
}
/// Reset bold/italic/fg but preserve background color.
pub const SOFT_RESET: &str = "\x1b[22;23;39m";
pub const DIM: &str = "\x1b[2m";
pub const NO_DIM: &str = "\x1b[22m";

/// Format a file header separator line.
pub fn file_header(path: &str, status: &str, width: usize) -> String {
    let label = format!(" {path} ({status}) ");
    let bar_len = width.saturating_sub(2 + label.len());
    let left = 2;
    let right = bar_len;
    format!(
        "{FG_FILE_HEADER}{}{label}{}{RESET}",
        "\u{2500}".repeat(left),
        "\u{2500}".repeat(right),
    )
}

/// Format a hunk separator line (dim dashed line between hunks).
pub fn hunk_separator(width: usize, color: bool) -> String {
    if color {
        format!("{FG_SEP}{}{RESET}", "\u{2508}".repeat(width))
    } else {
        "\u{00B7}".repeat(width)
    }
}

/// Format the dual line-number gutter.
pub fn gutter(old: Option<u32>, new: Option<u32>) -> String {
    let old_str = old.map_or(String::new(), |n| format!("{n}"));
    let new_str = new.map_or(String::new(), |n| format!("{n}"));
    format!(
        "{FG_GUTTER}{old_str:>4} {FG_SEP}\u{2502}{FG_GUTTER}{new_str:>4} {FG_SEP}\u{2502}{RESET}"
    )
}

/// The visible width of the gutter (always 12: "NNNN |NNNN |").
pub const GUTTER_WIDTH: usize = 12;

/// Blank gutter for continuation lines (same width, dim separators).
pub fn continuation_gutter(color: bool) -> String {
    if color {
        format!("{FG_GUTTER}     {FG_SEP}\u{2502}     {FG_SEP}\u{2502}{RESET}")
    } else {
        "     |     |".to_string()
    }
}

/// Return (icon, ansi_color) for a file path based on extension.
pub fn file_icon(path: &str) -> (&'static str, &'static str) {
    let ext = path.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" => ("\u{e7a8}", "\x1b[38;2;222;135;53m"), // Rust orange
        "ts" | "tsx" => ("\u{e628}", "\x1b[38;2;49;120;198m"), // TypeScript blue
        "js" | "jsx" => ("\u{e781}", "\x1b[38;2;241;224;90m"), // JavaScript yellow
        "go" => ("\u{e627}", "\x1b[38;2;0;173;216m"),  // Go cyan
        "py" => ("\u{e73c}", "\x1b[38;2;55;118;171m"), // Python blue
        "md" => ("\u{e73e}", "\x1b[38;2;81;154;186m"), // Markdown teal
        "json" => ("\u{e60b}", "\x1b[38;2;241;224;90m"), // JSON yellow
        "toml" => ("\u{e6b2}", "\x1b[38;2;139;148;158m"), // TOML gray
        "yaml" | "yml" => ("\u{e6a8}", "\x1b[38;2;203;75;83m"), // YAML red
        "sh" | "fish" | "bash" | "zsh" => ("\u{e795}", "\x1b[38;2;137;224;81m"), // Shell green
        "lua" => ("\u{e620}", "\x1b[38;2;81;160;207m"), // Lua blue
        "html" => ("\u{e736}", "\x1b[38;2;228;77;38m"), // HTML orange
        "css" => ("\u{e749}", "\x1b[38;2;86;156;214m"), // CSS blue
        "lock" => ("\u{f023}", "\x1b[38;2;139;148;158m"), // Lock gray
        _ => ("\u{f15b}", FG_TREE),                    // Generic file
    }
}

/// Return (icon, ansi_color) for a directory entry.
pub fn dir_icon(collapsed: bool) -> (&'static str, &'static str) {
    if collapsed {
        ("\u{f4d8}", FG_TREE_DIR)
    } else {
        ("\u{f413}", FG_TREE_DIR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::strip_ansi;

    #[test]
    fn test_file_header_basic() {
        let out = strip_ansi(&file_header("foo.rs", "M", 40));
        assert!(out.starts_with("──"), "should start with ──: {out:?}");
        assert!(
            out.contains("foo.rs (M)"),
            "should contain path and status: {out:?}"
        );
        assert!(out.ends_with("──"), "should end with ──: {out:?}");
    }

    #[test]
    fn test_file_header_narrow_width() {
        let out = file_header("path", "A", 5);
        let visible = strip_ansi(&out);
        assert!(
            visible.contains("path (A)"),
            "label still present: {visible:?}"
        );
    }

    #[test]
    fn test_file_header_wide() {
        let out = strip_ansi(&file_header("short", "D", 80));
        let bar_chars: usize = out.chars().filter(|&c| c == '─').count();
        let label = " short (D) ";
        assert_eq!(
            bar_chars,
            80 - label.len(),
            "bar chars should fill remaining width"
        );
    }

    #[test]
    fn test_gutter_both_some() {
        let out = strip_ansi(&gutter(Some(1), Some(2)));
        assert!(out.contains('1'), "should contain old line number: {out:?}");
        assert!(out.contains('2'), "should contain new line number: {out:?}");
        assert!(out.contains('│'), "should contain separator: {out:?}");
        assert_eq!(
            out.chars().count(),
            GUTTER_WIDTH,
            "visible width should be {GUTTER_WIDTH}"
        );
    }

    #[test]
    fn test_gutter_both_none() {
        let out = strip_ansi(&gutter(None, None));
        assert_eq!(out, "     │     │", "blank gutter with separators");
    }

    #[test]
    fn test_gutter_large_numbers() {
        let out = strip_ansi(&gutter(Some(9999), Some(100)));
        assert!(out.contains("9999"), "should contain 9999: {out:?}");
        assert!(out.contains("100"), "should contain 100: {out:?}");
    }

    #[test]
    fn test_file_icon_rust() {
        assert_eq!(file_icon("src/main.rs").0, "\u{e7a8}");
    }

    #[test]
    fn test_file_icon_all_extensions() {
        let default_icon = "\u{f15b}";
        let exts = [
            "rs", "ts", "tsx", "js", "jsx", "go", "py", "md", "json", "toml", "yaml", "yml", "sh",
            "fish", "bash", "zsh", "lua", "html", "css", "lock",
        ];
        for ext in exts {
            let path = format!("x.{ext}");
            let (icon, _) = file_icon(&path);
            assert_ne!(
                icon, default_icon,
                "extension {ext:?} should have a specific icon"
            );
        }
    }

    #[test]
    fn test_file_icon_fallback() {
        let default_icon = "\u{f15b}";
        for path in ["noext", ".gitignore", "a.b.c"] {
            let (icon, _) = file_icon(path);
            assert_eq!(
                icon, default_icon,
                "{path:?} should return the default icon"
            );
        }
    }

    #[test]
    fn test_dir_icon_collapsed_expanded() {
        assert_eq!(dir_icon(true).0, "\u{f4d8}", "collapsed icon");
        assert_eq!(dir_icon(false).0, "\u{f413}", "expanded icon");
        assert_eq!(dir_icon(true).1, FG_TREE_DIR, "collapsed color");
        assert_eq!(dir_icon(false).1, FG_TREE_DIR, "expanded color");
    }

    #[test]
    fn test_wrap_marker() {
        let colored = wrap_marker(true);
        assert!(
            colored.contains("\u{21aa}"),
            "colored marker contains arrow"
        );
        assert!(colored.contains(RESET), "colored marker contains RESET");
        let plain = wrap_marker(false);
        assert_eq!(plain, "\u{21aa}", "plain marker is just the arrow");
        assert_eq!(
            strip_ansi(&colored),
            plain,
            "stripping ANSI yields plain marker"
        );
    }

    #[test]
    fn test_continuation_gutter() {
        let plain = continuation_gutter(false);
        assert_eq!(plain, "     |     |", "plain continuation gutter");
        let colored = continuation_gutter(true);
        let stripped = strip_ansi(&colored);
        assert_eq!(
            stripped, "     │     │",
            "colored gutter stripped matches expected layout"
        );
    }
}
