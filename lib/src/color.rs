use arrayvec::ArrayString;
use const_format::formatc;
use regex_automata::{meta::Regex, util::captures::Captures};
use std::{range::Range, sync::LazyLock};
use strum::{EnumCount, EnumIter, VariantArray};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
#[error("invalid hex char: {0}")]
pub struct InvalidHexChar(char);

pub fn hex_char_to_int(c: char) -> Result<u8, InvalidHexChar> {
    match c.to_ascii_lowercase() {
        '0' => Ok(0),
        '1' => Ok(1),
        '2' => Ok(2),
        '3' => Ok(3),
        '4' => Ok(4),
        '5' => Ok(5),
        '6' => Ok(6),
        '7' => Ok(7),
        '8' => Ok(8),
        '9' => Ok(9),
        'a' => Ok(10),
        'b' => Ok(11),
        'c' => Ok(12),
        'd' => Ok(13),
        'e' => Ok(14),
        'f' => Ok(15),

        x => Err(InvalidHexChar(x)),
    }
}

pub fn hex_pair_to_int(a: char, b: char) -> Result<u8, InvalidHexChar> {
    let a = hex_char_to_int(a)? * 16;
    let b = hex_char_to_int(b)?;

    Ok(a + b)
}

pub fn parse_hex_string(s: &str) -> impl Iterator<Item = Result<u8, InvalidHexChar>> {
    let even = s.chars().step_by(2);
    let odd = s.chars().skip(1).step_by(2);

    even.zip(odd).map(|(a, b)| hex_pair_to_int(a, b))
}

pub fn int_to_hex_char(x: u8) -> [char; 2] {
    let hi = (x & 0xF0) / 16;
    let lo = x & 0x0F;

    fn hex(x: u8) -> Option<char> {
        match x {
            0 => Some('0'),
            1 => Some('1'),
            2 => Some('2'),
            3 => Some('3'),
            4 => Some('4'),
            5 => Some('5'),
            6 => Some('6'),
            7 => Some('7'),
            8 => Some('8'),
            9 => Some('9'),
            10 => Some('a'),
            11 => Some('b'),
            12 => Some('c'),
            13 => Some('d'),
            14 => Some('e'),
            15 => Some('f'),

            _ => None,
        }
    }

    [
        hex(hi).expect("hex char out of range"),
        hex(lo).expect("hex char out of range"),
    ]
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct HexColor {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Debug, Default, PartialEq)]
pub struct Hsl {
    hue: f32,
    sat: f32,
    lum: f32,
}

#[derive(Debug, Default, PartialEq)]
pub struct Hsv {
    hue: f32,
    sat: f32,
    val: f32,
}

pub fn hue_to_rgb_prime(hue: f32, chroma: f32) -> Rgb<f32> {
    let hue_prime = hue / 60.0;

    let c = chroma;
    let x = c * (1.0 - (hue_prime % 2.0 - 1.0).abs());

    match hue_prime.floor() as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x), // sector 5, hue in [300, 360)
    }
    .into()
}

#[derive(Debug, Default, PartialEq)]
pub struct Rgb<T> {
    r: T,
    g: T,
    b: T,
}
impl<T> From<(T, T, T)> for Rgb<T> {
    fn from((r, g, b): (T, T, T)) -> Self {
        Self { r, g, b }
    }
}
impl<T> Rgb<T> {
    #[cfg(test)]
    pub fn new(r: T, g: T, b: T) -> Self {
        Self { r, g, b }
    }
}
impl Rgb<u8> {
    #[rustfmt::skip]
    const WHITE: Self = Self { r: 255, g: 255, b: 255 };
    const BLACK: Self = Self { r: 0, g: 0, b: 0 };

    pub fn get_foreground_color(&self) -> Self {
        let grey = (self.r as f32 * 0.299) + (self.g as f32 * 0.587) + (self.b as f32 * 0.144);
        if grey < 167.0 {
            Self::WHITE
        } else {
            Self::BLACK
        }
    }

    pub fn new_from_prime(rgb: Rgb<f32>, m: f32) -> Self {
        let to_u8 = |x: f32| ((x + m) * 255.0).round() as u8;

        Self {
            r: to_u8(rgb.r),
            g: to_u8(rgb.g),
            b: to_u8(rgb.b),
        }
    }

    /// Doesn't include the '#'
    /// e.g. `0xFF00FF` -> `"ff00ff"`
    pub fn to_hex_string(&self) -> ArrayString<6> {
        let mut output = ArrayString::new_const();

        let mut add_chars = |hex| {
            let [a, b] = int_to_hex_char(hex);
            output.push(a);
            output.push(b);
        };

        add_chars(self.r);
        add_chars(self.g);
        add_chars(self.b);

        output
    }
}

