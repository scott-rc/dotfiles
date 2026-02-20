pub mod bridge;
pub mod grid;
pub mod input;

use std::path::Path;
use std::process::Stdio;

use nvim_rs::create::tokio as create;
use nvim_rs::Neovim;
use rmpv::Value;
use similar::{ChangeTag, TextDiff};
use tokio::process::Command;
use tokio::sync::mpsc;

use bridge::{NvimHandler, RedrawEvent, Writer};
use crate::app::DiffSource;
use crate::event::AppCmd;
use crate::git;
use crate::git::diff::{DiffFile, DiffLine, LineKind};

pub struct NvimSession {
    pub nvim: Neovim<Writer>,
    pub redraw_rx: mpsc::UnboundedReceiver<Vec<RedrawEvent>>,
    pub cmd_rx: mpsc::UnboundedReceiver<AppCmd>,
    pub default_fg: ratatui::style::Color,
    pub default_bg: ratatui::style::Color,
}

pub async fn spawn(width: u16, height: u16) -> NvimSession {
    let (tx, rx) = mpsc::unbounded_channel();
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    let handler = NvimHandler::new(tx, cmd_tx);

    let mut cmd = Command::new("nvim");
    cmd.arg("--embed")
        .arg("--headless")
        .arg("-u").arg("NONE")
        .arg("--clean")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    let (nvim, _io_handle, _child) = create::new_child_cmd(&mut cmd, handler)
        .await
        .expect("failed to spawn nvim");

    // Attach UI with ext_linegrid
    let opts = Value::Map(vec![
        (Value::from("rgb"), Value::from(true)),
        (Value::from("ext_linegrid"), Value::from(true)),
    ]);
    let _ = nvim.call("nvim_ui_attach", vec![
        Value::from(u64::from(width)),
        Value::from(u64::from(height)),
        opts,
    ]).await.expect("nvim_ui_attach failed");

    configure_treesitter(&nvim).await;

    let init_lua = r"
        vim.o.number = false
        vim.o.relativenumber = false
        vim.o.signcolumn = 'no'
        vim.o.showmode = false
        vim.o.ruler = false
        vim.o.laststatus = 0
        vim.o.cmdheight = 0
        vim.o.showtabline = 0
        vim.o.scrolloff = 3
    ";
    nvim.call("nvim_exec_lua", vec![Value::from(init_lua), Value::Array(vec![])])
        .await
        .ok();

    NvimSession {
        nvim,
        redraw_rx: rx,
        cmd_rx,
        default_fg: ratatui::style::Color::White,
        default_bg: ratatui::style::Color::Reset,
    }
}

async fn configure_treesitter(nvim: &Neovim<Writer>) {
    let lua = r"
        local paths = {
            vim.fn.stdpath('data') .. '/lazy/nvim-treesitter/parser',
            vim.fn.stdpath('data') .. '/site/pack/packer/start/nvim-treesitter/parser',
            vim.fn.stdpath('data') .. '/plugged/nvim-treesitter/parser',
        }
        for _, p in ipairs(paths) do
            if vim.fn.isdirectory(p) == 1 then
                vim.opt.runtimepath:append(vim.fn.fnamemodify(p, ':h:h'))
                return p
            end
        end
        return ''
    ";
    nvim.call("nvim_exec_lua", vec![Value::from(lua), Value::Array(vec![])])
        .await
        .ok();
}

pub async fn load_file(nvim: &Neovim<Writer>, path: &str) {
    let lua = r"
        local path = ...
        vim.cmd('edit ' .. vim.fn.fnameescape(path))
        vim.cmd('filetype detect')
        pcall(function()
            vim.treesitter.start()
        end)
    ";
    nvim.call("nvim_exec_lua", vec![
        Value::from(lua),
        Value::Array(vec![Value::from(path)]),
    ]).await.ok();
}

