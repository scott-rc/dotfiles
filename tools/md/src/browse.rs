use std::io::Read;
use std::process::{Command, Stdio};

const DEFAULT_FIND: &str = "find {dir} -type f \\( -name '*.md' -o -name '*.mdx' \\)";

pub fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

pub fn build_find_cmd(dir: &str, template: Option<&str>) -> String {
    let t = template.unwrap_or(DEFAULT_FIND);
    let quoted = shell_quote(dir);
    if t.contains("{dir}") {
        t.replace("{dir}", &quoted)
    } else {
        format!("{t} {quoted}")
    }
}

pub fn build_pick_cmd(template: Option<&str>) -> String {
    template.unwrap_or("fzf").to_string()
}

pub fn build_browse_cmd(dir: &str, find_cmd: Option<&str>, pick_cmd: Option<&str>) -> String {
    format!(
        "{} | {}",
        build_find_cmd(dir, find_cmd),
        build_pick_cmd(pick_cmd)
    )
}

pub fn should_page(
    no_pager: bool,
    is_tty: bool,
    content_lines: usize,
    terminal_rows: usize,
    browsing: bool,
) -> bool {
    if no_pager || !is_tty {
        return false;
    }
    if browsing {
        return true;
    }
    content_lines > terminal_rows
}

pub fn parse_selection(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn browse_directory(
    dir: &str,
    mut view_file: impl FnMut(&str),
    find_cmd: Option<&str>,
    pick_cmd: Option<&str>,
) {
    let cmd = build_browse_cmd(dir, find_cmd, pick_cmd);
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string());

    loop {
        let child = Command::new(&shell)
            .args(["-c", &cmd])
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn();

        let Ok(mut child) = child else { break };

        let mut stdout_buf = String::new();
        if let Some(ref mut stdout) = child.stdout {
            let _ = stdout.read_to_string(&mut stdout_buf);
        }

        let Ok(status) = child.wait() else { break };

        if !status.success() {
            break;
        }

        match parse_selection(&stdout_buf) {
            Some(selection) => view_file(&selection),
            None => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    // shell-quote fixtures
    #[derive(Deserialize)]
    struct ShellQuoteCase {
        name: String,
        input: String,
        expected: String,
    }

    #[test]
    fn test_shell_quote() {
        let json = include_str!("../fixtures/browse/shell-quote.json");
        let cases: Vec<ShellQuoteCase> = serde_json::from_str(json).unwrap();
        for case in &cases {
            assert_eq!(
                shell_quote(&case.input),
                case.expected,
                "shell_quote: {}",
                case.name
            );
        }
    }

    // build-find-cmd fixtures
    #[derive(Deserialize)]
    struct BuildFindCmdCase {
        name: String,
        input: BuildFindCmdInput,
        expected: String,
    }

    #[derive(Deserialize)]
    struct BuildFindCmdInput {
        dir: String,
        template: Option<String>,
    }

    #[test]
    fn test_build_find_cmd() {
        let json = include_str!("../fixtures/browse/build-find-cmd.json");
        let cases: Vec<BuildFindCmdCase> = serde_json::from_str(json).unwrap();
        for case in &cases {
            let result = build_find_cmd(&case.input.dir, case.input.template.as_deref());
            assert_eq!(result, case.expected, "build_find_cmd: {}", case.name);
        }
    }

    // build-pick-cmd fixtures
    #[derive(Deserialize)]
    struct BuildPickCmdCase {
        name: String,
        input: BuildPickCmdInput,
        expected: String,
    }

    #[derive(Deserialize)]
    struct BuildPickCmdInput {
        template: Option<String>,
    }

    #[test]
    fn test_build_pick_cmd() {
        let json = include_str!("../fixtures/browse/build-pick-cmd.json");
        let cases: Vec<BuildPickCmdCase> = serde_json::from_str(json).unwrap();
        for case in &cases {
            let result = build_pick_cmd(case.input.template.as_deref());
            assert_eq!(result, case.expected, "build_pick_cmd: {}", case.name);
        }
    }

    // parse-selection fixtures
    #[derive(Deserialize)]
    struct ParseSelectionCase {
        name: String,
        input: String,
        expected: Option<String>,
    }

    #[test]
    fn test_parse_selection() {
        let json = include_str!("../fixtures/browse/parse-selection.json");
        let cases: Vec<ParseSelectionCase> = serde_json::from_str(json).unwrap();
        for case in &cases {
            let result = parse_selection(&case.input);
            assert_eq!(result, case.expected, "parse_selection: {}", case.name);
        }
    }

    // should-page fixtures
    #[derive(Deserialize)]
    struct ShouldPageCase {
        name: String,
        input: ShouldPageInput,
        expected: bool,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ShouldPageInput {
        no_pager: bool,
        #[serde(rename = "isTTY")]
        is_tty: bool,
        content_lines: usize,
        terminal_rows: usize,
        browsing: bool,
    }

    #[test]
    fn test_should_page() {
        let json = include_str!("../fixtures/browse/should-page.json");
        let cases: Vec<ShouldPageCase> = serde_json::from_str(json).unwrap();
        for case in &cases {
            let result = should_page(
                case.input.no_pager,
                case.input.is_tty,
                case.input.content_lines,
                case.input.terminal_rows,
                case.input.browsing,
            );
            assert_eq!(result, case.expected, "should_page: {}", case.name);
        }
    }
}
