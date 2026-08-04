use const_format::formatc;
use regex_automata::{meta::Regex, util::captures::Captures};
use std::sync::LazyLock;
use strum::{EnumCount, EnumIter, VariantArray};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
#[error("invalid hex char: {0}")]
struct InvalidHexChar(char);

fn hex_char_to_int(c: char) -> Result<u8, InvalidHexChar> {
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

fn hex_pair_to_int(a: char, b: char) -> Result<u8, InvalidHexChar> {
    let a = hex_char_to_int(a)? * 16;
    let b = hex_char_to_int(b)?;

    Ok(a + b)
}

fn parse_hex_string(s: &str) -> impl Iterator<Item = Result<u8, InvalidHexChar>> {
    let even = s.chars().step_by(2);
    let odd = s.chars().skip(1).step_by(2);

    even.zip(odd).map(|(a, b)| hex_pair_to_int(a, b))
}

#[derive(Debug, Default, PartialEq, Eq)]
struct HexColor {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Debug, Default, PartialEq)]
struct Hsl {
    hue: f32,
    sat: f32,
    lum: f32,
}

#[derive(Debug, Default, PartialEq)]
struct Hsv {
    hue: f32,
    sat: f32,
    val: f32,
}

#[derive(Debug, PartialEq)]
enum ParseOutput {
    HexColor(HexColor),
    Hsl(Hsl),
    Hsv(Hsv),
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("{kind:?} error for pattern {pattern:?}")]
struct ParseError {
    pattern: Option<Pattern>,
    kind: ParseErrorKind,
}

#[derive(Debug, Error, PartialEq, Eq)]
enum ParseErrorKind {
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
enum Pattern {
    /// e.g. Hex rgb or rgba
    HexColor,
    /// hsl / hsla / hsv / hsva
    Hslv,
}
impl Pattern {
    fn to_regex_str(&self) -> &'static str {
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

    fn parse(&self, input: &str, capture: &Captures) -> Result<ParseOutput, ParseError> {
        match self {
            Pattern::HexColor => parse_hex_color(input, capture),
            Pattern::Hslv => parse_hslv(input, capture),
        }
    }

    fn parse_pattern_capture(input: &str, capture: &Captures) -> Result<ParseOutput, ParseError> {
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

        pattern.parse(input, capture)
    }
}

fn parse_hslv(input: &str, capture: &Captures) -> Result<ParseOutput, ParseError> {
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
        ColorKind::Value => ParseOutput::Hsv(Hsv {
            hue: hue_value,
            sat,
            val: lv,
        }),
        ColorKind::Luminance => ParseOutput::Hsl(Hsl {
            hue: hue_value,
            sat,
            lum: lv,
        }),
    })
}

fn parse_hex_color(input: &str, capture: &Captures) -> Result<ParseOutput, ParseError> {
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

    res.map(ParseOutput::HexColor)
}

static REGEX: LazyLock<Regex> = LazyLock::new(|| {
    let patterns: [&str; Pattern::COUNT] =
        core::array::from_fn(|i| Pattern::VARIANTS[i].to_regex_str());

    Regex::new_many(&patterns).expect("Failed to compile regex")
});

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
            .map(|x| Pattern::parse_pattern_capture(&input, &x))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let hsl = |hue, sat, lum| ParseOutput::Hsl(Hsl { hue, sat, lum });

        let hsv = |hue, sat, val| ParseOutput::Hsv(Hsv { hue, sat, val });

        assert_eq!(
            captures,
            vec![
                hsl(360.0, 100.0, 50.0),
                hsv(1.0f32.to_degrees(), 100.0, 50.0),
                hsl(270.0, 100.0, 50.0),
                hsl(225.0, 100.0, 50.0),
                hsl(180.0, 100.0, 50.0),
                hsl(135.0, 100.0, 50.0),
                hsl(90.0, 100.0, 50.0),
                hsl(45.0, 100.0, 50.0),
                hsl(0.0, 100.0, 50.0),
            ]
        );
    }
}
