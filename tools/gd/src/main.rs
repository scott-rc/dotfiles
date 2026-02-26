mod ansi;
mod git;
mod pager;
mod render;
mod style;

use std::io::{self, IsTerminal, Write};

use clap::Parser;

use tui::highlight::{SYNTAX_SET, THEME};

use crate::git::{DiffSource, resolve_commit_parent, resolve_source};

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
    #[arg(long)]
    no_pager: bool,

    /// Disable ANSI colors
    #[arg(long)]
    no_color: bool,

    /// Hide untracked files (only applies to working tree mode)
    #[arg(long)]
    no_untracked: bool,

    /// Diff against auto-detected base branch
    #[arg(long, short = 'b')]
    base: bool,

    /// Show whitespace-only changes (hidden by default)
    #[arg(long, short = 'w')]
    show_whitespace: bool,
}

fn main() {
    let cli = Cli::parse();
    let staged = cli.staged || cli.cached;

    // Eagerly initialize syntax highlighting statics in the background,
    // overlapping with git command execution.
    let syntax_init = std::thread::spawn(|| {
        let _ = &*SYNTAX_SET;
        let _ = &*THEME;
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
        (repo, DiffSource::Range(format!("{base}...HEAD"), String::new()))
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
    let raw = git::run_diff(&repo, &str_args);

    let mut files = git::diff::parse(&raw);
    git::append_untracked(&repo, &source, cli.no_untracked, &mut files);
    git::sort_files_for_display(&mut files);

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

    syntax_init.join().unwrap();

    let is_tty = io::stdout().is_terminal();
    let color = !cli.no_color && is_tty;
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));

    let output = render::render(&files, cols as usize, color);

    // Use pager if: tty, not --no-pager, content exceeds terminal height
    if is_tty && !cli.no_pager && output.lines.len() > rows as usize {
        let diff_ctx = pager::DiffContext {
            repo: repo.clone(),
            source: source.clone(),
            no_untracked: cli.no_untracked,
            ignore_whitespace: !cli.show_whitespace,
        };
        pager::run_pager(output, files, color, &diff_ctx);
    } else {
        let mut stdout = io::BufWriter::new(io::stdout().lock());
        for line in &output.lines {
            let _ = writeln!(stdout, "{line}");
        }
    }
}
