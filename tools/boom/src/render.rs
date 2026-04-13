use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, process};

use minijinja::{Environment, UndefinedBehavior};

#[derive(Debug)]
pub struct TemplateFile {
    pub path: PathBuf,
    pub content: String,
    pub is_template: bool,
}

pub fn load_templates(dir: &Path) -> Vec<TemplateFile> {
    let mut templates: Vec<TemplateFile> = fs::read_dir(dir)
        .unwrap_or_else(|e| {
            eprintln!("boom: cannot read directory {}: {e}", dir.display());
            process::exit(1);
        })
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if !path.is_file() {
                return None;
            }
            let name = path.file_name()?.to_str()?;
            let is_template = name.ends_with(".yml.j2") || name.ends_with(".yaml.j2");
            let is_plain = name.ends_with(".yml") || name.ends_with(".yaml");
            if !is_template && !is_plain {
                return None;
            }
            let content = fs::read_to_string(&path).ok()?;
            Some(TemplateFile {
                path,
                content,
                is_template,
            })
        })
        .collect();

    templates.sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name()));
    templates
}

pub fn render_templates(
    templates: &[TemplateFile],
    bindings: &HashMap<String, String>,
) -> Result<String, String> {
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);

    let ctx = minijinja::Value::from_serialize(bindings);
    let mut documents = Vec::new();

    for tmpl in templates {
        if tmpl.is_template {
            let name = tmpl.path.file_name().unwrap_or_default().to_string_lossy();
            let rendered = env
                .render_str(&tmpl.content, &ctx)
                .map_err(|e| format!("boom: render error in {name}: {e}"))?;
            documents.push(rendered);
        } else {
            documents.push(tmpl.content.clone());
        }
    }

    Ok(documents.join("---\n"))
}

pub fn parse_bindings(pairs: &[String]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in pairs {
        if let Some((key, value)) = pair.split_once('=') {
            map.insert(key.to_string(), value.to_string());
        } else {
            eprintln!("boom: malformed binding (missing '='): {pair}");
        }
    }
    map
}

pub fn load_bindings_file(path: &Path) -> HashMap<String, String> {
    let content = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("boom: cannot read bindings file {}: {e}", path.display());
        process::exit(1);
    });
    let value: serde_yaml::Value = serde_yaml::from_str(&content).unwrap_or_else(|e| {
        eprintln!("boom: cannot parse bindings file {}: {e}", path.display());
        process::exit(1);
    });
    let mut map = HashMap::new();
    if let serde_yaml::Value::Mapping(mapping) = value {
        for (k, v) in mapping {
            if let (serde_yaml::Value::String(key), serde_yaml::Value::String(val)) = (k, v) {
                map.insert(key, val);
            }
        }
    }
    map
}
