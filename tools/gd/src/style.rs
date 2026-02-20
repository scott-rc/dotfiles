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

/// File header foreground (bold blue)
pub const FG_FILE_HEADER: &str = "\x1b[1;38;2;121;192;255m";
/// Hunk header foreground (dim cyan)
pub const FG_HUNK_HEADER: &str = "\x1b[2;38;2;121;192;255m";
/// Line number gutter (dim gray)
pub const FG_GUTTER: &str = "\x1b[2;38;2;110;118;129m";
/// Separator character (dim gray)
pub const FG_SEP: &str = "\x1b[2;38;2;68;76;86m";
/// Added line marker foreground
pub const FG_ADDED_MARKER: &str = "\x1b[38;2;63;185;80m";
/// Deleted line marker foreground
pub const FG_DELETED_MARKER: &str = "\x1b[38;2;248;81;73m";

pub const RESET: &str = "\x1b[0m";
pub const DIM: &str = "\x1b[2m";
pub const NO_DIM: &str = "\x1b[22m";
pub const REVERSE: &str = "\x1b[7m";
pub const NO_REVERSE: &str = "\x1b[27m";

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

/// Format a hunk header line.
pub fn hunk_header(text: &str) -> String {
    format!("{FG_HUNK_HEADER}{text}{RESET}")
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
