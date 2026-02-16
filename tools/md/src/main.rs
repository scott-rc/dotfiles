use std::io::{self, IsTerminal, Read};
use std::path::Path;

use clap::Parser;

use md::browse::{browse_directory, should_page};
use md::pager::run_pager;
use md::render::render_markdown;
use md::style::Style;

const DEFAULT_MAX_WIDTH: usize = 100;

#[derive(Parser)]
#[command(name = "md", about = "Terminal markdown renderer")]
struct Args {
    /// File path, directory path, or "-" for stdin
    file: Option<String>,

    /// Render width
    #[arg(short, long)]
    width: Option<usize>,

    /// Disable ANSI color output
    #[arg(long)]
    no_color: bool,

    /// Disable the built-in pager
    #[arg(long)]
    no_pager: bool,
}

fn render_width(args: &Args) -> (usize, Option<usize>) {
    let term_width = if io::stdout().is_terminal() {
        crossterm::terminal::size()
            .ok()
            .map(|(cols, _)| cols as usize)
    } else {
        None
    };

    let width = if let Some(w) = args.width {
        w
    } else if let Some(tw) = term_width {
        tw.min(DEFAULT_MAX_WIDTH)
    } else {
        80
    };

    (width, term_width)
}

fn render_centered(input: &str, style: &Style, args: &Args) -> String {
    let (width, term_width) = render_width(args);
    let rendered = render_markdown(input, width, style);
    let margin = match term_width {
        Some(tw) => " ".repeat(tw.saturating_sub(width) / 2),
        None => String::new(),
    };
    if margin.is_empty() {
        rendered
    } else {
        rendered
            .lines()
            .map(|l| format!("{margin}{l}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn main() {
    let args = Args::parse();

    let color = !args.no_color && io::stdout().is_terminal();
    let style = Style::new(color);
    let is_tty = io::stdout().is_terminal();

    // No file and stdin is a TTY → print usage
    if args.file.is_none() && io::stdin().is_terminal() {
        eprintln!("Usage: md [OPTIONS] [FILE]");
        eprintln!();
        eprintln!("  Render markdown in the terminal.");
        eprintln!();
        eprintln!("  FILE can be a file path, directory, or \"-\" for stdin.");
        eprintln!("  Pipe markdown via stdin: cat README.md | md");
        std::process::exit(1);
    }

    // Check if file is a directory
    if let Some(ref file) = args.file
        && file != "-"
    {
        let path = Path::new(file);
        if path.is_dir() {
            let no_pager = args.no_pager;
            let dir = file.clone();
            browse_directory(
                &dir,
                |selection| {
                    let input = match std::fs::read_to_string(selection) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("Error reading {selection}: {e}");
                            return;
                        }
                    };
                    let real_path = std::fs::canonicalize(selection).map_or_else(
                        |_| selection.to_string(),
                        |p| p.to_string_lossy().to_string(),
                    );

                    let centered = render_centered(&input, &style, &args);
                    let content_lines = centered.lines().count();
                    let terminal_rows = crossterm::terminal::size()
                        .map(|(_, r)| r as usize)
                        .unwrap_or(24);

                    if should_page(no_pager, is_tty, content_lines, terminal_rows, true) {
                        let raw = input.clone();
                        let mut on_resize = || render_centered(&input, &style, &args);
                        run_pager(
                            &centered,
                            Some(&real_path),
                            Some(&raw),
                            Some(&mut on_resize),
                        );
                    } else {
                        println!("{centered}");
                    }
                },
                None,
                None,
            );
            return;
        }
    }

    // Read input
    let (input, file_path) = if args.file.as_deref() == Some("-")
        || (args.file.is_none() && !io::stdin().is_terminal())
    {
        // Read from stdin
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
            eprintln!("Error reading stdin: {e}");
            std::process::exit(1);
        });
        (buf, None)
    } else {
        let file = args.file.as_ref().unwrap();
        let content = std::fs::read_to_string(file).unwrap_or_else(|e| {
            eprintln!("Error reading {file}: {e}");
            std::process::exit(1);
        });
        let real_path = std::fs::canonicalize(file)
            .map_or_else(|_| file.clone(), |p| p.to_string_lossy().to_string());
        (content, Some(real_path))
    };

    let centered = render_centered(&input, &style, &args);
    let content_lines = centered.lines().count();
    let terminal_rows = crossterm::terminal::size()
        .map(|(_, r)| r as usize)
        .unwrap_or(24);

    if should_page(args.no_pager, is_tty, content_lines, terminal_rows, false)
        && file_path.is_some()
    {
        let fp = file_path.as_deref().unwrap();
        let raw = input.clone();
        let mut on_resize = || render_centered(&input, &style, &args);
        run_pager(&centered, Some(fp), Some(&raw), Some(&mut on_resize));
    } else {
        print!("{centered}");
    }
}
