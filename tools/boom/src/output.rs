use colored::Colorize;

pub fn info(msg: &str) {
    println!("{}", msg.cyan());
}

pub fn success(msg: &str) {
    println!("{}", msg.green());
}

pub fn warn(msg: &str) {
    println!("{}", msg.yellow());
}

pub fn error(msg: &str) {
    eprintln!("{}", msg.red());
}

pub fn summary_table(rows: &[(String, String, String, String)]) {
    println!(
        "{:<40} {:<30} {:<12} Duration",
        "Resource", "Kind", "Status"
    );
    println!("{}", "-".repeat(90));
    for (resource, kind, status, duration) in rows {
        let line = format!("{resource:<40} {kind:<30} {status:<12} {duration}");
        match status.as_str() {
            "OK" | "Ready" => println!("{}", line.green()),
            "Failed" | "Error" => println!("{}", line.red()),
            "Timeout" => println!("{}", line.yellow()),
            _ => println!("{line}"),
        }
    }
}
