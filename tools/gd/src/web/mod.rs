mod html_render;
mod protocol;
mod server;

use crate::git::diff::DiffFile;
use crate::pager::DiffContext;

pub fn run_web_server(files: Vec<DiffFile>, diff_ctx: &DiffContext) {
    server::start_server(files, diff_ctx);
}
