use nvim_rs::Handler;
use rmpv::Value;
use tokio::sync::mpsc;
use tokio_util::compat::Compat;

use super::grid::{nvim_color, HlAttr};
use crate::event::AppCmd;

pub type Writer = Compat<tokio::process::ChildStdin>;

#[derive(Clone, Debug)]
pub enum RedrawEvent {
    GridResize {
        width: u16,
        height: u16,
    },
    GridClear,
    GridCursorGoto {
        row: u16,
        col: u16,
    },
    GridLine {
        row: u16,
        col_start: u16,
        cells: Vec<(String, Option<u64>, u16)>,
    },
    GridScroll {
        top: u16,
        bot: u16,
        left: u16,
        right: u16,
        rows: i64,
    },
    HlAttrDefine {
        id: u64,
        attr: HlAttr,
    },
    DefaultColorsSet {
        fg: i64,
        bg: i64,
    },
    ModeInfoSet {
        cursor_style: Vec<ModeInfo>,
    },
    ModeChange {
        mode_idx: u64,
    },
    Flush,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct ModeInfo {
    pub cursor_shape: CursorShape,
    pub cell_percentage: u16,
}

#[derive(Clone, Debug, Default)]
pub enum CursorShape {
    #[default]
    Block,
    Horizontal,
    Vertical,
}

#[derive(Clone)]
pub struct NvimHandler {
    tx: mpsc::UnboundedSender<Vec<RedrawEvent>>,
    cmd_tx: mpsc::UnboundedSender<AppCmd>,
}

impl NvimHandler {
    pub fn new(
        tx: mpsc::UnboundedSender<Vec<RedrawEvent>>,
        cmd_tx: mpsc::UnboundedSender<AppCmd>,
    ) -> Self {
        Self { tx, cmd_tx }
    }
}

#[async_trait::async_trait]
impl Handler for NvimHandler {
    type Writer = Writer;

    async fn handle_notify(&self, name: String, args: Vec<Value>, _nvim: nvim_rs::Neovim<Self::Writer>) {
        if name == "gd_cmd" {
            if let Some(cmd) = args.first().and_then(Value::as_str) {
                let app_cmd = match cmd {
                    "next_file" => Some(AppCmd::NextFile),
                    "prev_file" => Some(AppCmd::PrevFile),
                    "quit" => Some(AppCmd::Quit),
                    _ => None,
                };
                if let Some(c) = app_cmd {
                    let _ = self.cmd_tx.send(c);
                }
            }
            return;
        }

        if name != "redraw" {
            return;
        }

        let mut events = Vec::new();
        for arg in &args {
            let Some(arr) = arg.as_array() else { continue };
            let Some(name) = arr.first().and_then(Value::as_str) else { continue };

            for params in &arr[1..] {
                let Some(params) = params.as_array() else { continue };
                if let Some(ev) = parse_event(name, params) {
                    events.push(ev);
                }
            }
        }

        if !events.is_empty() {
            let _ = self.tx.send(events);
        }
    }
}

fn parse_event(name: &str, params: &[Value]) -> Option<RedrawEvent> {
    match name {
        "grid_resize" => {
            let grid = params.first()?.as_u64()?;
            if grid != 1 { return None; }
            let w = params.get(1)?.as_u64()? as u16;
            let h = params.get(2)?.as_u64()? as u16;
            Some(RedrawEvent::GridResize { width: w, height: h })
        }
        "grid_clear" => Some(RedrawEvent::GridClear),
        "grid_cursor_goto" => {
            let grid = params.first()?.as_u64()?;
            if grid != 1 { return None; }
            let row = params.get(1)?.as_u64()? as u16;
            let col = params.get(2)?.as_u64()? as u16;
            Some(RedrawEvent::GridCursorGoto { row, col })
        }
        "grid_line" => parse_grid_line(params),
        "grid_scroll" => {
            let grid = params.first()?.as_u64()?;
            if grid != 1 { return None; }
            let top = params.get(1)?.as_u64()? as u16;
            let bot = params.get(2)?.as_u64()? as u16;
            let left = params.get(3)?.as_u64()? as u16;
            let right = params.get(4)?.as_u64()? as u16;
            let rows = params.get(5)?.as_i64()?;
            Some(RedrawEvent::GridScroll { top, bot, left, right, rows })
        }
        "hl_attr_define" => {
            let id = params.first()?.as_u64()?;
            let map = params.get(1)?.as_map()?;
            let attr = parse_hl_attr(map);
            Some(RedrawEvent::HlAttrDefine { id, attr })
        }
        "default_colors_set" => {
            let fg = params.first()?.as_i64()?;
            let bg = params.get(1)?.as_i64()?;
            Some(RedrawEvent::DefaultColorsSet { fg, bg })
        }
        "mode_info_set" => {
            let list = params.get(1)?.as_array()?;
            let styles = list.iter().filter_map(|v| {
                let map = v.as_map()?;
                let mut shape = CursorShape::Block;
                let mut pct = 0u16;
                for (k, v) in map {
                    match k.as_str()? {
                        "cursor_shape" => {
                            shape = match v.as_str()? {
                                "horizontal" => CursorShape::Horizontal,
                                "vertical" => CursorShape::Vertical,
                                _ => CursorShape::Block,
                            };
                        }
                        "cell_percentage" => pct = v.as_u64()? as u16,
                        _ => {}
                    }
                }
                Some(ModeInfo { cursor_shape: shape, cell_percentage: pct })
            }).collect();
            Some(RedrawEvent::ModeInfoSet { cursor_style: styles })
        }
        "mode_change" => {
            let idx = params.get(1)?.as_u64()?;
            Some(RedrawEvent::ModeChange { mode_idx: idx })
        }
        "flush" => Some(RedrawEvent::Flush),
        _ => None,
    }
}

fn parse_grid_line(params: &[Value]) -> Option<RedrawEvent> {
    let grid = params.first()?.as_u64()?;
    if grid != 1 { return None; }
    let row = params.get(1)?.as_u64()? as u16;
    let col_start = params.get(2)?.as_u64()? as u16;
    let cell_data = params.get(3)?.as_array()?;

    let mut cells = Vec::with_capacity(cell_data.len());
    for cell in cell_data {
        let arr = cell.as_array()?;
        let text = arr.first()?.as_str()?.to_string();
        let hl_id = arr.get(1).and_then(Value::as_u64);
        let repeat = arr.get(2).and_then(Value::as_u64).unwrap_or(1) as u16;
        cells.push((text, hl_id, repeat));
    }

    Some(RedrawEvent::GridLine { row, col_start, cells })
}

fn parse_hl_attr(map: &[(Value, Value)]) -> HlAttr {
    let mut attr = HlAttr::default();
    for (k, v) in map {
        let Some(key) = k.as_str() else { continue };
        match key {
            "foreground" => attr.fg = v.as_i64().map(nvim_color),
            "background" => attr.bg = v.as_i64().map(nvim_color),
            "bold" => attr.bold = v.as_bool().unwrap_or(false),
            "italic" => attr.italic = v.as_bool().unwrap_or(false),
            "underline" => attr.underline = v.as_bool().unwrap_or(false),
            "strikethrough" => attr.strikethrough = v.as_bool().unwrap_or(false),
            "reverse" => attr.reverse = v.as_bool().unwrap_or(false),
            "special" => attr.sp = v.as_i64().map(nvim_color),
            _ => {}
        }
    }
    attr
}