pub fn hsl_to_rgb(hsl: &Hsl) -> Rgb<u8> {
    // normalize values
    let hue = hsl.hue.rem_euclid(360.0);
    let sat = hsl.sat.clamp(0.0, 100.0) / 100.0;
    let lum = hsl.lum.clamp(0.0, 100.0) / 100.0;

    let c = (1.0 - (2.0 * lum - 1.0).abs()) * sat;

    let rgb_prime = hue_to_rgb_prime(hue, c);

    let m = lum - c / 2.0;

    Rgb::new_from_prime(rgb_prime, m)
}

pub fn hsv_to_rgb(hsv: &Hsv) -> Rgb<u8> {
    // normalize values
    let hue = hsv.hue.rem_euclid(360.0);
    let sat = hsv.sat.clamp(0.0, 100.0) / 100.0;
    let val = hsv.val.clamp(0.0, 100.0) / 100.0;

    let c = sat * val;

    let rgb_prime = hue_to_rgb_prime(hue, c);

    let m = val - c;

    Rgb::new_from_prime(rgb_prime, m)
}

#[derive(Debug, PartialEq)]
pub enum ParseOutputContent {
    HexColor(HexColor),
    Hsl(Hsl),
    Hsv(Hsv),
}
impl ParseOutputContent {
    pub fn to_rgb(&self) -> Rgb<u8> {
        match self {
            Self::HexColor(hex) => Rgb {
                r: hex.r,
                g: hex.g,
                b: hex.b,
            },
            Self::Hsl(hsl) => hsl_to_rgb(hsl),
            Self::Hsv(hsv) => hsv_to_rgb(hsv),
        }
    }

    #[cfg(test)]
    pub fn to_hex_string(&self) -> ArrayString<6> {
        self.to_rgb().to_hex_string()
    }
}

