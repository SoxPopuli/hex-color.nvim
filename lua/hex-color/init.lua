local ffi = require("ffi")
local lib_path = vim.api.nvim_get_runtime_file("hex_color_rs.so", false)[1]
local lib = ffi.load(lib_path)

ffi.cdef([[
typedef struct BufferColor BufferColor;

typedef struct BufferColors BufferColors;

typedef struct RgbString {
  char s[6];
} RgbString;

uintptr_t buffer_color_row_index(const struct BufferColor *b);

uintptr_t buffer_color_col_start(const struct BufferColor *b);

uintptr_t buffer_color_col_end(const struct BufferColor *b);

struct RgbString buffer_color_hex_string(const struct BufferColor *b);

struct RgbString buffer_color_foreground_hex(const struct BufferColor *b);

struct BufferColors *buffer_colors_init(void);

bool buffer_colors_has_value(const struct BufferColors *b);

const char *buffer_colors_error_msg(const struct BufferColors *b);

struct BufferColor *buffer_colors_value(struct BufferColors *b, uintptr_t idx);

uintptr_t buffer_colors_value_len(const struct BufferColors *b);

void buffer_colors_add_line(struct BufferColors *b,
                            uintptr_t row_index,
                            const char *line,
                            uintptr_t line_len);

void buffer_colors_free(struct BufferColors *b);
]])

local M = {}

local namespace = vim.api.nvim_create_namespace("hex-color.nvim")
local hl_groups = {}

local autocmd_id = nil

local function rgb_string(rgb)
	return ffi.string(rgb.s, 6)
end

---@param buf integer | nil
function M.highlight_hex_strings(buf)
	local bufnr = buf or 0

	local colors = lib.buffer_colors_init()

	local text = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

	for row, line in ipairs(text) do
		lib.buffer_colors_add_line(colors, row - 1, line, #line)
	end

	for i = 0, tonumber(lib.buffer_colors_value_len(colors)) - 1 do
		local b = lib.buffer_colors_value(colors, i)

		local bg = rgb_string(lib.buffer_color_hex_string(b))
		local hl_group = "HexColor_" .. bg
		if hl_groups[hl_group] == nil then
			local fg = rgb_string(lib.buffer_color_foreground_hex(b))

			vim.api.nvim_set_hl(0, hl_group, {
				bg = "#" .. bg,
				fg = "#" .. fg,
			})
            hl_groups[hl_group] = true
		end

		local row = tonumber(lib.buffer_color_row_index(b)) or 0
		local col_start = tonumber(lib.buffer_color_col_start(b)) or 0
		local col_end = tonumber(lib.buffer_color_col_end(b)) or 0

		vim.api.nvim_buf_set_extmark(bufnr, namespace, row, col_start, {
			end_col = col_end,
			hl_group = hl_group,
		})
	end

	lib.buffer_colors_free(colors)
end

---@type fun(bufnr: integer|nil): nil
function M.clear_highlights(bufnr)
	local buf = bufnr or vim.api.nvim_get_current_buf()
	vim.api.nvim_buf_clear_namespace(buf, namespace, 0, -1)
end

function M.enable()
	autocmd_id = vim.api.nvim_create_autocmd({ "BufEnter", "BufRead", "TextChanged", "InsertLeave" }, {
		pattern = "*",
		callback = function(ev)
			M.clear_highlights(ev.buf)
			M.highlight_hex_strings(ev.buf)
		end,
	})

	M.highlight_hex_strings()
end

function M.disable()
	if autocmd_id == nil then
		return
	end

	vim.api.nvim_del_autocmd(autocmd_id)
	autocmd_id = nil

	M.clear_highlights()
end

vim.api.nvim_create_user_command("HexColorEnable", function()
	M.enable()
end, {})
vim.api.nvim_create_user_command("HexColorDisable", function()
	M.disable()
end, {})
vim.api.nvim_create_user_command("HexColorRefresh", function()
	if autocmd_id ~= nil then
		M.clear_highlights()
		M.highlight_hex_strings()
	end
end, {})

return M
