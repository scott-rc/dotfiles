use nvim_rs::Neovim;
use rmpv::Value;

use crate::nvim::bridge::Writer;

/// Set up diff highlight groups in nvim.
pub async fn setup(nvim: &Neovim<Writer>) {
    let lua = r##"
        local groups = {
            GdAdded       = { bg = "#1a2e1a" },
            GdDeleted     = { bg = "#2e1a1a" },
            GdAddedWord   = { bg = "#2a4a2a" },
            GdDeletedWord = { bg = "#4a2a2a" },
            GdGutterNum   = { fg = "#888888" },
            GdGutterSep   = { fg = "#555555" },
        }
        for name, attrs in pairs(groups) do
            vim.api.nvim_set_hl(0, name, attrs)
        end
    "##;
    nvim.call(
        "nvim_exec_lua",
        vec![Value::from(lua), Value::Array(vec![])],
    )
    .await
    .ok();
}