#[derive(Debug, PartialEq)]
pub struct ParseOutput {
    pub span: Range<usize>,
    pub content: ParseOutputContent,
}
impl ParseOutput {
    fn from_pattern_capture(input: &str, capture: &Captures) -> Result<Self, ParseError> {
        let error = |kind| ParseError {
            pattern: None,
            kind,
        };

        let pattern_id = capture
            .pattern()
            .map(|p| p.as_usize())
            .ok_or(error(ParseErrorKind::InvalidPatternId))?;

        let pattern = Pattern::VARIANTS
            .get(pattern_id)
            .ok_or(error(ParseErrorKind::InvalidVariantId(pattern_id)))?;

        let match_span = capture.get_group(0).ok_or(ParseError {
            pattern: None,
            kind: ParseErrorKind::MissingCaptureGroup(0),
        })?;

        let content = pattern.parse(input, capture)?;

        Ok(Self {
            span: match_span.range().into(),
            content,
        })
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("{kind:?} error for pattern {pattern:?}")]
pub struct ParseError {
    pattern: Option<Pattern>,
    kind: ParseErrorKind,
}
impl From<ParseError> for nvim_oxi::Error {
    fn from(val: ParseError) -> Self {
        nvim_oxi::Error::Lua(nvim_oxi::lua::Error::RuntimeError(val.to_string()))
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseErrorKind {
    #[error("invalid hex char: {0:?}")]
    InvalidHexChar(InvalidHexChar),
    #[error("missing capture group: {0}")]
    MissingCaptureGroup(usize),
    #[error("capture range outside of input text: {0}-{1}")]
    InvalidCaptureRange(usize, usize),
    #[error("hex string too short")]
    HexStringTooShort,
    #[error("pattern id does not map to a pattern")]
    InvalidPatternId,
    #[error("unexpected pattern variant index: {0}")]
    InvalidVariantId(usize),
    #[error("unable to parse to float: {0:?}")]
    ParseFloat(#[from] std::num::ParseFloatError),
    #[error("missing required element: {0}")]
    MissingRequiredElement(&'static str),
}

#[derive(Debug, PartialEq, Eq)]
enum ColorKind {
    Value,
    Luminance,
}

#[derive(Debug, PartialEq)]
enum HueUnit {
    Degree,
    Gradian,
    Radian,
    Turn,
}
impl HueUnit {
    fn to_degrees(&self, value: f32) -> f32 {
        match self {
            HueUnit::Degree => value,
            HueUnit::Gradian => value * 0.9,
            HueUnit::Radian => value.to_degrees(),
            HueUnit::Turn => value * 360.0,
        }
    }
}

#[derive(Debug, PartialEq, Eq, EnumIter, VariantArray, EnumCount)]
pub enum Pattern {
    /// e.g. Hex rgb or rgba
    HexColor,
    /// hsl / hsla / hsv / hsva
    Hslv,
}
impl Pattern {
    pub fn to_regex_str(&self) -> &'static str {
        match self {
            Self::HexColor => r"#\b([[:xdigit:]]{8}|[[:xdigit:]]{6})\b",
            Self::Hslv => {
                const KIND: &str = r"(?<kind>l|v)";
                const H: &str = r"(?<h>-?\d+(?:\.\d+)?)";
                const H_UNIT: &str = r"(?<hunit>deg|g?rad|turn)";
                const S: &str = r"(?<s>\d+(?:\.\d+)?)";
                const LV: &str = r"(?<lv>\d+(?:\.\d+)?)";
                const A: &str = r"(?<a>\d*\.?\d+%?)";
                formatc!(
                    r"(?i)\bhs{KIND}a?\(\s*{H}{H_UNIT}?(?:\s*,\s*|\s+){S}%(?:\s*,\s*|\s+){LV}%(?:\s*(?:,|/)\s*{A})?\s*\)"
                )
            }
        }
    }

    fn parse(&self, input: &str, capture: &Captures) -> Result<ParseOutputContent, ParseError> {
        match self {
            Pattern::HexColor => parse_hex_color(input, capture),
            Pattern::Hslv => parse_hslv(input, capture),
        }
    }
}

fn parse_hslv(input: &str, capture: &Captures) -> Result<ParseOutputContent, ParseError> {
    let get_input_by_name = |name| {
        let cap = capture.get_group_by_name(name);

        cap.and_then(|span| input.get(span.range()))
    };
    let error = |kind| ParseError {
        kind,
        pattern: Some(Pattern::Hslv),
    };

    let parse_f32 = |x: &str| {
        x.parse::<f32>()
            .map_err(|e| error(ParseErrorKind::ParseFloat(e)))
    };

    let kind = match get_input_by_name("kind") {
        Some("l" | "L") => ColorKind::Luminance,
        Some("v" | "V") => ColorKind::Value,
        _ => unreachable!(),
    };
    let hue = get_input_by_name("h")
        .ok_or(error(ParseErrorKind::MissingRequiredElement("hue")))
        .and_then(parse_f32)?;

    let hue_unit = get_input_by_name("hunit").and_then(|unit| {
        if unit.eq_ignore_ascii_case("deg") {
            Some(HueUnit::Degree)
        } else if unit.eq_ignore_ascii_case("rad") {
            Some(HueUnit::Radian)
        } else if unit.eq_ignore_ascii_case("grad") {
            Some(HueUnit::Gradian)
        } else if unit.eq_ignore_ascii_case("turn") {
            Some(HueUnit::Turn)
        } else {
            None
        }
    });
    let sat = get_input_by_name("s")
        .ok_or(error(ParseErrorKind::MissingRequiredElement("sat")))
        .and_then(parse_f32)?;
    let lv = get_input_by_name("lv")
        .ok_or(error(ParseErrorKind::MissingRequiredElement("lv")))
        .and_then(parse_f32)?;
    // don't care about alpha, can't represent in terminal
    // let alpha = get_input_by_name("a");

    let hue_value = match hue_unit {
        Some(hue_unit) => hue_unit.to_degrees(hue),
        None => hue,
    };

    Ok(match kind {
        ColorKind::Value => ParseOutputContent::Hsv(Hsv {
            hue: hue_value,
            sat,
            val: lv,
        }),
        ColorKind::Luminance => ParseOutputContent::Hsl(Hsl {
            hue: hue_value,
            sat,
            lum: lv,
        }),
    })
}

fn parse_hex_color(input: &str, capture: &Captures) -> Result<ParseOutputContent, ParseError> {
    let error = |kind| ParseError {
        pattern: Some(Pattern::HexColor),
        kind,
    };

    let span = capture
        .get_group(1)
        .ok_or(error(ParseErrorKind::MissingCaptureGroup(1)))?;
    let range = span.range();
    let text = input
        .get(range.clone())
        .ok_or(error(ParseErrorKind::InvalidCaptureRange(
            range.start,
            range.end,
        )))?;

    let values = parse_hex_string(text)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| error(ParseErrorKind::InvalidHexChar(e)))?;

    let r = values.first();
    let g = values.get(1);
    let b = values.get(2);
    // let a = values.get(3);

    let res = match (r, g, b) {
        (Some(&r), Some(&g), Some(&b)) => Ok(HexColor { r, g, b }),

        _ => Err(error(ParseErrorKind::HexStringTooShort)),
    };

    res.map(ParseOutputContent::HexColor)
}

pub static REGEX: LazyLock<Regex> = LazyLock::new(|| {
    let patterns: [&str; Pattern::COUNT] =
        core::array::from_fn(|i| Pattern::VARIANTS[i].to_regex_str());

    Regex::new_many(&patterns).expect("Failed to compile regex")
});

pub fn parse_string<'a>(
    input: &'a str,
) -> impl Iterator<Item = Result<ParseOutput, ParseError>> + 'a {
    REGEX
        .captures_iter(input)
        .map(|x| ParseOutput::from_pattern_capture(input, &x))
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("ff" => Ok(vec![0xFF]))]
    #[test_case("ffff" => Ok(vec![0xFF, 0xFF]))]
    #[test_case("ffffff" => Ok(vec![0xFF, 0xFF, 0xFF]))]
    #[test_case("ffffffff" => Ok(vec![0xFF, 0xFF, 0xFF, 0xFF]))]
    #[test_case("CAFEbabe" => Ok(vec![0xCA, 0xFE, 0xBA, 0xBE]))]
    #[test_case("invalidhex" => Err(InvalidHexChar('i')))]
    fn hex_conversion_tests(s: &str) -> Result<Vec<u8>, InvalidHexChar> {
        parse_hex_string(s).collect()
    }

