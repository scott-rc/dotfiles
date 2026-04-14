mod ansi;
mod debug;
mod git;
mod pager;
mod render;
mod style;
#[cfg(feature = "web")]
mod web;

use std::io::{self, IsTerminal, Write};

use clap::{ArgAction, Parser, ValueEnum};

use tui::highlight::{SYNTAX_SET, THEME_DARK, THEME_LIGHT, ThemeVariant};

use crate::git::{DiffSource, resolve_commit_parent, resolve_source};

/// CLI theme argument (maps to `ThemeVariant` after resolving "system").
#[derive(ValueEnum, Clone, Copy, Default)]
enum ThemeArg {
    Light,
    Dark,
    #[default]
    System,
}

impl ThemeArg {
    /// Resolve to a concrete `ThemeVariant` based on system preference.
    fn resolve(self) -> ThemeVariant {
        match self {
            Self::Light => ThemeVariant::Light,
            Self::Dark => ThemeVariant::Dark,
            Self::System => {
                // Check if terminal supports dark mode detection.
                // Default to dark for most terminals.
                ThemeVariant::Dark
            }
        }
    }
}

#[derive(Parser)]
#[command(name = "gd", about = "Terminal git diff viewer")]
struct Cli {
    /// Commit, range (a..b), or ref to diff
    #[arg(value_name = "REF")]
    source: Vec<String>,

    /// Show staged changes
    #[arg(long, short = 's')]
    staged: bool,

    /// Alias for --staged
    #[arg(long, hide = true)]
    cached: bool,

    /// Print to stdout without pager
    #[arg(long = "no-pager", action = ArgAction::SetFalse)]
    pager: bool,

    /// Disable ANSI colors
    #[arg(long = "no-color", action = ArgAction::SetFalse)]
    color: bool,

    /// Hide untracked files (only applies to working tree mode)
    #[arg(long = "no-untracked", action = ArgAction::SetFalse)]
    untracked: bool,

    /// Skip opening browser automatically (just print URL)
    #[arg(long = "no-open", action = ArgAction::SetFalse)]
    open: bool,

    /// Diff against auto-detected base branch
    #[arg(long, short = 'b')]
    base: bool,

    /// Show whitespace-only changes (hidden by default)
    #[arg(long, short = 'w')]
    show_whitespace: bool,

    /// Start a local web server with an interactive browser-based diff viewer
    #[arg(long)]
    web: bool,

    /// Replay keystrokes through the pager pipeline without a TTY (for benchmarking).
    /// Plain chars map to keys; use <Enter>, <Esc>, <C-c>, etc. for special keys.
    #[arg(long, value_name = "KEYS")]
    replay: Option<String>,

    /// Terminal columns for --replay mode
    #[arg(long, default_value = "120", requires = "replay")]
    cols: u16,

    /// Terminal rows for --replay mode
    #[arg(long, default_value = "50", requires = "replay")]
    rows: u16,

    /// Shutdown grace period in milliseconds after last browser tab closes
    #[arg(long, default_value = "1000", requires = "web")]
    shutdown_grace_ms: u64,

    /// Color theme for syntax highlighting (TUI mode only; web mode uses CSS)
    #[arg(long, value_enum, default_value_t = ThemeArg::System)]
    theme: ThemeArg,
}

