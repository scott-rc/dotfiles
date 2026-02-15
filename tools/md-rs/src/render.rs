use indexmap::IndexMap;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser, Tag, TagEnd};

use crate::frontmatter::parse_frontmatter;
use crate::highlight::highlight_code;
use crate::style::Style;
use crate::wrap::word_wrap;

struct ListContext {
    ordered: bool,
    next_number: u64,
    items: Vec<String>,
    depth: usize,
}

struct CodeBlockCtx {
    lang: Option<String>,
    content: String,
}

pub fn render_markdown(markdown: &str, width: usize, style: &Style) -> String {
    let parsed = parse_frontmatter(markdown);
    let mut parts = Vec::new();

    if let Some(ref fm) = parsed.frontmatter {
        if !fm.is_empty() {
            parts.push(render_frontmatter(fm, width, style));
        }
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

    let max_key_len = attrs.keys().map(|k| k.len()).max().unwrap_or(0);
    let indent = " ".repeat(max_key_len + 2);

    attrs
        .iter()
        .map(|(key, value)| {
            let padded_key = format!("{:width$}", key, width = max_key_len);
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
                other => format!("{:?}", other),
            })
            .collect::<Vec<_>>()
            .join(", "),
        serde_yaml::Value::Mapping(_) => serde_json::to_string(value).unwrap_or_default(),
        serde_yaml::Value::Tagged(t) => format_value(&t.value),
    }
}