    #[test_case(0xFF => ['f', 'f'])]
    #[test_case(0x00 => ['0', '0'])]
    #[test_case(0xA0 => ['a', '0'])]
    #[test_case(0x0A => ['0', 'a'])]
    #[test_case(0xCE => ['c', 'e'])]
    fn int_to_hex(x: u8) -> [char; 2] {
        int_to_hex_char(x)
    }

    #[test]
    fn regex_test() {
        let inputs = [
            "hsla(360 100% 50%)",
            "hsv(1rad 100% 50%)",
            "hsl(270 100% 50%)",
            "hsl(225 100% 50%)",
            "hsl(180 100% 50%)",
            "hsl(135 100% 50%)",
            "hsl(90 100% 50%)",
            "hsl(45 100% 50%)",
            "hsl(0 100% 50%)",
        ];

        let input = inputs.join("\n");

        let captures = REGEX
            .captures_iter(&input)
            .map(|x| ParseOutput::from_pattern_capture(&input, &x))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let hsl = |span: std::ops::Range<usize>, hue, sat, lum| ParseOutput {
            span: span.into(),
            content: ParseOutputContent::Hsl(Hsl { hue, sat, lum }),
        };
        let hsv = |span: std::ops::Range<usize>, hue, sat, val| ParseOutput {
            span: span.into(),
            content: ParseOutputContent::Hsv(Hsv { hue, sat, val }),
        };

        pretty_assertions::assert_eq!(
            captures,
            vec![
                hsl(0..18, 360.0, 100.0, 50.0),
                hsv(19..37, 1.0f32.to_degrees(), 100.0, 50.0),
                hsl(38..55, 270.0, 100.0, 50.0),
                hsl(56..73, 225.0, 100.0, 50.0),
                hsl(74..91, 180.0, 100.0, 50.0),
                hsl(92..109, 135.0, 100.0, 50.0),
                hsl(110..126, 90.0, 100.0, 50.0),
                hsl(127..143, 45.0, 100.0, 50.0),
                hsl(144..159, 0.0, 100.0, 50.0),
            ]
        );
    }

    #[test]
    fn hex_color_to_string() {
        let hex = HexColor {
            r: 255,
            g: 0,
            b: 127,
        };
        let hex_str = ParseOutputContent::HexColor(hex)
            .to_hex_string()
            .to_string();

        assert_eq!(hex_str, "ff007f");
    }

    #[test_case(270.0, 100.0, 100.0 => Rgb::new(255, 255, 255))]
    #[test_case(270.0, 100.0, 50.0 => Rgb::new(128, 0, 255))]
    #[test_case(180.0, 100.0, 50.0 => Rgb::new(0, 255, 255))]
    #[test_case(180.0, 100.0, 0.0 => Rgb::new(0, 0, 0))]
    #[test_case(180.0, 50.0, 50.0 => Rgb::new(64, 191, 191))]
    fn hsl_to_string(hue: f32, sat: f32, lum: f32) -> Rgb<u8> {
        let hsl = Hsl { hue, sat, lum };
        hsl_to_rgb(&hsl)
    }

    #[test_case(270.0, 100.0, 100.0 => Rgb::new(128, 0, 255))]
    #[test_case(270.0, 100.0, 50.0 => Rgb::new(64, 0, 128))]
    #[test_case(180.0, 100.0, 50.0 => Rgb::new(0, 128, 128))]
    #[test_case(180.0, 100.0, 0.0 => Rgb::new(0, 0, 0))]
    #[test_case(180.0, 50.0, 50.0 => Rgb::new(64, 128, 128))]
    fn hsv_to_string(hue: f32, sat: f32, val: f32) -> Rgb<u8> {
        let hsv = Hsv { hue, sat, val };
        hsv_to_rgb(&hsv)
    }
}
