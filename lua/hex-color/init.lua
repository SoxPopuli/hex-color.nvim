local lib = require("hex_color_rs")

local M = {}

local autocmd_id = nil

function M.enable()
	autocmd_id = vim.api.nvim_create_autocmd({ "BufEnter", "BufRead", "TextChanged", "InsertLeave" }, {
		pattern = "*",
		callback = function(ev)
			lib.clear_highlights()
			lib.highlight_hex_strings(ev.buf)
		end,
	})

	lib.highlight_hex_strings()
end

function M.disable()
	if autocmd_id == nil then
		return
	end

	vim.api.nvim_del_autocmd(autocmd_id)
	autocmd_id = nil
	local buf = vim.api.nvim_get_current_buf()
	lib.clear_highlights(buf)
end

return M
