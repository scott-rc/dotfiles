use std::collections::HashMap;
use std::io::Write;

use tempfile::TempDir;

fn make_dir_with_files(files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    for (name, content) in files {
        let path = dir.path().join(name);
        let mut f = std::fs::File::create(&path).expect("failed to create file");
        f.write_all(content.as_bytes())
            .expect("failed to write file");
    }
    dir
}

#[test]
fn plain_yaml_passes_through() {
    let yaml = "apiVersion: v1\nkind: Service\n";
    let dir = make_dir_with_files(&[("svc.yml", yaml)]);
    let templates = boom::render::load_templates(dir.path());
    let output = boom::render::render_templates(&templates, &HashMap::new()).unwrap();
    assert_eq!(output.trim(), yaml.trim());
}

#[test]
fn j2_file_renders_variable() {
    let dir = make_dir_with_files(&[("svc.yml.j2", "value: {{ greeting }}\n")]);
    let mut bindings = HashMap::new();
    bindings.insert("greeting".to_string(), "hello".to_string());
    let templates = boom::render::load_templates(dir.path());
    let output = boom::render::render_templates(&templates, &bindings).unwrap();
    assert!(
        output.contains("value: hello"),
        "expected 'value: hello' in output: {output}"
    );
}

#[test]
fn if_block_renders() {
    let dir = make_dir_with_files(&[("cfg.yml.j2", "{% if enabled %}flag: true{% endif %}\n")]);
    let mut bindings = HashMap::new();
    bindings.insert("enabled".to_string(), "true".to_string());
    let templates = boom::render::load_templates(dir.path());
    let output = boom::render::render_templates(&templates, &bindings).unwrap();
    assert!(
        output.contains("flag: true"),
        "expected 'flag: true' in output: {output}"
    );
}

#[test]
fn missing_variable_is_error() {
    let dir = make_dir_with_files(&[("bad.yml.j2", "value: {{ undefined_var }}\n")]);
    let templates = boom::render::load_templates(dir.path());
    let result = boom::render::render_templates(&templates, &HashMap::new());
    assert!(result.is_err(), "expected error for missing variable");
}

#[test]
fn multiple_files_all_rendered() {
    let dir = make_dir_with_files(&[
        ("a.yml", "kind: A\n"),
        ("b.yml.j2", "kind: {{ bkind }}\n"),
    ]);
    let mut bindings = HashMap::new();
    bindings.insert("bkind".to_string(), "B".to_string());
    let templates = boom::render::load_templates(dir.path());
    let output = boom::render::render_templates(&templates, &bindings).unwrap();
    assert!(
        output.contains("kind: A"),
        "expected 'kind: A' in output: {output}"
    );
    assert!(
        output.contains("kind: B"),
        "expected 'kind: B' in output: {output}"
    );
}

#[test]
fn parse_bindings_splits_on_first_equals() {
    let pairs = vec!["key=value".to_string(), "a=b=c".to_string()];
    let map = boom::render::parse_bindings(&pairs);
    assert_eq!(map.get("key").map(String::as_str), Some("value"));
    assert_eq!(map.get("a").map(String::as_str), Some("b=c"));
}

#[test]
fn load_bindings_file_reads_yaml() {
    let dir = make_dir_with_files(&[("bindings.yml", "greeting: hello\n")]);
    let map = boom::render::load_bindings_file(&dir.path().join("bindings.yml"));
    assert_eq!(map.get("greeting").map(String::as_str), Some("hello"));
}
