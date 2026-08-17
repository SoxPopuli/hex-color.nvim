mod color;
mod ffi;

use crate::color::Rgb;
use std::range::Range;

#[derive(Debug)]
pub struct BufferColor {
    row_index: usize,
    col_span: Range<usize>,
    color: Rgb<u8>,
}