pub async fn resize(nvim: &Neovim<Writer>, width: u16, height: u16) {
    nvim.call("nvim_ui_try_resize", vec![
        Value::from(u64::from(width)),
        Value::from(u64::from(height)),
    ]).await.ok();
}

pub async fn shutdown(nvim: &Neovim<Writer>) {
    nvim.call("nvim_exec_lua", vec![
        Value::from("vim.cmd('qa!')"),
        Value::Array(vec![]),
    ]).await.ok();
}

/// Install the statuscolumn function that renders dual line numbers.
pub async fn setup_statuscolumn(nvim: &Neovim<Writer>) {
    let lua = r#"
        function GdStatusColumn()
            local lnum = vim.v.lnum
            local virtnum = vim.v.virtnum

            if virtnum ~= 0 then
                -- Virtual line: show old lineno from virt_map
                local vmap = vim.b.gd_virt_map
                if vmap then
                    local entries = vmap[tostring(lnum)]
                    if entries then
                        local entry = entries[math.abs(virtnum)]
                        if entry and entry.old then
                            return string.format("%%#GdGutterNum#%5d %%#GdGutterSep#│%%#GdGutterNum#      %%#GdGutterSep#│%%*", entry.old)
                        end
                    end
                end
                return string.format("%%#GdGutterNum#      %%#GdGutterSep#│%%#GdGutterNum#      %%#GdGutterSep#│%%*")
            end

            local lmap = vim.b.gd_line_map
            if not lmap then
                return string.format("%%#GdGutterNum#%5d %%#GdGutterSep#│%%#GdGutterNum#%5d %%#GdGutterSep#│%%*", lnum, lnum)
            end

            local entry = lmap[tostring(lnum)]
            if not entry then
                return string.format("%%#GdGutterNum#      %%#GdGutterSep#│%%#GdGutterNum#      %%#GdGutterSep#│%%*")
            end

            local old_str = entry.old and string.format("%5d", entry.old) or "     "
            local new_str = entry.new and string.format("%5d", entry.new) or "     "
            return string.format("%%#GdGutterNum#%s %%#GdGutterSep#│%%#GdGutterNum#%s %%#GdGutterSep#│%%*", old_str, new_str)
        end

        vim.o.statuscolumn = "%!v:lua.GdStatusColumn()"
        vim.o.number = true
        vim.o.numberwidth = 1
        vim.o.signcolumn = "no"
    "#;
    nvim.call("nvim_exec_lua", vec![Value::from(lua), Value::Array(vec![])])
        .await
        .ok();
}

/// Set up buffer-local keymaps for hunk/file navigation and quit.
async fn setup_keymaps(nvim: &Neovim<Writer>) {
    let lua = r"
        local buf = vim.api.nvim_get_current_buf()
        local opts = { buffer = buf, silent = true }

        vim.keymap.set('n', ']c', function()
            local starts = vim.b.gd_hunk_starts or {}
            local cur = vim.fn.line('.')
            for _, s in ipairs(starts) do
                if s > cur then
                    vim.api.nvim_win_set_cursor(0, {s, 0})
                    return
                end
            end
        end, opts)

        vim.keymap.set('n', '[c', function()
            local starts = vim.b.gd_hunk_starts or {}
            local cur = vim.fn.line('.')
            for i = #starts, 1, -1 do
                if starts[i] < cur then
                    vim.api.nvim_win_set_cursor(0, {starts[i], 0})
                    return
                end
            end
        end, opts)

        vim.keymap.set('n', ']f', function()
            vim.rpcnotify(0, 'gd_cmd', 'next_file')
        end, opts)

        vim.keymap.set('n', '[f', function()
            vim.rpcnotify(0, 'gd_cmd', 'prev_file')
        end, opts)

        vim.keymap.set('n', 'q', function()
            vim.rpcnotify(0, 'gd_cmd', 'quit')
        end, opts)
    ";
    nvim.call("nvim_exec_lua", vec![Value::from(lua), Value::Array(vec![])])
        .await
        .ok();
}

