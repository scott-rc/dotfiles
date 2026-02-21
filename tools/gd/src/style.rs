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

/// File header foreground (bold blue)
pub const FG_FILE_HEADER: &str = "\x1b[1;38;2;121;192;255m";
/// Hunk header foreground (dim cyan)
pub const FG_HUNK_HEADER: &str = "\x1b[2;38;2;121;192;255m";
/// Line number gutter (dim gray)
pub const FG_GUTTER: &str = "\x1b[2;38;2;110;118;129m";
/// Separator character (dim gray)
pub const FG_SEP: &str = "\x1b[2;38;2;68;76;86m";
/// File tree text (muted gray, no DIM to avoid bleed)
pub const FG_TREE: &str = "\x1b[38;2;139;148;158m";
/// File tree directory names (folder yellow)
pub const FG_TREE_DIR: &str = "\x1b[38;2;224;177;77m";
/// File tree connector lines (gray, matches lsd tree-edge #484f58)
pub const FG_TREE_GUIDE: &str = "\x1b[38;2;72;79;88m";
/// Added line marker foreground
pub const FG_ADDED_MARKER: &str = "\x1b[38;2;63;185;80m";
/// Deleted line marker foreground
pub const FG_DELETED_MARKER: &str = "\x1b[38;2;248;81;73m";

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
pub const REVERSE: &str = "\x1b[7m";
pub const NO_REVERSE: &str = "\x1b[27m";
pub const UNDERLINE: &str = "\x1b[4m";
pub const NO_UNDERLINE: &str = "\x1b[24m";

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
        "rs" => ("\u{e7a8}", "\x1b[38;2;222;135;53m"),          // Rust orange
        "ts" | "tsx" => ("\u{e628}", "\x1b[38;2;49;120;198m"),   // TypeScript blue
        "js" | "jsx" => ("\u{e781}", "\x1b[38;2;241;224;90m"),   // JavaScript yellow
        "go" => ("\u{e627}", "\x1b[38;2;0;173;216m"),            // Go cyan
        "py" => ("\u{e73c}", "\x1b[38;2;55;118;171m"),           // Python blue
        "md" => ("\u{e73e}", "\x1b[38;2;81;154;186m"),           // Markdown teal
        "json" => ("\u{e60b}", "\x1b[38;2;241;224;90m"),         // JSON yellow
        "toml" => ("\u{e6b2}", "\x1b[38;2;139;148;158m"),        // TOML gray
        "yaml" | "yml" => ("\u{e6a8}", "\x1b[38;2;203;75;83m"),  // YAML red
        "sh" | "fish" | "bash" | "zsh" => ("\u{e795}", "\x1b[38;2;137;224;81m"), // Shell green
        "lua" => ("\u{e620}", "\x1b[38;2;81;160;207m"),          // Lua blue
        "html" => ("\u{e736}", "\x1b[38;2;228;77;38m"),          // HTML orange
        "css" => ("\u{e749}", "\x1b[38;2;86;156;214m"),          // CSS blue
        "lock" => ("\u{f023}", "\x1b[38;2;139;148;158m"),        // Lock gray
        _ => ("\u{f15b}", FG_TREE),                              // Generic file
    }
}

/// Return (icon, ansi_color) for a directory entry.
pub fn dir_icon() -> (&'static str, &'static str) {
    ("\u{f413}", FG_TREE_DIR)
}