pub fn render_tokens(markdown_body: &str, width: usize, style: &Style) -> String {
    let parser = Parser::new(markdown_body);
    let events: Vec<Event> = parser.collect();

    let mut output_parts: Vec<String> = Vec::new();
    let mut inline_buffer = String::new();
    let mut list_stack: Vec<ListContext> = Vec::new();
    let mut blockquote_depth: usize = 0;
    let mut blockquote_buffer: Vec<Vec<String>> = Vec::new();
    let mut code_block: Option<CodeBlockCtx> = None;
    let mut in_strong = false;
    let mut in_emphasis = false;
    let mut link_dest: Vec<String> = Vec::new();
    let mut heading_level: Option<HeadingLevel> = None;

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

                    let opening = match &lang {
                        Some(l) => format!(
                            "{}{}",
                            style.marker("```"),
                            style.code_language(l)
                        ),
                        None => style.marker("```"),
                    };
                    let block = format!(
                        "{}\n{}\n{}",
                        opening,
                        highlighted,
                        style.marker("```")
                    );

                    code_block = None;
                    push_block(&mut output_parts, &mut list_stack, &mut blockquote_buffer, block);
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
                let level_num = heading_level_num(level);
                let prefix = style.marker(&"#".repeat(level_num));
                let styled_text = apply_heading_style(style, level, &inline_buffer);
                let block = format!("{} {}", prefix, styled_text);
                inline_buffer.clear();
                push_block(&mut output_parts, &mut list_stack, &mut blockquote_buffer, block);
            }
            Event::Start(Tag::Paragraph) => {
                inline_buffer.clear();
            }
            Event::End(TagEnd::Paragraph) => {
                let text = std::mem::take(&mut inline_buffer);
                if blockquote_depth > 0 || !list_stack.is_empty() {
                    // Inside blockquote or list: wrap at reduced width
                    let wrapped = if blockquote_depth > 0 {
                        word_wrap(&text, width.saturating_sub(blockquote_depth * 3), "")
                    } else {
                        // Inside list — wrapping will be applied when the item ends
                        text
                    };
                    push_block(&mut output_parts, &mut list_stack, &mut blockquote_buffer, wrapped);
                } else {
                    let wrapped = word_wrap(&text, width, "");
                    push_block(&mut output_parts, &mut list_stack, &mut blockquote_buffer, wrapped);
                }
            }
            Event::Start(Tag::Strong) => {
                inline_buffer.push_str(&style.marker("**"));
                in_strong = true;
            }
            Event::End(TagEnd::Strong) => {
                inline_buffer.push_str(&style.marker("**"));
                in_strong = false;
            }
            Event::Start(Tag::Emphasis) => {
                inline_buffer.push_str(&style.marker("*"));
                in_emphasis = true;
            }
            Event::End(TagEnd::Emphasis) => {
                inline_buffer.push_str(&style.marker("*"));
                in_emphasis = false;
            }
            Event::Code(text) => {
                inline_buffer.push_str(&style.code_span(&text));
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                inline_buffer.push_str(&style.marker("["));
                link_dest.push(dest_url.to_string());
            }
            Event::End(TagEnd::Link) => {
                let dest = link_dest.pop().unwrap_or_default();
                inline_buffer.push_str(&format!(
                    "{}{}{}",
                    style.marker("]("),
                    style.link_url(&dest),
                    style.marker(")")
                ));
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(ref lang) if !lang.is_empty() => {
                        Some(lang.to_string())
                    }
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
                        ctx.items.push(format!("{} {}", marker, text));
                    }
                }
                let depth = list_stack.len();
                list_stack.push(ListContext {
                    ordered: start.is_some(),
                    next_number: start.unwrap_or(1),
                    items: Vec::new(),
                    depth,
                });
            }
            Event::Start(Tag::Item) => {
                inline_buffer.clear();
            }
            Event::End(TagEnd::Item) => {
                let text = std::mem::take(&mut inline_buffer);
                if !text.is_empty() {
                    if let Some(ctx) = list_stack.last_mut() {
                        let marker = make_list_marker(ctx, style);
                        ctx.items.push(format!("{} {}", marker, text));
                    }
                }
                // If text is empty, the item was already committed by Start(List)
            }
            Event::End(TagEnd::List(_)) => {
                if let Some(ctx) = list_stack.pop() {
                    let indent = "    ".repeat(ctx.depth);
                    let formatted: Vec<String> = ctx
                        .items
                        .iter()
                        .map(|item| format!("{}{}", indent, item))
                        .collect();
                    let block = formatted.join("\n");
                    if list_stack.is_empty() {
                        push_block(
                            &mut output_parts,
                            &mut list_stack,
                            &mut blockquote_buffer,
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
                    let prefixed: String = inner
                        .lines()
                        .map(|line| {
                            format!("{} {}", style.marker(">"), style.blockquote_text(line))
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    push_block(
                        &mut output_parts,
                        &mut list_stack,
                        &mut blockquote_buffer,
                        prefixed,
                    );
                }
            }
            Event::Rule => {
                let block = style.hr_style("---");
                push_block(&mut output_parts, &mut list_stack, &mut blockquote_buffer, block);
            }
            Event::Text(text) => {
                let styled = if !link_dest.is_empty() {
                    style.link_text(&text)
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
                push_block(&mut output_parts, &mut list_stack, &mut blockquote_buffer, wrapped);
            }
            Event::InlineHtml(text) => {
                // Inline HTML: pass through as-is (e.g., <br>, <!-- comment -->)
                inline_buffer.push_str(&text);
            }
            _ => {}
        }
    }

    output_parts.join("\n\n")
}

fn push_block(
    output_parts: &mut Vec<String>,
    list_stack: &mut [ListContext],
    blockquote_buffer: &mut Vec<Vec<String>>,
    block: String,
) {
    if block.is_empty() {
        return;
    }
    if let Some(bq_parts) = blockquote_buffer.last_mut() {
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

fn make_list_marker(ctx: &mut ListContext, style: &Style) -> String {
    if ctx.ordered {
        let m = style.list_marker(&format!("{}.", ctx.next_number));
        ctx.next_number += 1;
        m
    } else {
        style.list_marker("-")
    }
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
        let style = Style::new(false);
        render_tokens(md, WIDTH, &style)
    }

    fn render_md_plain(md: &str) -> String {
        let style = Style::new(false);
        render_markdown(md, WIDTH, &style)
    }

    // Group 1: direct rendering fixtures
    macro_rules! rendering_fixture {
        ($name:ident, $file:expr) => {
            #[test]
            fn $name() {
                let input =
                    include_str!(concat!("../fixtures/rendering/", $file, ".md"));
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
    rendering_fixture!(test_blockquote, "blockquote");
    rendering_fixture!(test_link, "link");
    rendering_fixture!(test_hr, "hr");
    rendering_fixture!(test_paragraph_wrap, "paragraph-wrap");
    rendering_fixture!(test_mixed_document, "mixed-document");

    // Group 2: frontmatter fixtures (use render_markdown)
    macro_rules! frontmatter_fixture {
        ($name:ident, $file:expr) => {
            #[test]
            fn $name() {
                let input =
                    include_str!(concat!("../fixtures/rendering/", $file, ".md"));
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
}