/// Load a diff file into the nvim buffer with highlights and virtual lines.
pub async fn load_diff(
    nvim: &Neovim<Writer>,
    diff_file: &DiffFile,
    source: &DiffSource,
    repo: &Path,
) {
    let path = diff_file.path();

    // Load buffer content
    match source {
        DiffSource::WorkingTree => {
            let abs = repo.join(path);
            load_file(nvim, &abs.to_string_lossy()).await;
        }
        _ => {
            load_from_git(nvim, source, path, repo).await;
        }
    }

    let Some(hunk) = diff_file.hunks.first() else {
        return;
    };

    // Build line_map, virt_map, extmarks, and hunk_starts
    let (lua_code, args) = build_diff_lua(hunk);

    nvim.call("nvim_exec_lua", vec![
        Value::from(lua_code),
        Value::Array(args),
    ])
    .await
    .ok();

    setup_keymaps(nvim).await;

    // Make non-WorkingTree buffers readonly
    if !matches!(source, DiffSource::WorkingTree) {
        nvim.call("nvim_exec_lua", vec![
            Value::from("vim.bo.modifiable = false; vim.bo.readonly = true"),
            Value::Array(vec![]),
        ])
        .await
        .ok();
    }
}

/// Load file content from git into a scratch buffer.
async fn load_from_git(nvim: &Neovim<Writer>, source: &DiffSource, path: &str, repo: &Path) {
    let show_args = source.show_args(path);
    let content = if let Some(args) = show_args {
        let str_args: Vec<&str> = args.iter().map(String::as_str).collect();
        git::run(repo, &str_args).await.unwrap_or_default()
    } else {
        String::new()
    };

    // Detect filetype from extension
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let lua = r"
        local content, ext, path = ...
        vim.cmd('enew')
        local buf = vim.api.nvim_get_current_buf()
        local lines = vim.split(content, '\n', { plain = true })
        -- Remove trailing empty line from git show output
        if #lines > 0 and lines[#lines] == '' then
            table.remove(lines)
        end
        vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
        vim.bo[buf].buftype = 'nofile'
        vim.bo[buf].bufhidden = 'wipe'
        vim.bo[buf].swapfile = false
        -- Set filetype for syntax
        if ext ~= '' then
            local ft = vim.filetype.match({ filename = path }) or ext
            vim.bo[buf].filetype = ft
        end
        pcall(function() vim.treesitter.start() end)
    ";
    nvim.call("nvim_exec_lua", vec![
        Value::from(lua),
        Value::Array(vec![
            Value::from(content),
            Value::from(ext),
            Value::from(path),
        ]),
    ])
    .await
    .ok();
}

