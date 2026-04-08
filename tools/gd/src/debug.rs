use std::io::Write;
use std::time::Instant;

pub(crate) fn enabled() -> bool {
    std::env::var_os("GD_DEBUG").is_some_and(|v| v == "1")
}

/// Emit a structured timing trace to stderr (only when `GD_DEBUG=1`).
/// `t0` is the process-wide start instant; elapsed ms are computed from it.
pub(crate) fn trace(location: &str, message: &str, t0: Instant) {
    if !enabled() {
        return;
    }
    let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let line = format!("[gd:timing] {location}: {message} ({elapsed_ms:.1}ms)\n");
    let _ = std::io::stderr().write_all(line.as_bytes());
}
