mod app;
mod event;
mod git;
mod nvim;
mod theme;
mod ui;

use std::io::{self, stdout};

use clap::Parser;
use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{execute, event as crossterm_event};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use event::{AppCmd, AppEvent};
use nvim::bridge::{CursorShape, ModeInfo, RedrawEvent};
use nvim::grid::GridBuffer;

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
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let staged = cli.staged || cli.cached;

    // Find repo root
    let cwd = std::env::current_dir().unwrap_or_else(|e| {
        eprintln!("gd: {e}");
        std::process::exit(1);
    });
    let repo = git::repo_root(&cwd).await.unwrap_or_else(|| {
        eprintln!("gd: not a git repository");
        std::process::exit(1);
    });

    // Resolve diff source and run git diff
    let source = app::resolve_source(staged, &cli.source);
    let diff_args = source.diff_args();
    let str_args: Vec<&str> = diff_args.iter().map(String::as_str).collect();
    let raw = git::run(&repo, &str_args).await.unwrap_or_default();

    let files = git::diff::parse(&raw);
    if files.is_empty() {
        // No changes — exit cleanly like git diff
        return;
    }

    let mut file_list = app::FileList::new(files);

    // Setup terminal
    if terminal::enable_raw_mode().is_err() {
        eprintln!("gd: not a terminal");
        std::process::exit(1);
    }
    execute!(stdout(), EnterAlternateScreen).expect("failed to enter alt screen");
    execute!(
        stdout(),
        crossterm_event::PushKeyboardEnhancementFlags(
            crossterm_event::KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
        )
    ).ok();

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend).expect("failed to create terminal");
    let size = terminal.size().expect("failed to get terminal size");

    // Spawn nvim
    let mut session = nvim::spawn(size.width, size.height).await;
    let mut grid = GridBuffer::new(size.width, size.height);
    let mut mode_info: Vec<ModeInfo> = vec![ModeInfo {
        cursor_shape: CursorShape::Block,
        cell_percentage: 0,
    }];
    let mut current_mode: usize = 0;

    // Setup diff display
    theme::setup(&session.nvim).await;
    nvim::setup_statuscolumn(&session.nvim).await;
    nvim::load_diff(&session.nvim, file_list.current(), &source, &repo).await;

    // Main event loop
    loop {
        let Some(ev) = event::next(&mut session.redraw_rx, &mut session.cmd_rx).await else {
            break;
        };

        match ev {
            AppEvent::Cmd(cmd) => match cmd {
                AppCmd::Quit => break,
                AppCmd::NextFile => {
                    if file_list.next() {
                        nvim::load_diff(&session.nvim, file_list.current(), &source, &repo).await;
                    }
                }
                AppCmd::PrevFile => {
                    if file_list.prev() {
                        nvim::load_diff(&session.nvim, file_list.current(), &source, &repo).await;
                    }
                }
            },
            AppEvent::Key(key) => {
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    break;
                }

                if let Some(input) = nvim::input::to_nvim_key(&key) {
                    session.nvim.input(&input).await.ok();
                }
            }
            AppEvent::Resize(w, h) => {
                grid.resize(w, h);
                nvim::resize(&session.nvim, w, h).await;
            }
            AppEvent::Redraw(events) => {
                let mut needs_render = false;
                for ev in events {
                    match ev {
                        RedrawEvent::GridResize { width, height } => {
                            grid.resize(width, height);
                        }
                        RedrawEvent::GridClear => grid.clear(),
                        RedrawEvent::GridCursorGoto { row, col } => {
                            grid.cursor_goto(row, col);
                        }
                        RedrawEvent::GridLine { row, col_start, cells } => {
                            grid.put_line(row, col_start, &cells);
                        }
                        RedrawEvent::GridScroll { top, bot, left, right, rows } => {
                            grid.scroll(top, bot, left, right, rows);
                        }
                        RedrawEvent::HlAttrDefine { id, attr } => {
                            grid.set_hl_attr(id, attr);
                        }
                        RedrawEvent::DefaultColorsSet { fg, bg } => {
                            session.default_fg = nvim::grid::nvim_color(fg);
                            session.default_bg = nvim::grid::nvim_color(bg);
                        }
                        RedrawEvent::ModeInfoSet { cursor_style } => {
                            mode_info = cursor_style;
                        }
                        RedrawEvent::ModeChange { mode_idx } => {
                            current_mode = mode_idx as usize;
                        }
                        RedrawEvent::Flush => needs_render = true,
                    }
                }

                if needs_render {
                    let default_fg = session.default_fg;
                    let default_bg = session.default_bg;
                    let mi = mode_info.get(current_mode);
                    terminal.draw(|f| {
                        let area = ui::layout::single_pane(f.area());
                        let widget = ui::nvim_widget::NvimWidget {
                            grid: &grid,
                            default_fg,
                            default_bg,
                            cursor_visible: true,
                            mode_info: mi,
                        };
                        f.render_widget(widget, area);
                    }).ok();
                }
            }
        }
    }

    // Teardown
    nvim::shutdown(&session.nvim).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    execute!(
        stdout(),
        crossterm_event::PopKeyboardEnhancementFlags
    ).ok();
    execute!(io::stdout(), LeaveAlternateScreen).ok();
    terminal::disable_raw_mode().ok();
}