/// Build the Lua code + args to apply all diff decorations in one call.
fn build_diff_lua(
    hunk: &crate::git::diff::DiffHunk,
) -> (String, Vec<Value>) {
    // Walk the lines, building structures for the Lua side.
    // Buffer lines are only Context + Added lines (deleted are virtual).
    let mut line_map_entries = Vec::new(); // (buf_line_1indexed, old, new, kind)
    let mut buf_line: u32 = 0; // 0-indexed count of real buffer lines
    let mut extmark_calls = Vec::new(); // Lua snippets for extmarks
    let mut hunk_starts = Vec::new(); // 1-indexed buffer lines where change blocks start

    // Collect deleted line blocks for virtual line attachment
    let lines = &hunk.lines;
    let mut i = 0;

    while i < lines.len() {
        let line = &lines[i];
        match line.kind {
            LineKind::Context => {
                buf_line += 1;
                line_map_entries.push(format!(
                    "[\"{buf_line}\"] = {{old = {}, new = {}}}",
                    opt_num(line.old_lineno),
                    opt_num(line.new_lineno),
                ));
                i += 1;
            }
            LineKind::Added => {
                buf_line += 1;
                let is_start = i == 0
                    || lines[i - 1].kind == LineKind::Context;
                if is_start {
                    hunk_starts.push(buf_line);
                }
                line_map_entries.push(format!(
                    "[\"{buf_line}\"] = {{new = {}}}",
                    opt_num(line.new_lineno),
                ));
                extmark_calls.push(format!(
                    "vim.api.nvim_buf_set_extmark(buf, ns, {}, 0, {{line_hl_group = 'GdAdded'}})",
                    buf_line - 1,
                ));
                i += 1;
            }
            LineKind::Deleted => {
                // Collect consecutive deleted lines
                let del_start = i;
                let mut deleted_block = Vec::new();
                while i < lines.len() && lines[i].kind == LineKind::Deleted {
                    deleted_block.push(&lines[i]);
                    i += 1;
                }

                // Check if followed by added lines (for word-level diff)
                let mut added_block = Vec::new();
                while i < lines.len() && lines[i].kind == LineKind::Added {
                    added_block.push(&lines[i]);
                    i += 1;
                }

                // Mark start of change block
                if del_start == 0 || lines[del_start - 1].kind == LineKind::Context {
                    hunk_starts.push(buf_line + 1);
                }

                if added_block.is_empty() {
                    // Pure deleted block — attach as virtual lines
                    let virt = build_virt_lines(&deleted_block, buf_line);
                    extmark_calls.push(virt);
                } else {
                    // Paired deleted+added: do word-level diffs
                    let (del_virts, add_marks) =
                        word_diff_paired(&deleted_block, &added_block, buf_line);
                    extmark_calls.extend(del_virts);

                    for al in &added_block {
                        buf_line += 1;
                        line_map_entries.push(format!(
                            "[\"{buf_line}\"] = {{new = {}}}",
                            opt_num(al.new_lineno),
                        ));
                        extmark_calls.push(format!(
                            "vim.api.nvim_buf_set_extmark(buf, ns, {}, 0, {{line_hl_group = 'GdAdded'}})",
                            buf_line - 1,
                        ));
                    }
                    extmark_calls.extend(add_marks);
                }
            }
        }
    }

    // Build virt_map for statuscolumn
    let virt_map = build_virt_map(lines);

    let line_map_str = line_map_entries.join(", ");
    let hunk_starts_str = hunk_starts
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ");

    let lua = format!(
        r"
        local buf = vim.api.nvim_get_current_buf()
        local ns = vim.api.nvim_create_namespace('gd_diff')
        vim.api.nvim_buf_clear_namespace(buf, ns, 0, -1)

        vim.b.gd_line_map = {{{line_map_str}}}
        vim.b.gd_virt_map = {{{virt_map}}}
        vim.b.gd_hunk_starts = {{{hunk_starts_str}}}

        {extmarks}
        ",
        extmarks = extmark_calls.join("\n        "),
    );

    (lua, vec![])
}

fn opt_num(v: Option<u32>) -> String {
    match v {
        Some(n) => n.to_string(),
        None => "nil".into(),
    }
}

/// Build virtual lines extmark for a block of deleted lines attached after `anchor_buf_line`.
fn build_virt_lines(deleted: &[&DiffLine], anchor_buf_line: u32) -> String {
    let mut virt_lines = Vec::new();
    for dl in deleted {
        let content = lua_escape(&dl.content);
        virt_lines.push(format!(
            "{{{{\"{content}\", \"GdDeleted\"}}}}",
        ));
    }
    let virt_str = virt_lines.join(", ");
    if anchor_buf_line == 0 {
        format!(
            "vim.api.nvim_buf_set_extmark(buf, ns, 0, 0, {{virt_lines = {{{virt_str}}}, virt_lines_above = true}})"
        )
    } else {
        format!(
            "vim.api.nvim_buf_set_extmark(buf, ns, {}, 0, {{virt_lines = {{{virt_str}}}}})",
            anchor_buf_line - 1,
        )
    }
}

