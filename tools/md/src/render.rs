use std::fmt::Write;

use indexmap::IndexMap;
use pulldown_cmark::{Alignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::frontmatter::parse_frontmatter;
use crate::highlight::highlight_code;
use crate::style::Style;
use crate::wrap::{visible_length, word_wrap, wrap_line_for_display, wrap_line_greedy};

struct ListContext {
    ordered: bool,
    next_number: u64,
    items: Vec<String>,
    depth: usize,
    /// Number of items at the start of the current item — used to detect
    /// whether the first paragraph of a loose-list item still needs a marker.
    item_start_count: usize,
}

struct CodeBlockCtx {
    lang: Option<String>,
    content: String,
}

struct TableCtx {
    alignments: Vec<Alignment>,
    head_cells: Vec<String>,
    rows: Vec<Vec<String>>,
    current_row: Vec<String>,
    in_head: bool,
}

pub fn render_markdown(markdown: &str, width: usize, style: &Style) -> String {
    let parsed = parse_frontmatter(markdown);
    let mut parts = Vec::new();

    if let Some(ref fm) = parsed.frontmatter
        && !fm.is_empty()
    {
        parts.push(render_frontmatter(fm, width, style));
    }

    let body = render_tokens(&parsed.body, width, style);
    if !body.is_empty() {
        parts.push(body);
    }

    parts.join("\n\n")
}

pub fn render_frontmatter(
    attrs: &IndexMap<String, serde_yaml::Value>,
    width: usize,
    style: &Style,
) -> String {
    if attrs.is_empty() {
        return String::new();
    }

    let max_key_len = attrs
        .keys()
        .map(std::string::String::len)
        .max()
        .unwrap_or(0);
    let indent = " ".repeat(max_key_len + 2);

    attrs
        .iter()
        .map(|(key, value)| {
            let padded_key = format!("{key:max_key_len$}");
            let formatted = format_value(value);

            if width > 0 {
                let first_line_width = width.saturating_sub(max_key_len + 2);
                let wrapped = word_wrap(&formatted, first_line_width, "");
                let lines: Vec<&str> = wrapped.split('\n').collect();
                let styled_lines: Vec<String> = lines
                    .iter()
                    .enumerate()
                    .map(|(i, line)| {
                        if i == 0 {
                            format!(
                                "{}  {}",
                                style.frontmatter_key(&padded_key),
                                style.frontmatter_value(line)
                            )
                        } else {
                            format!("{}{}", indent, style.frontmatter_value(line))
                        }
                    })
                    .collect();
                styled_lines.join("\n")
            } else {
                format!(
                    "{}  {}",
                    style.frontmatter_key(&padded_key),
                    style.frontmatter_value(&formatted)
                )
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_value(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::Null => String::new(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Sequence(seq) => seq
            .iter()
            .map(|v| match v {
                serde_yaml::Value::String(s) => s.clone(),
                other => format!("{other:?}"),
            })
            .collect::<Vec<_>>()
            .join(", "),
        serde_yaml::Value::Mapping(_) => serde_json::to_string(value).unwrap_or_default(),
        serde_yaml::Value::Tagged(t) => format_value(&t.value),
    }
}

pub fn render_tokens(markdown_body: &str, width: usize, style: &Style) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(markdown_body, options);
    let events: Vec<Event> = parser.collect();

    let mut output_parts: Vec<String> = Vec::new();
    let mut inline_buffer = String::new();
    let mut list_stack: Vec<ListContext> = Vec::new();
    let mut blockquote_depth: usize = 0;
    let mut blockquote_buffer: Vec<Vec<String>> = Vec::new();
    let mut code_block: Option<CodeBlockCtx> = None;
    let mut in_strong = false;
    let mut in_emphasis = false;
    let mut in_strikethrough = false;
    let mut link_dest: Vec<String> = Vec::new();
    let mut heading_level: Option<HeadingLevel> = None;
    let mut table: Option<TableCtx> = None;
    let mut footnotes: Vec<(String, Vec<String>)> = Vec::new();
    let mut footnote_labels: Vec<String> = Vec::new();
    let mut current_footnote: Option<(String, Vec<String>)> = None;

    for event in events {
        // Code block accumulation
        if let Some(ref mut cb) = code_block {
            match event {
                Event::Text(text) => {
                    cb.content.push_str(&text);
                    continue;
                }
                Event::End(TagEnd::CodeBlock) => {
                    let mut content = cb.content.clone();
                    // Remove trailing newline from pulldown-cmark
                    if content.ends_with('\n') {
                        content.pop();
                    }

                    let lang = cb.lang.clone();
                    let highlighted = highlight_code(&content, lang.as_deref(), style.color);

                    let block = if style.pretty {
                        render_code_block_pretty(&highlighted, lang.as_deref(), width, style)
                    } else {
                        let opening = match &lang {
                            Some(l) => {
                                format!("{}{}", style.marker("```"), style.code_language(l))
                            }
                            None => style.marker("```"),
                        };
                        format!("{}\n{}\n{}", opening, highlighted, style.marker("```"))
                    };

                    code_block = None;
                    push_block(
                        &mut output_parts,
                        &mut list_stack,
                        &mut blockquote_buffer,
                        &mut current_footnote,
                        block,
                    );
                    continue;
                }
                _ => continue,
            }
        }

        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                heading_level = Some(level);
                inline_buffer.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                let level = heading_level.take().unwrap_or(HeadingLevel::H1);
                let styled_text = apply_heading_style(style, level, &inline_buffer);
                let heading_line =
                    if style.pretty && matches!(level, HeadingLevel::H1 | HeadingLevel::H2) {
                        styled_text
                    } else {
                        let level_num = heading_level_num(level);
                        let prefix = style.marker(&"#".repeat(level_num));
                        format!("{prefix} {styled_text}")
                    };
                let block = if style.pretty
                    && matches!(level, HeadingLevel::H1 | HeadingLevel::H2)
                {
                    let vis_len = visible_length(&heading_line);
                    let ch = if matches!(level, HeadingLevel::H1) { "═" } else { "─" };
                    let underline = style.marker(&ch.repeat(vis_len));
                    format!("{heading_line}\n{underline}")
                } else {
                    heading_line
                };
                inline_buffer.clear();
                push_block(
                    &mut output_parts,
                    &mut list_stack,
                    &mut blockquote_buffer,
                    &mut current_footnote,
                    block,
                );
            }
            Event::Start(Tag::Paragraph | Tag::Item | Tag::TableCell) => {
                if matches!(event, Event::Start(Tag::Item)) {
                    if let Some(ctx) = list_stack.last_mut() {
                        ctx.item_start_count = ctx.items.len();
                    }
                }
                inline_buffer.clear();
            }
            Event::End(TagEnd::Paragraph) => {
                let text = std::mem::take(&mut inline_buffer);
                if !list_stack.is_empty() && blockquote_depth == 0 {
                    // Inside list: format as list item immediately so block
                    // elements (code blocks, etc.) that follow stay in order.
                    if let Some(ctx) = list_stack.last_mut() {
                        if ctx.items.len() == ctx.item_start_count {
                            // First paragraph of item: add marker
                            let marker = make_list_marker(ctx, style);
                            ctx.items.push(format_list_item(
                                &text, &marker, ctx.depth, width,
                            ));
                        } else {
                            // Subsequent paragraph: continuation indent
                            let indent = "    ".repeat(ctx.depth);
                            let marker_width =
                                if ctx.ordered { 3 } else { 2 }; // "- " or "1. "
                            let content_indent =
                                format!("{indent}{}", " ".repeat(marker_width));
                            let content_width =
                                width.saturating_sub(indent.len() + marker_width);
                            let wrapped = word_wrap(&text, content_width, "");
                            let indented: String = wrapped
                                .lines()
                                .map(|l| format!("{content_indent}{l}"))
                                .collect::<Vec<_>>()
                                .join("\n");
                            if let Some(last) = ctx.items.last_mut() {
                                last.push('\n');
                                last.push_str(&indented);
                            }
                        }
                    }
                } else {
                    let wrapped = if blockquote_depth > 0 {
                        word_wrap(&text, width.saturating_sub(blockquote_depth * 3), "")
                    } else {
                        word_wrap(&text, width, "")
                    };
                    push_block(
                        &mut output_parts,
                        &mut list_stack,
                        &mut blockquote_buffer,
                        &mut current_footnote,
                        wrapped,
                    );
                }
            }
            Event::Start(Tag::Strong) => {
                if !(style.pretty && style.color) {
                    inline_buffer.push_str(&style.marker("**"));
                }
                in_strong = true;
            }
            Event::End(TagEnd::Strong) => {
                if !(style.pretty && style.color) {
                    inline_buffer.push_str(&style.marker("**"));
                }
                in_strong = false;
            }
            Event::Start(Tag::Emphasis) => {
                if !(style.pretty && style.color) {
                    inline_buffer.push_str(&style.marker("*"));
                }
                in_emphasis = true;
            }
            Event::End(TagEnd::Emphasis) => {
                if !(style.pretty && style.color) {
                    inline_buffer.push_str(&style.marker("*"));
                }
                in_emphasis = false;
            }
            Event::Start(Tag::Strikethrough) => {
                if !(style.pretty && style.color) {
                    inline_buffer.push_str(&style.marker("~~"));
                }
                in_strikethrough = true;
            }
            Event::End(TagEnd::Strikethrough) => {
                if !(style.pretty && style.color) {
                    inline_buffer.push_str(&style.marker("~~"));
                }
                in_strikethrough = false;
            }
            Event::Code(text) => {
                inline_buffer.push_str(&style.code_span(&text));
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                if !style.pretty {
                    inline_buffer.push_str(&style.marker("["));
                }
                link_dest.push(dest_url.to_string());
            }
            Event::End(TagEnd::Link) => {
                let dest = link_dest.pop().unwrap_or_default();
                if style.pretty {
                    let _ = write!(
                        inline_buffer,
                        " ({})",
                        style.link_url(&dest),
                    );
                } else {
                    let _ = write!(
                        inline_buffer,
                        "{}{}{}",
                        style.marker("]("),
                        style.link_url(&dest),
                        style.marker(")")
                    );
                }
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(ref lang) if !lang.is_empty() => Some(lang.to_string()),
                    _ => None,
                };
                code_block = Some(CodeBlockCtx {
                    lang,
                    content: String::new(),
                });
            }
            Event::Start(Tag::List(start)) => {
                // If we're inside a list item and have accumulated text,
                // commit it as the item's text before starting the nested list
                if !list_stack.is_empty() && !inline_buffer.is_empty() {
                    let text = std::mem::take(&mut inline_buffer);
                    if let Some(ctx) = list_stack.last_mut() {
                        let marker = make_list_marker(ctx, style);
                        ctx.items
                            .push(format_list_item(&text, &marker, ctx.depth, width));
                    }
                }
                let depth = list_stack.len();
                list_stack.push(ListContext {
                    ordered: start.is_some(),
                    next_number: start.unwrap_or(1),
                    items: Vec::new(),
                    depth,
                    item_start_count: 0,
                });
            }
            Event::End(TagEnd::Item) => {
                let text = std::mem::take(&mut inline_buffer);
                if !text.is_empty()
                    && let Some(ctx) = list_stack.last_mut()
                {
                    let marker = make_list_marker(ctx, style);
                    ctx.items
                        .push(format_list_item(&text, &marker, ctx.depth, width));
                }
                // If text is empty, the item was already committed by Start(List)
            }
            Event::End(TagEnd::List(_)) => {
                if let Some(ctx) = list_stack.pop() {
                    let block = ctx.items.join("\n");
                    if list_stack.is_empty() {
                        push_block(
                            &mut output_parts,
                            &mut list_stack,
                            &mut blockquote_buffer,
                            &mut current_footnote,
                            block,
                        );
                    } else {
                        // Nested list: append to parent's current item
                        if let Some(parent) = list_stack.last_mut() {
                            if let Some(last_item) = parent.items.last_mut() {
                                last_item.push('\n');
                                last_item.push_str(&block);
                            } else {
                                parent.items.push(block);
                            }
                        }
                    }
                }
            }
            Event::Start(Tag::BlockQuote(_)) => {
                blockquote_depth += 1;
                blockquote_buffer.push(Vec::new());
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                blockquote_depth = blockquote_depth.saturating_sub(1);
                if let Some(inner_parts) = blockquote_buffer.pop() {
                    let inner = inner_parts.join("\n\n");
                    let bq_marker = if style.pretty { "│" } else { ">" };
                    let prefixed: String = inner
                        .lines()
                        .map(|line| {
                            format!("{} {}", style.marker(bq_marker), style.blockquote_text(line))
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    push_block(
                        &mut output_parts,
                        &mut list_stack,
                        &mut blockquote_buffer,
                        &mut current_footnote,
                        prefixed,
                    );
                }
            }
            Event::Rule => {
                let hr_text = if style.pretty {
                    "─".repeat(width)
                } else {
                    "---".to_string()
                };
                let block = style.hr_style(&hr_text);
                push_block(
                    &mut output_parts,
                    &mut list_stack,
                    &mut blockquote_buffer,
                    &mut current_footnote,
                    block,
                );
            }
            Event::Text(text) => {
                let styled = if !link_dest.is_empty() {
                    style.link_text(&text)
                } else if in_strikethrough {
                    style.strikethrough_style(&text)
                } else if in_strong && in_emphasis {
                    style.strong_style(&style.em_style(&text))
                } else if in_strong {
                    style.strong_style(&text)
                } else if in_emphasis {
                    style.em_style(&text)
                } else {
                    text.to_string()
                };
                inline_buffer.push_str(&styled);
            }
            Event::SoftBreak => {
                inline_buffer.push(' ');
            }
            Event::HardBreak => {
                inline_buffer.push('\n');
            }
            Event::Html(text) => {
                // Block-level HTML: treat as a paragraph (word-wrap the raw text)
                let wrapped = word_wrap(&text, width, "");
                push_block(
                    &mut output_parts,
                    &mut list_stack,
                    &mut blockquote_buffer,
                    &mut current_footnote,
                    wrapped,
                );
            }
            Event::InlineHtml(text) => {
                // Inline HTML: pass through as-is (e.g., <br>, <!-- comment -->)
                inline_buffer.push_str(&text);
            }
            Event::Start(Tag::Table(alignments)) => {
                table = Some(TableCtx {
                    alignments: alignments.clone(),
                    head_cells: Vec::new(),
                    rows: Vec::new(),
                    current_row: Vec::new(),
                    in_head: false,
                });
            }
            Event::Start(Tag::TableHead) => {
                if let Some(ref mut t) = table {
                    t.in_head = true;
                    t.current_row.clear();
                }
            }
            Event::End(TagEnd::TableHead) => {
                if let Some(ref mut t) = table {
                    t.head_cells = std::mem::take(&mut t.current_row);
                    t.in_head = false;
                }
            }
            Event::Start(Tag::TableRow) => {
                if let Some(ref mut t) = table {
                    t.current_row.clear();
                }
            }
            Event::End(TagEnd::TableRow) => {
                if let Some(ref mut t) = table
                    && !t.in_head
                {
                    let row = std::mem::take(&mut t.current_row);
                    t.rows.push(row);
                }
            }
            Event::End(TagEnd::TableCell) => {
                let cell = std::mem::take(&mut inline_buffer);
                if let Some(ref mut t) = table {
                    t.current_row.push(cell);
                }
            }
            Event::End(TagEnd::Table) => {
                if let Some(t) = table.take() {
                    let block = render_table(&t, width, style);
                    push_block(
                        &mut output_parts,
                        &mut list_stack,
                        &mut blockquote_buffer,
                        &mut current_footnote,
                        block,
                    );
                }
            }
            Event::TaskListMarker(checked) => {
                let marker = if style.pretty {
                    if checked { style.task_marker("☑") } else { style.task_marker("☐") }
                } else if checked {
                    style.task_marker("[x]")
                } else {
                    style.task_marker("[ ]")
                };
                inline_buffer.insert_str(0, &format!("{marker} "));
            }
            Event::FootnoteReference(label) => {
                let label_str = label.to_string();
                let num = if let Some(pos) =
                    footnote_labels.iter().position(|l| l == &label_str)
                {
                    pos + 1
                } else {
                    footnote_labels.push(label_str);
                    footnote_labels.len()
                };
                inline_buffer
                    .push_str(&style.footnote_ref(&format!("[{num}]")));
            }
            Event::Start(Tag::FootnoteDefinition(label)) => {
                current_footnote = Some((label.to_string(), Vec::new()));
            }
            Event::End(TagEnd::FootnoteDefinition) => {
                if let Some(footnote) = current_footnote.take() {
                    footnotes.push(footnote);
                }
            }
            _ => {}
        }
    }

    // Render footnote definitions at the bottom
    if !footnotes.is_empty() {
        let fn_hr = if style.pretty {
            "─".repeat(width)
        } else {
            "---".to_string()
        };
        output_parts.push(style.hr_style(&fn_hr));
        for (label, body_parts) in &footnotes {
            let num = footnote_labels
                .iter()
                .position(|l| l == label)
                .map_or(0, |i| i + 1);
            let prefix = style.footnote_ref(&format!("[{num}]"));
            let body = body_parts.join(" ");
            let wrapped = word_wrap(
                &format!("{prefix} {body}"),
                width,
                "",
            );
            output_parts.push(wrapped);
        }
    }

    output_parts.join("\n\n")
}

const MIN_COL_WIDTH: usize = 3;

/// Shrink column widths to fit within target_width, protecting narrow columns.
///
/// Narrow columns (those at or below the fair share of available space) keep
/// their natural width. Only wide columns absorb overflow, proportionally.
fn shrink_columns(col_widths: &mut [usize], target_width: usize) {
    let num_cols = col_widths.len();
    let total_width: usize = col_widths.iter().sum::<usize>() + 3 * num_cols + 1;
    if total_width <= target_width {
        return;
    }

    let available = target_width.saturating_sub(3 * num_cols + 1);

    // Iteratively freeze narrow columns at their natural width so only
    // wide columns absorb overflow.
    let mut frozen = vec![false; num_cols];
    let mut frozen_total = 0usize;
    let mut unfrozen_count = num_cols;

    loop {
        if unfrozen_count == 0 {
            break;
        }
        let fair_share = available.saturating_sub(frozen_total) / unfrozen_count;

        let mut changed = false;
        for i in 0..num_cols {
            if !frozen[i] && col_widths[i] <= fair_share {
                frozen[i] = true;
                frozen_total += col_widths[i];
                unfrozen_count -= 1;
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }

    if unfrozen_count == 0 {
        return;
    }

    // Distribute remaining space proportionally among unfrozen columns.
    let remaining = available.saturating_sub(frozen_total);
    let unfrozen_total: usize = (0..num_cols)
        .filter(|i| !frozen[*i])
        .map(|i| col_widths[i])
        .sum();

    if unfrozen_total == 0 {
        return;
    }

    let unfrozen_indices: Vec<usize> = (0..num_cols).filter(|i| !frozen[*i]).collect();
    let mut allocated = 0usize;

    for (idx, &i) in unfrozen_indices.iter().enumerate() {
        let new_width = if idx == unfrozen_indices.len() - 1 {
            remaining.saturating_sub(allocated)
        } else {
            (col_widths[i] * remaining + unfrozen_total / 2) / unfrozen_total
        };
        col_widths[i] = new_width.max(MIN_COL_WIDTH);
        allocated += col_widths[i];
    }
}

/// Wrap a cell's text to fit within max_width, returning one string per visual line.
fn wrap_cell(cell: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![cell.to_string()];
    }
    let vis_width = visible_length(cell);
    if vis_width <= max_width {
        return vec![cell.to_string()];
    }
    wrap_line_greedy(cell, max_width)
        .iter()
        .flat_map(|line| wrap_line_for_display(line, max_width))
        .collect()
}

fn render_table(table: &TableCtx, width: usize, style: &Style) -> String {
    let num_cols = table.alignments.len();

    // Compute natural column widths from visible text
    let mut col_widths = vec![0usize; num_cols];
    for (i, cell) in table.head_cells.iter().enumerate() {
        if i < num_cols {
            col_widths[i] = col_widths[i].max(visible_length(cell));
        }
    }
    for row in &table.rows {
        for (i, cell) in row.iter().enumerate() {
            if i < num_cols {
                col_widths[i] = col_widths[i].max(visible_length(cell));
            }
        }
    }

    // Constrain to available width
    if width > 0 {
        shrink_columns(&mut col_widths, width);
    }

    let pipe = if style.pretty { "│" } else { "|" };
    let dash = if style.pretty { "─" } else { "-" };

    // Build a horizontal border line: left + segments + right
    let border_line = |left: &str, mid: &str, right: &str| -> String {
        let segments: Vec<String> = col_widths
            .iter()
            .map(|&w| dash.repeat(w + 2))
            .collect();
        style.table_border(&format!("{}{}{}", left, segments.join(mid), right))
    };

    // Build separator line (between head and body)
    let separator_line = || -> String {
        if style.pretty {
            border_line("├", "┼", "┤")
        } else {
            let segments: Vec<String> = col_widths
                .iter()
                .enumerate()
                .map(|(i, &w)| {
                    let dashes = w + 2;
                    match table.alignments.get(i) {
                        Some(Alignment::Left) => format!(":{}{pipe}", "-".repeat(dashes - 1)),
                        Some(Alignment::Center) => {
                            format!(":{}:{pipe}", "-".repeat(dashes.saturating_sub(2)))
                        }
                        Some(Alignment::Right) => format!("{}:{pipe}", "-".repeat(dashes - 1)),
                        _ => format!("{}{pipe}", "-".repeat(dashes)),
                    }
                })
                .collect();
            style.table_border(&format!("{pipe}{}", segments.join("")))
        }
    };

    // Render a row of cells, wrapping content into multiple visual lines if needed.
    let format_row = |cells: &[String], bold_cells: bool| -> Vec<String> {
        let wrapped: Vec<Vec<String>> = cells
            .iter()
            .enumerate()
            .take(num_cols)
            .map(|(i, cell)| wrap_cell(cell, col_widths[i]))
            .collect();

        let row_height = wrapped.iter().map(std::vec::Vec::len).max().unwrap_or(1);
        let sep = format!(" {} ", style.table_border(pipe));

        let mut row_lines = Vec::new();
        for line_idx in 0..row_height {
            let mut parts = Vec::new();
            for (i, cell_lines) in wrapped.iter().enumerate() {
                let text = cell_lines
                    .get(line_idx)
                    .map_or("", std::string::String::as_str);
                let vis_len = visible_length(text);
                let pad = col_widths[i].saturating_sub(vis_len);
                let content = if bold_cells {
                    style.table_header(text)
                } else {
                    text.to_string()
                };
                let padded = match table.alignments.get(i) {
                    Some(Alignment::Center) => {
                        let left = pad / 2;
                        let right = pad - left;
                        format!("{}{}{}", " ".repeat(left), content, " ".repeat(right))
                    }
                    Some(Alignment::Right) => {
                        format!("{}{}", " ".repeat(pad), content)
                    }
                    _ => {
                        format!("{}{}", content, " ".repeat(pad))
                    }
                };
                parts.push(padded);
            }
            row_lines.push(format!(
                "{} {} {}",
                style.table_border(pipe),
                parts.join(&sep),
                style.table_border(pipe)
            ));
        }
        row_lines
    };

    let mut lines = Vec::new();
    if style.pretty {
        lines.push(border_line("┌", "┬", "┐"));
    }
    lines.extend(format_row(&table.head_cells, true));
    lines.push(separator_line());
    for row in &table.rows {
        lines.extend(format_row(row, false));
    }
    if style.pretty {
        lines.push(border_line("└", "┴", "┘"));
    }

    lines.join("\n")
}

fn render_code_block_pretty(
    highlighted: &str,
    lang: Option<&str>,
    width: usize,
    style: &Style,
) -> String {
    // inner_width = width minus the two border chars (╭/│/╰ and ╮/│/╯)
    let inner = width.saturating_sub(2);

    // Top border: ╭─ lang ───...╮ or ╭───...╮
    let top = match lang {
        Some(l) => {
            // "─ {lang} " then fill with ─
            let label_vis = 2 + l.len() + 1; // "─ " + lang + " "
            let fill = inner.saturating_sub(label_vis);
            format!(
                "{}{}{}{}",
                style.marker("╭"),
                style.marker(&format!("─ ")),
                style.code_language(&format!("{} ", l)),
                style.marker(&format!("{}╮", "─".repeat(fill)))
            )
        }
        None => {
            format!(
                "{}{}",
                style.marker(&format!("╭{}", "─".repeat(inner))),
                style.marker("╮")
            )
        }
    };

    // Content lines: │ code ... │
    let content_lines: Vec<String> = highlighted
        .lines()
        .map(|line| {
            let vis_len = visible_length(line);
            // 1 char for " " after │, pad to fill inner width
            let pad = inner.saturating_sub(vis_len + 1);
            format!(
                "{} {}{}{}",
                style.marker("│"),
                line,
                " ".repeat(pad),
                style.marker("│")
            )
        })
        .collect();

    // Bottom border: ╰───...╯
    let bottom = format!(
        "{}{}{}",
        style.marker("╰"),
        style.marker(&"─".repeat(inner)),
        style.marker("╯")
    );

    let mut lines = vec![top];
    lines.extend(content_lines);
    lines.push(bottom);
    lines.join("\n")
}

fn push_block(
    output_parts: &mut Vec<String>,
    list_stack: &mut [ListContext],
    blockquote_buffer: &mut [Vec<String>],
    current_footnote: &mut Option<(String, Vec<String>)>,
    block: String,
) {
    if block.is_empty() {
        return;
    }
    if let Some(ref mut footnote) = *current_footnote {
        footnote.1.push(block);
    } else if let Some(bq_parts) = blockquote_buffer.last_mut() {
        bq_parts.push(block);
    } else if !list_stack.is_empty() {
        // Inside a list item — append to inline buffer via the item
        // This case is handled by code block inside list items
        if let Some(ctx) = list_stack.last_mut() {
            if let Some(last_item) = ctx.items.last_mut() {
                last_item.push('\n');
                last_item.push_str(&block);
            } else {
                ctx.items.push(block);
            }
        }
    } else {
        output_parts.push(block);
    }
}

const PRETTY_BULLETS: [&str; 3] = ["•", "◦", "▪"];

fn make_list_marker(ctx: &mut ListContext, style: &Style) -> String {
    if ctx.ordered {
        let m = style.list_marker(&format!("{}.", ctx.next_number));
        ctx.next_number += 1;
        m
    } else if style.pretty {
        style.list_marker(PRETTY_BULLETS[ctx.depth % PRETTY_BULLETS.len()])
    } else {
        style.list_marker("-")
    }
}

fn format_list_item(text: &str, marker: &str, depth: usize, width: usize) -> String {
    let indent = "    ".repeat(depth);
    let marker_vis = visible_length(marker);
    let full_prefix_width = indent.len() + marker_vis + 1;
    let content_width = width.saturating_sub(full_prefix_width);

    if content_width == 0 {
        return format!("{indent}{marker} {text}");
    }

    let wrapped = word_wrap(text, content_width, "");
    let lines: Vec<&str> = wrapped.split('\n').collect();
    let content_indent = format!("{indent}{}", " ".repeat(marker_vis + 1));

    let mut result = format!("{indent}{marker} {}", lines[0]);
    for line in &lines[1..] {
        result.push('\n');
        result.push_str(&content_indent);
        result.push_str(line);
    }
    result
}

fn heading_level_num(level: HeadingLevel) -> usize {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn apply_heading_style(style: &Style, level: HeadingLevel, text: &str) -> String {
    match level {
        HeadingLevel::H1 => style.h1(text),
        HeadingLevel::H2 => style.h2(text),
        HeadingLevel::H3 => style.h3(text),
        HeadingLevel::H4 => style.h4(text),
        HeadingLevel::H5 => style.h5(text),
        HeadingLevel::H6 => style.h6(text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIDTH: usize = 60;

    fn render_plain(md: &str) -> String {
        let style = Style::new(false, false);
        render_tokens(md, WIDTH, &style)
    }

    fn render_md_plain(md: &str) -> String {
        let style = Style::new(false, false);
        render_markdown(md, WIDTH, &style)
    }

    // Group 1: direct rendering fixtures
    macro_rules! rendering_fixture {
        ($name:ident, $file:expr) => {
            #[test]
            fn $name() {
                let input = include_str!(concat!("../fixtures/rendering/", $file, ".md"));
                let expected =
                    include_str!(concat!("../fixtures/rendering/", $file, ".expected.txt"));
                let result = render_plain(input);
                assert_eq!(result, expected, "fixture: {}", $file);
            }
        };
    }

    rendering_fixture!(test_heading_h1, "heading-h1");
    rendering_fixture!(test_heading_h2, "heading-h2");
    rendering_fixture!(test_bold, "bold");
    rendering_fixture!(test_italic, "italic");
    rendering_fixture!(test_inline_code, "inline-code");
    rendering_fixture!(test_code_block_plain, "code-block-plain");
    rendering_fixture!(test_code_block_lang, "code-block-lang");
    rendering_fixture!(test_unordered_list, "unordered-list");
    rendering_fixture!(test_ordered_list, "ordered-list");
    rendering_fixture!(test_nested_list, "nested-list");
    rendering_fixture!(test_list_wrap, "list-wrap");
    rendering_fixture!(test_blockquote, "blockquote");
    rendering_fixture!(test_link, "link");
    rendering_fixture!(test_hr, "hr");
    rendering_fixture!(test_paragraph_wrap, "paragraph-wrap");
    rendering_fixture!(test_table, "table");
    rendering_fixture!(test_table_aligned, "table-aligned");
    rendering_fixture!(test_table_inline, "table-inline");
    rendering_fixture!(test_table_wide, "table-wide");
    rendering_fixture!(test_table_empty, "table-empty");
    rendering_fixture!(test_table_code_wrap, "table-code-wrap");
    rendering_fixture!(test_mixed_document, "mixed-document");
    rendering_fixture!(test_heading_h3, "heading-h3");
    rendering_fixture!(test_heading_h4, "heading-h4");
    rendering_fixture!(test_heading_h5, "heading-h5");
    rendering_fixture!(test_heading_h6, "heading-h6");
    rendering_fixture!(test_nested_blockquote, "nested-blockquote");
    rendering_fixture!(test_bold_italic, "bold-italic");
    rendering_fixture!(test_image, "image");
    rendering_fixture!(test_hard_break, "hard-break");
    rendering_fixture!(test_html_inline, "html-inline");
    rendering_fixture!(test_code_block_in_list, "code-block-in-list");
    rendering_fixture!(test_multiple_paragraphs, "multiple-paragraphs");
    rendering_fixture!(test_strikethrough, "strikethrough");
    rendering_fixture!(test_task_list, "task-list");
    rendering_fixture!(test_footnote, "footnote");
    rendering_fixture!(test_dangling_bracket, "dangling-bracket");

    // Group 1b: pretty rendering fixtures
    fn render_pretty(md: &str) -> String {
        let style = Style::new(false, true);
        render_tokens(md, WIDTH, &style)
    }

    macro_rules! pretty_fixture {
        ($name:ident, $file:expr) => {
            #[test]
            fn $name() {
                let input = include_str!(concat!("../fixtures/pretty/", $file, ".md"));
                let expected =
                    include_str!(concat!("../fixtures/pretty/", $file, ".expected.txt"));
                let result = render_pretty(input);
                assert_eq!(result, expected, "pretty fixture: {}", $file);
            }
        };
    }

    pretty_fixture!(test_pretty_heading_h1, "heading-h1");
    pretty_fixture!(test_pretty_heading_h2, "heading-h2");
    pretty_fixture!(test_pretty_code_block_lang, "code-block-lang");
    pretty_fixture!(test_pretty_code_block_plain, "code-block-plain");
    pretty_fixture!(test_pretty_blockquote, "blockquote");
    pretty_fixture!(test_pretty_hr, "hr");
    pretty_fixture!(test_pretty_unordered_list, "unordered-list");
    pretty_fixture!(test_pretty_nested_list, "nested-list");
    pretty_fixture!(test_pretty_task_list, "task-list");
    pretty_fixture!(test_pretty_table, "table");

    // Group 2: frontmatter fixtures (use render_markdown)
    macro_rules! frontmatter_fixture {
        ($name:ident, $file:expr) => {
            #[test]
            fn $name() {
                let input = include_str!(concat!("../fixtures/rendering/", $file, ".md"));
                let expected =
                    include_str!(concat!("../fixtures/rendering/", $file, ".expected.txt"));
                let result = render_md_plain(input);
                assert_eq!(result, expected, "fixture: {}", $file);
            }
        };
    }

    frontmatter_fixture!(test_frontmatter_basic, "frontmatter-basic");
    frontmatter_fixture!(test_frontmatter_arrays, "frontmatter-arrays");
    frontmatter_fixture!(test_frontmatter_empty, "frontmatter-empty");
    frontmatter_fixture!(test_frontmatter_malformed, "frontmatter-malformed");
    frontmatter_fixture!(test_bare_hr_not_frontmatter, "bare-hr-not-frontmatter");

    // Group 3: wrap_cell unit tests
    use crate::wrap::visible_length;

    // Group 4: render_frontmatter unit tests
    #[test]
    fn test_render_frontmatter_formats_key_value() {
        let style = Style::new(false, false);
        let mut attrs = IndexMap::new();
        attrs.insert("title".into(), serde_yaml::Value::String("Hello".into()));
        let result = render_frontmatter(&attrs, WIDTH, &style);
        assert!(result.contains("title"), "should contain key");
        assert!(result.contains("Hello"), "should contain value");
    }

    #[test]
    fn test_render_frontmatter_aligns_keys() {
        let style = Style::new(false, false);
        let mut attrs = IndexMap::new();
        attrs.insert("a".into(), serde_yaml::Value::String("short".into()));
        attrs.insert("longer".into(), serde_yaml::Value::String("val".into()));
        let result = render_frontmatter(&attrs, WIDTH, &style);
        let lines: Vec<&str> = result.lines().collect();
        // Both lines should have the value at the same column
        assert_eq!(lines.len(), 2);
        let pos0 = lines[0].find("short").unwrap();
        let pos1 = lines[1].find("val").unwrap();
        assert_eq!(pos0, pos1, "values should be aligned");
    }

    #[test]
    fn test_render_frontmatter_joins_arrays() {
        let style = Style::new(false, false);
        let mut attrs = IndexMap::new();
        attrs.insert(
            "tags".into(),
            serde_yaml::Value::Sequence(vec![
                serde_yaml::Value::String("a".into()),
                serde_yaml::Value::String("b".into()),
                serde_yaml::Value::String("c".into()),
            ]),
        );
        let result = render_frontmatter(&attrs, WIDTH, &style);
        assert!(
            result.contains("a, b, c"),
            "should join with commas, got: {result}"
        );
    }

    #[test]
    fn test_render_frontmatter_empty_map() {
        let style = Style::new(false, false);
        let attrs = IndexMap::new();
        let result = render_frontmatter(&attrs, WIDTH, &style);
        assert_eq!(result, "");
    }

    #[test]
    fn test_render_frontmatter_long_values_wrap() {
        let style = Style::new(false, false);
        let mut attrs = IndexMap::new();
        let long_value = "word ".repeat(30);
        attrs.insert(
            "desc".into(),
            serde_yaml::Value::String(long_value.trim().into()),
        );
        let result = render_frontmatter(&attrs, WIDTH, &style);
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.len() > 1, "long value should wrap to multiple lines");
        // Continuation lines should be indented
        for line in &lines[1..] {
            assert!(
                line.starts_with(' '),
                "continuation should be indented: {line:?}"
            );
        }
    }

    #[test]
    fn test_wrap_cell_preserves_ansi() {
        let styled = Style::new(true, false).code_span("done");
        // Force wrapping by using a narrow width
        let result = wrap_cell(&styled, 4);
        let joined = result.join("");
        assert!(
            joined.contains("\x1b["),
            "wrap_cell should preserve ANSI codes, got: {result:?}"
        );
    }

    #[test]
    fn test_wrap_cell_enforces_max_width() {
        let cell = "a `blah` end";
        let max = 5;
        let result = wrap_cell(cell, max);
        for line in &result {
            let vl = visible_length(line);
            assert!(
                vl <= max,
                "line exceeds max_width {max}: visible_length={vl}, line={line:?}"
            );
        }
    }

    // ── format_value tests ─────────────────────────────────

    #[test]
    fn test_format_value_null() {
        let result = format_value(&serde_yaml::Value::Null);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_value_bool() {
        assert_eq!(format_value(&serde_yaml::Value::Bool(true)), "true");
        assert_eq!(format_value(&serde_yaml::Value::Bool(false)), "false");
    }

    #[test]
    fn test_format_value_number() {
        let int: serde_yaml::Value = serde_yaml::from_str("42").unwrap();
        assert_eq!(format_value(&int), "42");

        let float: serde_yaml::Value = serde_yaml::from_str("3.14").unwrap();
        assert_eq!(format_value(&float), "3.14");
    }

    #[test]
    fn test_format_value_string() {
        let val = serde_yaml::Value::String("hello world".into());
        assert_eq!(format_value(&val), "hello world");
    }

    #[test]
    fn test_format_value_sequence() {
        let val = serde_yaml::Value::Sequence(vec![
            serde_yaml::Value::String("a".into()),
            serde_yaml::Value::String("b".into()),
            serde_yaml::Value::String("c".into()),
        ]);
        assert_eq!(format_value(&val), "a, b, c");
    }

    #[test]
    fn test_format_value_sequence_non_string() {
        let val = serde_yaml::Value::Sequence(vec![
            serde_yaml::Value::Bool(true),
            serde_yaml::Value::Number(serde_yaml::Number::from(1)),
        ]);
        let result = format_value(&val);
        // Non-string items use Debug formatting
        assert!(result.contains("Bool(true)"), "got: {result}");
    }

    #[test]
    fn test_format_value_mapping() {
        let val: serde_yaml::Value = serde_yaml::from_str("key: value").unwrap();
        let result = format_value(&val);
        // Mapping gets serialized as JSON
        assert!(result.contains("key"), "got: {result}");
        assert!(result.contains("value"), "got: {result}");
    }

    // ── shrink_columns tests ───────────────────────────────

    #[test]
    fn test_shrink_columns_no_op() {
        // 2 cols of width 5 each: total = 5+5 + 3*2 + 1 = 17
        // target = 20 → no shrink needed
        let mut widths = vec![5, 5];
        shrink_columns(&mut widths, 20);
        assert_eq!(widths, vec![5, 5]);
    }

    #[test]
    fn test_shrink_columns_proportional() {
        // 2 cols of width 20 each: total = 20+20 + 3*2 + 1 = 47
        // target = 40 → need to shrink by 7
        let mut widths = vec![20, 20];
        shrink_columns(&mut widths, 40);
        let total: usize = widths.iter().sum::<usize>() + 3 * 2 + 1;
        assert!(total <= 40, "total {total} should be <= 40");
    }

    #[test]
    fn test_shrink_columns_min_width() {
        // Columns at MIN_COL_WIDTH (3) can't shrink further
        let mut widths = vec![3, 3];
        let before = widths.clone();
        shrink_columns(&mut widths, 1);
        assert_eq!(widths, before, "columns at MIN_COL_WIDTH should not shrink");
    }

    #[test]
    fn test_list_item_no_dangling_bracket() {
        use crate::wrap::strip_ansi;

        let cases = [
            "- Directory browsing via `find` piped to `fzf` (via `$SHELL`) to pick a file",
            "- **Directory browsing** — when given a directory, uses `find` + `fzf` (via `$SHELL`) to pick a `.md`/`.mdx` file, then renders it",
            "- Terminal markdown renderer (Rust) with color output, syntax highlighting (github-dark theme), YAML frontmatter support, word wrapping, directory browsing (via `$SHELL` + `fzf`), and a built-in pager.",
            "- Creates symlinks using `ensure_symlink()` (backs up existing files to `.bak`)",
        ];
        for md in cases {
            for w in 20..120 {
                for color in [false, true] {
                    let style = Style::new(color, false);
                    let result = render_tokens(md, w, &style);
                    for (i, line) in result.lines().enumerate() {
                        let vis = strip_ansi(line).trim().to_string();
                        assert!(
                            !is_only_brackets(&vis),
                            "md={md:?}\nwidth={w} color={color} line {i} has dangling bracket: {vis:?}\nFull (stripped):\n{}",
                            result.lines().enumerate().map(|(j, l)| format!("  [{j}] {:?}", strip_ansi(l))).collect::<Vec<_>>().join("\n"),
                        );
                    }
                }
            }
        }
    }

    fn is_only_brackets(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| matches!(c, ')' | ']' | '.' | ',' | ';' | ':'))
    }

    #[test]
    fn test_loose_list_has_markers() {
        let md = "- First item\n\n- Second item\n";
        let result = render_plain(md);
        assert!(result.contains("- First item"), "should have marker: {result:?}");
        assert!(result.contains("- Second item"), "should have marker: {result:?}");
    }

    #[test]
    fn test_loose_list_wraps() {
        let md = "- When given a directory, it uses `find` piped to `fzf` (via `$SHELL`) to pick a `.md` file and render it.\n\n- Second\n";
        let style = Style::new(false, false);
        let result = render_tokens(md, 50, &style);
        // Should have list marker
        assert!(result.starts_with("- "), "should start with list marker: {result:?}");
        // Should be wrapped (multiple lines for first item)
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.len() > 2, "should wrap to multiple lines: {result:?}");
    }

    #[test]
    fn test_loose_list_no_dangling_bracket() {
        use crate::wrap::strip_ansi;

        let md = "- Uses `find` + `fzf` (via `$SHELL`) to pick a file\n\n- Second\n";
        for w in 20..80 {
            for color in [false, true] {
                let style = Style::new(color, false);
                let result = render_tokens(md, w, &style);
                for (i, line) in result.lines().enumerate() {
                    let vis = strip_ansi(line).trim().to_string();
                    assert!(
                        !is_only_brackets(&vis),
                        "width={w} color={color} line {i} has dangling bracket: {vis:?}\nFull:\n{}",
                        result.lines().enumerate().map(|(j, l)| format!("  [{j}] {:?}", strip_ansi(l))).collect::<Vec<_>>().join("\n"),
                    );
                }
            }
        }
    }

    #[test]
    fn test_shrink_columns_narrow_preserved() {
        // narrow (4) + wide (27): total = 4+27 + 3*2+1 = 38
        // target = 34 → narrow column should keep its natural width
        let mut widths = vec![4, 27];
        shrink_columns(&mut widths, 34);
        assert_eq!(widths[0], 4, "narrow column should not shrink");
        let total: usize = widths.iter().sum::<usize>() + 3 * 2 + 1;
        assert!(total <= 34, "total {total} should be <= 34");
    }
}
