return {
	"lewis6991/gitsigns.nvim",
	opts = {
		current_line_blame = true,
		current_line_blame_opts = { delay = 250 },
		on_attach = function(bufnr)
			local gs = require("gitsigns")

			local function map(mode, l, r, desc)
				vim.keymap.set(mode, l, r, { buffer = bufnr, desc = desc })
			end

			-- Hunk navigation
			map("n", "]c", function()
				gs.nav_hunk("next")
			end, "Next hunk")
			map("n", "[c", function()
				gs.nav_hunk("prev")
			end, "Prev hunk")
			map("n", "]C", function()
				gs.nav_hunk("last")
			end, "Last hunk")
			map("n", "[C", function()
				gs.nav_hunk("first")
			end, "First hunk")

			-- Stage/unstage
			map("n", "<leader>gs", gs.stage_hunk, "Stage hunk")
			map("v", "<leader>gs", function()
				gs.stage_hunk({ vim.fn.line("."), vim.fn.line("v") })
			end, "Stage selected lines")
			map("n", "<leader>gu", gs.undo_stage_hunk, "Undo stage hunk")
			map("n", "<leader>gS", gs.stage_buffer, "Stage buffer")
			map("n", "<leader>gr", gs.reset_hunk, "Reset hunk")
			map("v", "<leader>gr", function()
				gs.reset_hunk({ vim.fn.line("."), vim.fn.line("v") })
			end, "Reset selected lines")
			map("n", "<leader>gR", gs.reset_buffer, "Reset buffer")

			-- Preview and blame
			map("n", "<leader>gp", gs.preview_hunk_inline, "Preview hunk inline")
			map("n", "<leader>gb", gs.blame_line, "Blame line")
			map("n", "<leader>gB", gs.toggle_current_line_blame, "Toggle line blame")

			-- Hunk text object
			map({ "o", "x" }, "ih", gs.select_hunk, "Select hunk")

			-- Copy hunk to clipboard
			map("n", "<leader>yh", function()
				local hunks = gs.get_hunks(bufnr)
				if not hunks then
					return
				end
				local lnum = vim.fn.line(".")
				for _, h in ipairs(hunks) do
					local s = h.added.start
					local e = s + math.max(h.added.count, 1) - 1
					if lnum >= s and lnum <= e then
						local clean = vim.tbl_map(function(l)
							return l:sub(2)
						end, h.lines)
						vim.fn.setreg("+", table.concat(clean, "\n"))
						vim.notify("Copied hunk (" .. #h.lines .. " lines)")
						return
					end
				end
				vim.notify("No hunk at cursor", vim.log.levels.WARN)
			end, "Copy hunk")
			map("v", "<leader>yh", '"+y', "Copy selection")
		end,
	},
}
