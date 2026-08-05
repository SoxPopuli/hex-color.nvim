mod color;

use nvim_oxi::{Dictionary, Function, Result as NvimResult, api::Buffer};

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

fn highlight_hex_strings(bufnr: Option<i32>) -> NvimResult<()> {
    let buf = bufnr
        .map(Buffer::from)
        .unwrap_or_else(nvim_oxi::api::get_current_buf);

    let lines = buf.get_lines(.., false)?;
    let s = lines_to_string(lines);

    nvim_oxi::print!("hello {}", s);

    Ok(())
}

#[nvim_oxi::plugin]
fn hex_color_rs() -> Dictionary {
    let highlight_hex_strings: Function<Option<i32>, NvimResult<()>> =
        Function::from_fn(highlight_hex_strings);

    Dictionary::from_iter([("highlight_hex_strings", highlight_hex_strings)])
}