/// Build virt_map Lua table for statuscolumn rendering of virtual lines.
fn build_virt_map(lines: &[DiffLine]) -> String {
    // Track which buffer line each deleted block is anchored to
    let mut entries = Vec::new();
    let mut buf_line: u32 = 0;
    let mut i = 0;

    while i < lines.len() {
        match lines[i].kind {
            LineKind::Context | LineKind::Added => {
                buf_line += 1;
                i += 1;
            }
            LineKind::Deleted => {
                let mut del_entries = Vec::new();
                while i < lines.len() && lines[i].kind == LineKind::Deleted {
                    let old = opt_num(lines[i].old_lineno);
                    del_entries.push(format!("{{old = {old}}}"));
                    i += 1;
                }

                // Check if followed by added (they consume the same anchor)
                while i < lines.len() && lines[i].kind == LineKind::Added {
                    buf_line += 1;
                    i += 1;
                }

                let anchor = if buf_line == 0 { 1 } else { buf_line };
                let inner = del_entries.join(", ");
                entries.push(format!("[\"{anchor}\"] = {{{inner}}}"));
            }
        }
    }
    entries.join(", ")
}

/// Generate word-level diff extmarks for paired deleted+added blocks.
fn word_diff_paired(
    deleted: &[&DiffLine],
    added: &[&DiffLine],
    buf_line_before: u32,
) -> (Vec<String>, Vec<String>) {
    let mut virt_extmarks = Vec::new();
    let mut add_extmarks = Vec::new();

    let pair_count = deleted.len().min(added.len());

    // Build virtual lines for deleted with word-level highlights
    let mut virt_lines = Vec::new();
    for (idx, dl) in deleted.iter().enumerate() {
        if idx < pair_count {
            let al = added[idx];
            let diff = TextDiff::from_words(&dl.content, &al.content);

            // Build virtual line chunks for deleted
            let mut chunks = Vec::new();
            for change in diff.iter_all_changes() {
                let val = lua_escape(change.value());
                match change.tag() {
                    ChangeTag::Delete => {
                        chunks.push(format!("{{\"{val}\", \"GdDeletedWord\"}}"));
                    }
                    ChangeTag::Equal => {
                        chunks.push(format!("{{\"{val}\", \"GdDeleted\"}}"));
                    }
                    ChangeTag::Insert => {} // skip inserts in deleted line
                }
            }
            virt_lines.push(format!("{{{}}}", chunks.join(", ")));

            // Build column-range extmarks for added line
            let add_buf_line = buf_line_before + idx as u32 + 1; // 1-indexed
            let mut col: u32 = 0;
            for change in diff.iter_all_changes() {
                let len = change.value().len() as u32;
                match change.tag() {
                    ChangeTag::Insert => {
                        add_extmarks.push(format!(
                            "vim.api.nvim_buf_set_extmark(buf, ns, {}, {col}, {{end_col = {}, hl_group = 'GdAddedWord'}})",
                            add_buf_line - 1,
                            col + len,
                        ));
                        col += len;
                    }
                    ChangeTag::Equal => {
                        col += len;
                    }
                    ChangeTag::Delete => {} // not on this line
                }
            }
        } else {
            // Unpaired deleted line — plain highlight
            let content = lua_escape(&dl.content);
            virt_lines.push(format!("{{{{\"{content}\", \"GdDeleted\"}}}}"));
        }
    }

    let virt_str = virt_lines.join(", ");
    let anchor = if buf_line_before == 0 {
        format!(
            "vim.api.nvim_buf_set_extmark(buf, ns, 0, 0, {{virt_lines = {{{virt_str}}}, virt_lines_above = true}})"
        )
    } else {
        format!(
            "vim.api.nvim_buf_set_extmark(buf, ns, {}, 0, {{virt_lines = {{{virt_str}}}}})",
            buf_line_before - 1,
        )
    };
    virt_extmarks.push(anchor);

    (virt_extmarks, add_extmarks)
}

fn lua_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .replace('\0', "")
}