fn main() {
    let t0 = std::time::Instant::now();
    let cli = Cli::parse();
    let staged = cli.staged || cli.cached;

    // Eagerly initialize syntax highlighting statics in the background,
    // overlapping with git command execution.
    let theme_variant = cli.theme.resolve();
    let syntax_init = std::thread::spawn(move || {
        let _ = &*SYNTAX_SET;
        // Initialize the appropriate theme based on CLI selection
        match theme_variant {
            ThemeVariant::Dark => { let _ = &*THEME_DARK; }
            ThemeVariant::Light => { let _ = &*THEME_LIGHT; }
        }
    });

    let cwd = std::env::current_dir().unwrap_or_else(|e| {
        eprintln!("gd: {e}");
        std::process::exit(1);
    });

    // When using -b, overlap repo_root with base branch detection phase 1
    // (both are independent git commands that work from any directory).
    let (repo, source) = if cli.base {
        let (repo_opt, init) = std::thread::scope(|s| {
            let repo_h = s.spawn(|| git::repo_root(&cwd));
            let init_h = s.spawn(|| git::base_branch_init(&cwd));
            (repo_h.join().unwrap(), init_h.join().unwrap())
        });
        let repo = repo_opt.unwrap_or_else(|| {
            eprintln!("gd: not a git repository");
            std::process::exit(1);
        });
        let (current, default) = init;
        let base = git::base_branch_finish(&repo, &current, &default);
        // Use triple-dot merge-base syntax to avoid a separate `git merge-base` call
        (
            repo,
            DiffSource::Range(format!("{base}...HEAD"), String::new()),
        )
    } else {
        let repo = git::repo_root(&cwd).unwrap_or_else(|| {
            eprintln!("gd: not a git repository");
            std::process::exit(1);
        });
        let source = resolve_source(staged, &cli.source);
        (repo, source)
    };
    let source = match source {
        DiffSource::Commit(ref_str) => {
            let parent = resolve_commit_parent(&repo, &ref_str);
            DiffSource::Range(parent, ref_str)
        }
        other => other,
    };
    let mut diff_args = source.diff_args();
    if !cli.show_whitespace {
        diff_args.push("-w".into());
    }
    let str_args: Vec<&str> = diff_args.iter().map(String::as_str).collect();
    debug::trace("main", "pre-diff", t0);
    let raw = git::run_diff(&repo, &str_args);
    debug::trace("main", "post-diff", t0);

    let mut files = git::diff::parse(&raw);
    git::append_untracked(&repo, &source, !cli.untracked, &mut files);
    git::sort_files_for_display(&mut files);
    debug::trace("main", &format!("post-parse ({} files)", files.len()), t0);

    if files.is_empty() {
        if !cli.show_whitespace {
            let base_args = source.diff_args();
            let base_str: Vec<&str> = base_args.iter().map(String::as_str).collect();
            let raw_with_ws = git::run_diff(&repo, &base_str);
            if !raw_with_ws.is_empty() {
                eprintln!("gd: only whitespace changes found (use -w to show)");
            }
        }
        return;
    }

    // Web mode: serve diff in a browser
    if cli.web {
        #[cfg(feature = "web")]
        {
            syntax_init.join().unwrap();
            let diff_ctx = pager::DiffContext {
                repo: repo.clone(),
                source: source.clone(),
                no_untracked: !cli.untracked,
                ignore_whitespace: !cli.show_whitespace,
            };
            web::run_web_server(files, &diff_ctx, cli.open, cli.shutdown_grace_ms);
            return;
        }
        #[cfg(not(feature = "web"))]
        {
            eprintln!("gd: --web requires the 'web' feature (cargo build --features web)");
            std::process::exit(1);
        }
    }

    syntax_init.join().unwrap();
    debug::trace("main", "post-syntax-join", t0);

    let diff_ctx = pager::DiffContext {
        repo: repo.clone(),
        source: source.clone(),
        no_untracked: !cli.untracked,
        ignore_whitespace: !cli.show_whitespace,
    };

    // Decide full-context upfront so pager paths don't re-run git diff.
    let total_hunks: usize = files.iter().map(|f| f.hunks.len()).sum();
    let use_full_context = pager::default_full_context(files.len(), total_hunks);

    // Replay mode: drive the pager without a TTY
    if let Some(ref keys) = cli.replay {
        let color = true; // force color for realistic benchmarking
        let (cols, rows) = (cli.cols, cli.rows);
        let use_pager = true;
        let files = maybe_regenerate(files, use_pager, use_full_context, &diff_ctx);
        debug::trace("main", "post-full-context", t0);
        pager::run_pager_replay(files, color, use_full_context, &diff_ctx, keys, cols, rows);
        return;
    }

    let is_tty = io::stdout().is_terminal();
    let color = cli.color && is_tty;
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));

    // Estimate rendered line count cheaply: file headers + hunk separators + diff lines.
    // Avoids a full render() just to decide whether to use the pager.
    let estimated_lines: usize = files
        .iter()
        .map(|f| {
            1 + f.hunks.len().saturating_sub(1)
                + f.hunks.iter().map(|h| h.lines.len()).sum::<usize>()
        })
        .sum();
    let use_pager = is_tty && cli.pager && estimated_lines > rows as usize;

    if use_pager {
        let files = maybe_regenerate(files, true, use_full_context, &diff_ctx);
        debug::trace("main", "post-full-context", t0);
        pager::run_pager(files, color, use_full_context, &diff_ctx);
    } else {
        // No-pager path: render and print
        let output = render::render(&files, cols as usize, color);
        let mut stdout = io::BufWriter::with_capacity(256 * 1024, io::stdout().lock());
        for line in output.lines() {
            let _ = writeln!(stdout, "{line}");
        }
    }
}

/// Re-run git diff with full context (`-U999999`) when the pager will use it.
/// Returns files unchanged if full context isn't needed.
fn maybe_regenerate(
    files: Vec<git::diff::DiffFile>,
    use_pager: bool,
    use_full_context: bool,
    diff_ctx: &pager::DiffContext,
) -> Vec<git::diff::DiffFile> {
    if !use_pager || !use_full_context {
        return files;
    }
    let mut args = diff_ctx.source.diff_args_full_context();
    if diff_ctx.ignore_whitespace {
        args.push("-w".into());
    }
    let str_args: Vec<&str> = args.iter().map(String::as_str).collect();
    let raw = git::run_diff(&diff_ctx.repo, &str_args);
    let mut files = git::diff::parse(&raw);
    git::append_untracked(
        &diff_ctx.repo,
        &diff_ctx.source,
        diff_ctx.no_untracked,
        &mut files,
    );
    git::sort_files_for_display(&mut files);
    files
}
