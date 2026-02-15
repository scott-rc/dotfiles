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
