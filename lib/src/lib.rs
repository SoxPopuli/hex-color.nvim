mod color;

use crate::color::Rgb;
use arrayvec::ArrayString;
use nvim_oxi::{
    Dictionary, Function, Result as NvimResult,
    api::{
        Buffer,
        opts::{SetExtmarkOpts, SetHighlightOpts},
    },
};
use std::{range::Range, sync::LazyLock};

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

fn unwrap_or_current(bufnr: Option<i32>) -> Buffer {
    bufnr
        .map(Buffer::from)
        .unwrap_or_else(nvim_oxi::api::get_current_buf)
}

fn hex_hash_prefixed(x: &Rgb<u8>) -> ArrayString<7> {
    let mut output = ArrayString::new_const();

    let suffix = x.to_hex_string();

    output.push('#');
    output.push_str(&suffix);

    output
}

fn highlight_hex_strings(bufnr: Option<i32>) -> NvimResult<()> {
    let mut buf = unwrap_or_current(bufnr);

    let colors = get_buffer_colors(&buf)?;

    for c in colors {
        let hex_color = c.color.to_hex_string();
        let hl_group = format!("HexColor_{hex_color}");

        let bg = hex_hash_prefixed(&c.color);
        let fg = hex_hash_prefixed(&c.color.get_foreground_color());

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

fn clear_highlights(bufnr: Option<i32>) -> NvimResult<()> {
    let mut buf = unwrap_or_current(bufnr);
    buf.clear_namespace(*NVIM_NAMESPACE, ..)?;
    Ok(())
}

#[nvim_oxi::plugin]
fn hex_color_rs() -> Dictionary {
    let highlight_hex_strings: Function<Option<i32>, NvimResult<()>> =
        Function::from_fn(highlight_hex_strings);
    let clear_highlights: Function<Option<i32>, NvimResult<()>> =
        Function::from_fn(clear_highlights);

    Dictionary::from_iter([
        ("highlight_hex_strings", highlight_hex_strings),
        ("clear_highlights", clear_highlights),
    ])
}
