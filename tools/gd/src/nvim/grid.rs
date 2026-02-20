use std::collections::HashMap;
use std::sync::LazyLock;

use ratatui::style::Color;

#[derive(Clone, Debug)]
pub struct Cell {
    pub text: String,
    pub hl_id: u64,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            text: " ".into(),
            hl_id: 0,
        }
    }
}

#[derive(Clone, Debug, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct HlAttr {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub reverse: bool,
    pub sp: Option<Color>,
}

static DEFAULT_HL: LazyLock<HlAttr> = LazyLock::new(HlAttr::default);

pub struct GridBuffer {
    pub cells: Vec<Vec<Cell>>,
    pub width: u16,
    pub height: u16,
    pub cursor_row: u16,
    pub cursor_col: u16,
    pub hl_table: HashMap<u64, HlAttr>,
}

impl GridBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        let cells = vec![vec![Cell::default(); width as usize]; height as usize];
        Self {
            cells,
            width,
            height,
            cursor_row: 0,
            cursor_col: 0,
            hl_table: HashMap::new(),
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        let mut cells = vec![vec![Cell::default(); width as usize]; height as usize];
        let copy_h = self.height.min(height) as usize;
        let copy_w = self.width.min(width) as usize;
        for (r, new_row) in cells.iter_mut().enumerate().take(copy_h) {
            for (c, cell) in new_row.iter_mut().enumerate().take(copy_w) {
                cell.clone_from(&self.cells[r][c]);
            }
        }
        self.cells = cells;
        self.width = width;
        self.height = height;
    }

    pub fn clear(&mut self) {
        for row in &mut self.cells {
            for cell in row.iter_mut() {
                *cell = Cell::default();
            }
        }
    }

    pub fn cursor_goto(&mut self, row: u16, col: u16) {
        self.cursor_row = row;
        self.cursor_col = col;
    }

    pub fn put_line(&mut self, row: u16, col_start: u16, cells: &[(String, Option<u64>, u16)]) {
        if row as usize >= self.cells.len() {
            return;
        }
        let grid_row = &mut self.cells[row as usize];
        let mut col = col_start as usize;
        let mut last_hl: u64 = 0;

        for (text, hl_id, repeat) in cells {
            let hl = match hl_id {
                Some(id) => {
                    last_hl = *id;
                    *id
                }
                None => last_hl,
            };
            let count = (*repeat).max(1) as usize;
            for _ in 0..count {
                if col >= grid_row.len() {
                    break;
                }
                grid_row[col] = Cell {
                    text: text.clone(),
                    hl_id: hl,
                };
                col += 1;
            }
        }
    }

    pub fn scroll(&mut self, top: u16, bot: u16, left: u16, right: u16, rows: i64) {
        if rows == 0 {
            return;
        }
        let t = top as usize;
        let b = bot as usize;
        let l = left as usize;
        let r = right as usize;

        if rows > 0 {
            let shift = rows as usize;
            for dst in t..b {
                let src = dst + shift;
                if src < b {
                    for c in l..r {
                        self.cells[dst][c] = self.cells[src][c].clone();
                    }
                } else {
                    for c in l..r {
                        self.cells[dst][c] = Cell::default();
                    }
                }
            }
        } else {
            let shift = (-rows) as usize;
            for dst in (t..b).rev() {
                if dst >= t + shift {
                    let src = dst - shift;
                    for c in l..r {
                        self.cells[dst][c] = self.cells[src][c].clone();
                    }
                } else {
                    for c in l..r {
                        self.cells[dst][c] = Cell::default();
                    }
                }
            }
        }
    }

    pub fn set_hl_attr(&mut self, id: u64, attr: HlAttr) {
        self.hl_table.insert(id, attr);
    }

    pub fn get_hl(&self, hl_id: u64) -> &HlAttr {
        self.hl_table.get(&hl_id).unwrap_or(&DEFAULT_HL)
    }
}

pub fn nvim_color(val: i64) -> Color {
    let r = ((val >> 16) & 0xff) as u8;
    let g = ((val >> 8) & 0xff) as u8;
    let b = (val & 0xff) as u8;
    Color::Rgb(r, g, b)
}
