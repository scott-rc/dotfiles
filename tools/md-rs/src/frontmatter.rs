use indexmap::IndexMap;

pub struct ParsedDocument {
    pub frontmatter: Option<IndexMap<String, serde_yaml::Value>>,
    pub body: String,
}

pub fn parse_frontmatter(markdown: &str) -> ParsedDocument {
    // Must start with "---\n"
    if !markdown.starts_with("---\n") {
        return ParsedDocument {
            frontmatter: None,
            body: markdown.to_string(),
        };
    }

    // Find closing "---" delimiter (search after the opening "---\n")
    let rest = &markdown[4..];

    // Handle empty frontmatter: rest starts directly with "---"
    let (yaml_content, after_closing) = if rest.starts_with("---\n") || rest == "---" {
        ("", &rest[3..])
    } else if let Some(pos) = rest.find("\n---") {
        (&rest[..pos], &rest[pos + 4..])
    } else {
        // No closing delimiter found — not frontmatter
        return ParsedDocument {
            frontmatter: None,
            body: markdown.to_string(),
        };
    };

    // Strip leading newline from body
    let body = if after_closing.starts_with('\n') {
        &after_closing[1..]
    } else {
        after_closing
    };

    // Try parsing YAML
    match serde_yaml::from_str::<serde_yaml::Value>(yaml_content) {
        Ok(serde_yaml::Value::Mapping(mapping)) => {
            let mut map = IndexMap::new();
            for (k, v) in mapping {
                if let serde_yaml::Value::String(key) = k {
                    map.insert(key, v);
                }
            }
            ParsedDocument {
                frontmatter: Some(map),
                body: body.to_string(),
            }
        }
        Ok(serde_yaml::Value::Null) => {
            // Empty frontmatter (---\n---)
            ParsedDocument {
                frontmatter: Some(IndexMap::new()),
                body: body.to_string(),
            }
        }
        _ => {
            // Malformed YAML — treat entire document as body
            ParsedDocument {
                frontmatter: None,
                body: markdown.to_string(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_extraction() {
        let input = "---\ntitle: Hello\n---\nbody text";
        let result = parse_frontmatter(input);
        let fm = result.frontmatter.expect("should have frontmatter");
        assert_eq!(fm.get("title").unwrap(), &serde_yaml::Value::String("Hello".into()));
        assert_eq!(result.body, "body text");
    }

    #[test]
    fn no_frontmatter() {
        let input = "# Hello";
        let result = parse_frontmatter(input);
        assert!(result.frontmatter.is_none());
        assert_eq!(result.body, "# Hello");
    }

    #[test]
    fn empty_frontmatter() {
        let input = "---\n---\nbody";
        let result = parse_frontmatter(input);
        let fm = result.frontmatter.expect("should have frontmatter");
        assert!(fm.is_empty());
        assert_eq!(result.body, "body");
    }

    #[test]
    fn malformed_yaml() {
        let input = "---\n: [[[\n---\nbody";
        let result = parse_frontmatter(input);
        assert!(result.frontmatter.is_none());
        assert_eq!(result.body, input);
    }

    #[test]
    fn bare_hr_not_frontmatter() {
        let input = "# Title\n\n---\n\ntext";
        let result = parse_frontmatter(input);
        assert!(result.frontmatter.is_none());
        assert_eq!(result.body, input);
    }

    #[test]
    fn multiple_values_preserve_order() {
        let input = "---\nalpha: 1\nbeta: 2\ngamma: 3\n---\nbody";
        let result = parse_frontmatter(input);
        let fm = result.frontmatter.expect("should have frontmatter");
        let keys: Vec<&String> = fm.keys().collect();
        assert_eq!(keys, vec!["alpha", "beta", "gamma"]);
    }
}
