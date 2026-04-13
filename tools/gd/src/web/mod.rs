mod html_render;
mod protocol;
mod server;

use crate::git::diff::DiffFile;
use crate::pager::DiffContext;

pub fn run_web_server(
    files: Vec<DiffFile>,
    diff_ctx: &DiffContext,
    open: bool,
    shutdown_grace_ms: u64,
) {
    server::start_server(files, diff_ctx, open, shutdown_grace_ms);
}
