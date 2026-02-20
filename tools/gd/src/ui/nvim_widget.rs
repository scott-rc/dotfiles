use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;
use unicode_width::UnicodeWidthStr;

use crate::nvim::bridge::{CursorShape, ModeInfo};
use crate::nvim::grid::GridBuffer;

pub struct NvimWidget<'a> {
    pub grid: &'a GridBuffer,
    pub default_fg: Color,
    pub default_bg: Color,
    pub cursor_visible: bool,
    pub mode_info: Option<&'a ModeInfo>,
}

impl Widget for NvimWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows = area.height.min(self.grid.height);
        let cols = area.width.min(self.grid.width);

        for row in 0..rows {
            let mut col = 0u16;
            while col < cols {
                let grid_row = row as usize;
                let grid_col = col as usize;

                if grid_row >= self.grid.cells.len() || grid_col >= self.grid.cells[grid_row].len() {
                    col += 1;
                    continue;
                }

                let cell = &self.grid.cells[grid_row][grid_col];
                let hl = self.grid.get_hl(cell.hl_id);

                let (fg, bg) = if hl.reverse {
                    (
                        hl.bg.unwrap_or(self.default_bg),
                        hl.fg.unwrap_or(self.default_fg),
                    )
                } else {
                    (
                        hl.fg.unwrap_or(self.default_fg),
                        hl.bg.unwrap_or(self.default_bg),
                    )
                };

                let mut style = Style::default().fg(fg).bg(bg);
                if hl.bold {
                    style = style.add_modifier(Modifier::BOLD);
                }
                if hl.italic {
                    style = style.add_modifier(Modifier::ITALIC);
                }
                if hl.underline {
                    style = style.add_modifier(Modifier::UNDERLINED);
                }
                if hl.strikethrough {
                    style = style.add_modifier(Modifier::CROSSED_OUT);
                }

                let x = area.x + col;
                let y = area.y + row;
                let w: u16 = cell.text.width() as u16;

                if x < area.x + area.width {
                    buf.set_string(x, y, &cell.text, style);
                }

                col += w.max(1);
            }
        }

        // Draw cursor
        if self.cursor_visible {
            let cr = self.grid.cursor_row;
            let cc = self.grid.cursor_col;
            if cr < rows && cc < cols {
                let x = area.x + cc;
                let y = area.y + cr;
                let buf_cell = &mut buf[(x, y)];

                if let Some(CursorShape::Vertical | CursorShape::Horizontal) =
                    self.mode_info.map(|m| &m.cursor_shape)
                {
                    buf_cell.set_style(buf_cell.style().add_modifier(Modifier::UNDERLINED));
                } else {
                    let s = buf_cell.style();
                    let fg = s.bg.unwrap_or(self.default_bg);
                    let bg = s.fg.unwrap_or(self.default_fg);
                    buf_cell.set_style(Style::default().fg(fg).bg(bg));
                }
            }
        }
    }
}
