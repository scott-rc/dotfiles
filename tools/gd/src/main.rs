mod ansi;
mod git;
mod pager;
mod render;
mod style;

use std::io::{self, IsTerminal, Write};

use clap::Parser;

use crate::git::{DiffSource, resolve_source};

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
}

fn main() {
    let cli = Cli::parse();
    let staged = cli.staged || cli.cached;

    let cwd = std::env::current_dir().unwrap_or_else(|e| {
        eprintln!("gd: {e}");
        std::process::exit(1);
    });
    let repo = git::repo_root(&cwd).unwrap_or_else(|| {
        eprintln!("gd: not a git repository");
        std::process::exit(1);
    });

    let source = if cli.base {
        let base = git::find_base_branch(&repo);
        let merge_base = git::run(&repo, &["merge-base", &base, "HEAD"])
            .map_or_else(|| base.clone(), |s| s.trim().to_string());
        DiffSource::Range(merge_base, "HEAD".into())
    } else {
        resolve_source(staged, &cli.source)
    };
    let diff_args = source.diff_args();
    let str_args: Vec<&str> = diff_args.iter().map(String::as_str).collect();
    let raw = git::run_diff(&repo, &str_args);

    let mut files = git::diff::parse(&raw);
    git::append_untracked(&repo, &source, cli.no_untracked, &mut files);

    if files.is_empty() {
        return;
    }

    let is_tty = io::stdout().is_terminal();
    let color = !cli.no_color && is_tty;
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));

    let output = render::render(&files, cols as usize, color, false);

    // Use pager if: tty, not --no-pager, content exceeds terminal height
    if is_tty && !cli.no_pager && output.lines.len() > rows as usize {
        let diff_ctx = pager::DiffContext {
            repo: repo.clone(),
            source: source.clone(),
            no_untracked: cli.no_untracked,
        };
        pager::run_pager(output, files, color, &diff_ctx);
    } else {
        let mut stdout = io::BufWriter::new(io::stdout().lock());
        for line in &output.lines {
            let _ = writeln!(stdout, "{line}");
        }
    }
}
