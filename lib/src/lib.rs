mod color;
use nvim_oxi::{
    Dictionary, Function, Result as NvimResult,
    api::{
        Buffer,
        opts::{SetExtmarkOpts, SetHighlightOpts},
    },
};
use std::{range::Range, sync::LazyLock};

use crate::color::Rgb;

fn lines_to_string(lines: impl Iterator<Item = nvim_oxi::String>) -> String {
    let mut s = String::new();
    let mut first = true;
    for l in lines {
        if !first {
            s.push('\n');
        }
        s.push_str(&l.to_string_lossy());
        first = false;
    }

    s
}

trait ToLuaError<T>: Sized {
    fn into_lua_error(self) -> NvimResult<T>;
}
impl<T> ToLuaError<T> for Result<T, color::ParseError> {
    fn into_lua_error(self) -> NvimResult<T> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => Err(nvim_oxi::Error::Lua(nvim_oxi::lua::Error::RuntimeError(
                e.to_string(),
            ))),
        }
    }
}

#[derive(Debug)]
pub struct BufferColor {
    row_index: usize,
    col_span: Range<usize>,
    color: Rgb<u8>,
}

fn get_buffer_colors(buf: &Buffer) -> NvimResult<Vec<BufferColor>> {
    let lines = buf.get_lines(.., false)?;

    let mut buf_colors = vec![];

    for (row, line) in lines.enumerate() {
        let line = match line.to_str() {
            Ok(s) => s,
            Err(_) => continue, // skip non utf-8 rows
        };

        let results = color::parse_string(line).collect::<Result<Box<_>, _>>()?;
        for r in results {
            let buf_color = BufferColor {
                row_index: row,
                col_span: r.span,
                color: r.content.to_rgb(),
            };

            buf_colors.push(buf_color);
        }
    }

    Ok(buf_colors)
}

static NVIM_NAMESPACE: LazyLock<u32> =
    LazyLock::new(|| nvim_oxi::api::create_namespace(env!("CARGO_CRATE_NAME")));

fn highlight_hex_strings(bufnr: Option<i32>) -> NvimResult<()> {
    let mut buf = bufnr
        .map(Buffer::from)
        .unwrap_or_else(nvim_oxi::api::get_current_buf);

    let colors = get_buffer_colors(&buf)?;

    for c in colors {
        let hex_color = c.color.to_hex_string();
        let hl_group = format!("HexColor_{hex_color}");

        let bg = format!("#{hex_color}");
        let fg = format!("#{}", c.color.get_foreground_color().to_hex_string());

        let hl_opts = SetHighlightOpts::builder()
            .bg(&bg)
            .fg(&fg)
            .force(true)
            .build();

        nvim_oxi::api::set_hl(0, &hl_group, &hl_opts)?;

        let extmark_opts = SetExtmarkOpts::builder()
            .end_col(c.col_span.end)
            .hl_group(hl_group.as_str())
            .build();

        buf.set_extmark(
            *NVIM_NAMESPACE,
            c.row_index,
            c.col_span.start,
            &extmark_opts,
        )?;
    }

    Ok(())
}

#[nvim_oxi::plugin]
fn hex_color_rs() -> Dictionary {
    let highlight_hex_strings: Function<Option<i32>, NvimResult<()>> =
        Function::from_fn(highlight_hex_strings);

    Dictionary::from_iter([("highlight_hex_strings", highlight_hex_strings)])
}
