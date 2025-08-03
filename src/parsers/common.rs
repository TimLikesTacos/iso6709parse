use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::combinator::{map_res, opt, recognize};
use nom::sequence::terminated;
use nom::IResult;
use nom::Parser;

#[cfg(test)]
pub(crate) fn assert_float_approx<E: std::fmt::Debug>(
    actual: IResult<&str, f64, E>,
    expected: f64,
) {
    let actual = actual.unwrap();
    assert!(
        (actual.1 - expected).abs() < 0.0001f64,
        "expected: {}, actual: {}",
        expected,
        actual.1
    )
}

pub(crate) mod human_readable {
    use super::*;
    pub(crate) fn parse_value(inp: &str) -> IResult<&str, f64> {
        map_res(digit1, |x: &str| x.parse::<f64>()).parse(inp)
    }

    pub(crate) fn parse_degree(inp: &str) -> IResult<&str, f64> {
        terminated(parse_value, tag("°")).parse(inp)
    }

    pub(crate) fn parse_minutes(inp: &str) -> IResult<&str, f64> {
        terminated(parse_value, alt((tag("'"), tag("′")))).parse(inp)
    }

    pub(crate) fn parse_seconds_with_decimal(inp: &str) -> IResult<&str, f64> {
        map_res(recognize((digit1, opt((tag("."), digit1)))), |x: &str| {
            x.parse::<f64>()
        })
        .parse(inp)
    }

    pub(crate) fn parse_seconds(inp: &str) -> IResult<&str, f64> {
        terminated(parse_seconds_with_decimal, alt((tag("\""), tag("″")))).parse(inp)
    }
}
