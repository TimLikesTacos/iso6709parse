use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::{is_not, take_while};
use nom::character::complete::{alpha1, digit1};
use nom::combinator::map_res;
use nom::combinator::value;
use nom::sequence::{pair, preceded};
use nom::AsChar;
use nom::IResult;
use nom::Parser;

pub mod human_readable {
    use super::*;
    //     50°40′46.461″N 95°48′26.533″W 123.45m
    //     50°03′46.461″S 125°48′26.533″E 978.90m
    fn parse_sign(inp: &str) -> IResult<&str, f64> {
        let negative: IResult<&str, &str> = tag("-")(inp);
        match negative {
            Ok((rem, _)) => Ok((rem, -1.)),
            Err(_) => Ok((inp, 1.)),
        }
    }

    fn is_part_of_float(ch: char) -> bool {
        ch.is_ascii() && (AsChar::is_dec_digit(ch as u8) || ch == '.')
    }
    fn altitude_decimal(inp: &str) -> IResult<&str, f64> {
        map_res(take_while(is_part_of_float), |x: &str| x.parse::<f64>()).parse(inp)
    }

    pub fn altitude_parser(inp: &str) -> IResult<&str, f64> {
        let (rem, mag) = parse_sign(inp)?;
        let (rem, alt) = altitude_decimal(rem)?;
        Ok((rem, alt * mag))
    }

    #[allow(dead_code)]
    /// Follows only after using altitude_parser
    pub fn altitude_unit(inp: &str) -> IResult<&str, &str> {
        alpha1(inp)
    }

    #[cfg(test)]
    mod altitude_test {
        use super::*;

        #[test]
        fn should_parse_alt() {
            let inp = "978.90m";
            assert_eq!(altitude_parser(inp), Ok(("m", 978.9)));
            let inp = "-978.90m";
            assert_eq!(altitude_parser(inp), Ok(("m", -978.9)));

            assert_eq!(altitude_unit("m"), Ok(("", "m")));
        }

        #[test]
        fn should_err_alt() {
            let inp = "a978.90m";
            assert!(altitude_parser(inp).is_err());
        }
    }
}
pub mod string_expression {
    use super::*;

    fn parse_positive(inp: &str) -> IResult<&str, f64> {
        value(1., tag("+")).parse(inp)
    }

    fn parse_negative(inp: &str) -> IResult<&str, f64> {
        value(-1., tag("-")).parse(inp)
    }

    fn parse_sign(inp: &str) -> IResult<&str, f64> {
        alt((parse_positive, parse_negative)).parse(inp)
    }

    // Unsure if decimals are allowed, so we will support both
    fn altitude(inp: &str) -> IResult<&str, f64> {
        // Order matters
        alt((altitude_decimal, altitude_int)).parse(inp)
    }

    fn is_part_of_float(ch: char) -> bool {
        ch.is_ascii() && (AsChar::is_dec_digit(ch as u8) || ch == '.')
    }
    fn altitude_decimal(inp: &str) -> IResult<&str, f64> {
        map_res(take_while(is_part_of_float), |x: &str| x.parse::<f64>()).parse(inp)
    }
    fn altitude_int(inp: &str) -> IResult<&str, f64> {
        map_res(digit1, |x: &str| x.parse::<f64>()).parse(inp)
    }

    fn parse_altitude_digits(inp: &str) -> IResult<&str, f64> {
        let (rem, (sign, altitude)) = (parse_sign, altitude).parse(inp)?;
        Ok((rem, sign * altitude))
    }

    /// Parses the string that contains altitude AND the crs.
    /// +2122CRSWGS_85
    /// Only returns the altitude in f64
    pub(crate) fn altitude_parser(altitude_with_crs: &str) -> IResult<&str, f64> {
        let (reference_system, (alt, _)) =
            pair(parse_altitude_digits, tag("CRS")).parse(altitude_with_crs)?;
        Ok((reference_system, alt))
    }

    #[allow(dead_code)]
    /// Parses the string that contains altitude AND the crs.
    /// +2122CRSWGS_85
    /// Only returns the CRS (Coordinate Reference System)
    pub(crate) fn crs_parser(altitude_with_crs: &str) -> IResult<&str, &str> {
        preceded(altitude_parser, is_not("/")).parse(altitude_with_crs)
    }

    #[cfg(test)]
    mod altitude_test {
        use super::*;

        #[test]
        fn should_parse_altitude() {
            let inp = "+2122CRSWGS_85/";
            assert_eq!(altitude_parser(inp), Ok(("WGS_85/", 2122.)));
            let inp = "+2122.4CRSWGS_85/";
            assert_eq!(altitude_parser(inp), Ok(("WGS_85/", 2122.4)));
        }

        #[test]
        fn should_err_altitude() {
            let inp = "+2122";
            assert!(altitude_parser(inp).is_err());
            let inp = "2122CRSWGS_85/";
            assert!(altitude_parser(inp).is_err());
        }

        #[test]
        fn should_parse_crs() {
            let inp = "+2122CRSWGS_85/";
            assert_eq!(crs_parser(inp), Ok(("/", "WGS_85")));
        }

        #[test]
        fn should_err_crs() {
            let inp = "+2122CRS";
            assert!(crs_parser(inp).is_err());
        }
    }
}
