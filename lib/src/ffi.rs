use crate::BufferColor;
use std::{
    ffi::{CString, c_char},
    ptr::NonNull,
};

#[unsafe(no_mangle)]
extern "C" fn buffer_color_row_index(b: Option<&BufferColor>) -> usize {
    b.map_or(0, |x| x.row_index)
}

#[unsafe(no_mangle)]
extern "C" fn buffer_color_col_start(b: Option<&BufferColor>) -> usize {
    b.map_or(0, |x| x.col_span.start)
}

#[unsafe(no_mangle)]
extern "C" fn buffer_color_col_end(b: Option<&BufferColor>) -> usize {
    b.map_or(0, |x| x.col_span.end)
}

#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq)]
struct RgbString {
    s: [c_char; 6],
}
impl From<crate::color::RgbString> for RgbString {
    #[expect(clippy::missing_transmute_annotations)]
    fn from(value: crate::color::RgbString) -> Self {
        Self {
            s: unsafe { std::mem::transmute(value.0) },
        }
    }
}

#[unsafe(no_mangle)]
extern "C" fn buffer_color_hex_string(b: Option<&BufferColor>) -> RgbString {
    match b {
        Some(b) => b.color.to_hex_string().into(),
        None => RgbString::default(),
    }
}

#[unsafe(no_mangle)]
extern "C" fn buffer_color_foreground_hex(b: Option<&BufferColor>) -> RgbString {
    match b {
        Some(b) => b.color.get_foreground_color().to_hex_string().into(),
        None => RgbString::default(),
    }
}

#[derive(Debug)]
pub struct BufferColors(Result<Vec<BufferColor>, CString>);

#[unsafe(no_mangle)]
extern "C" fn buffer_colors_init() -> *mut BufferColors {
    let b = BufferColors(Ok(vec![]));
    Box::leak(Box::new(b))
}

#[unsafe(no_mangle)]
extern "C" fn buffer_colors_has_value(b: Option<&BufferColors>) -> bool {
    match b {
        Some(b) => b.0.is_ok(),
        None => false,
    }
}

#[unsafe(no_mangle)]
extern "C" fn buffer_colors_error_msg(b: Option<&BufferColors>) -> *const c_char {
    match b {
        Some(BufferColors(Err(e))) => e.as_ptr(),
        _ => std::ptr::null(),
    }
}

#[unsafe(no_mangle)]
extern "C" fn buffer_colors_value(
    b: Option<&mut BufferColors>,
    idx: usize,
) -> Option<&mut BufferColor> {
    match b {
        Some(BufferColors(Ok(v))) => v.get_mut(idx),
        _ => None,
    }
}

#[unsafe(no_mangle)]
extern "C" fn buffer_colors_value_len(b: Option<&BufferColors>) -> usize {
    match b {
        Some(BufferColors(Ok(b))) => b.len(),
        _ => 0,
    }
}

fn add_buffer_colors(b: Option<&mut BufferColors>, text: &str, row_index: usize) {
    let b = match b {
        Some(b) => b,
        None => return,
    };

    let colors = crate::color::parse_string(text);

    let v = match b {
        BufferColors(Ok(v)) => v,
        _ => return,
    };

    for c in colors {
        match c {
            Ok(c) => {
                v.push(BufferColor {
                    row_index,
                    col_span: c.span,
                    color: c.content.to_rgb(),
                });
            }
            Err(e) => {
                let msg = e.to_string();
                b.0 = Err(CString::new(msg).expect("error message has null byte"));
                return;
            }
        }
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn buffer_colors_add_line(
    b: Option<&mut BufferColors>,
    row_index: usize,
    line: *const c_char,
    line_len: usize,
) {
    let line =
        unsafe { String::from_utf8_lossy(std::slice::from_raw_parts(line.cast(), line_len)) };

    add_buffer_colors(b, &line, row_index);
}

#[unsafe(no_mangle)]
extern "C" fn buffer_colors_free(b: Option<NonNull<BufferColors>>) {
    if let Some(b) = b {
        drop(unsafe { Box::from_raw(b.as_ptr()) });
    }
}
