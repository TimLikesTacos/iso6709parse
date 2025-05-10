#![allow(dead_code)]
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while_m_n;
use nom::character::complete::{digit0, u8};
use nom::character::is_digit;
use nom::combinator::{map, opt, value};
use nom::combinator::{map_parser, map_res, recognize};
use nom::error::ParseError;
use nom::sequence::tuple;
use nom::IResult;

pub mod human_readable {
    use super::*;
    use crate::parsers::common::human_readable::*;
    //     50°40′46.461″N 95°48′26.533″W 123.45m
    //     50°03′46.461″S 125°48′26.533″E 978.90m

    fn parse_north(inp: &str) -> IResult<&str, f64> {
        value(1., tag("N"))(inp)
    }

    fn parse_south(inp: &str) -> IResult<&str, f64> {
        value(-1., tag("S"))(inp)
    }

    fn parse_north_or_south(inp: &str) -> IResult<&str, f64> {
        alt((parse_north, parse_south))(inp)
    }

    pub fn latitude_parser(inp: &str) -> IResult<&str, f64> {
        let (rem, deg) = parse_degree(inp)?;
        let (rem, min) = parse_minutes(rem)?;
        let (rem, sec) = parse_seconds(rem)?;
        let (rem, mag) = parse_north_or_south(rem)?;
        let value = deg + min / 60. + sec / 3600.;
        if value > 90.0 {
            Err(nom::Err::Failure(nom::error::Error::new(
                inp,
                nom::error::ErrorKind::Fail,
            )))
        } else {
            Ok((rem, mag * value))
        }
    }

    #[cfg(test)]
    mod lat_tests {
        use super::*;
        use crate::parsers::common::assert_float_approx;
        #[test]
        fn should_parse_latitude() {
            let inp = "50°40′46.461″N 95°48′26.533″W 123.45m";
            assert_float_approx(latitude_parser(inp), 50.679573);
            assert_eq!(latitude_parser(inp).unwrap().0, " 95°48′26.533″W 123.45m");
            let inp = "50°40'46.461\"N 95°48′26.533″W 123.45m";
            assert_float_approx(latitude_parser(inp), 50.679573);
            let inp = "00°40′46.461″N 95°48′26.533″W 123.45m";
            assert_float_approx(latitude_parser(inp), 0.679573);

            let inp = "50°40′46.461″S 95°48′26.533″W 123.45m";
            assert_float_approx(latitude_parser(inp), -50.679573);
            let inp = "50°40'46.461\"S 95°48′26.533″W 123.45m";
            assert_float_approx(latitude_parser(inp), -50.679573);
            let inp = "50°40'46\"S 95°48′26.533″W 123.45m";
            assert_float_approx(latitude_parser(inp), -50.679444);
            let inp = "90°00'00.00\"S 95°48′26.533″W 123.45m";
            assert_float_approx(latitude_parser(inp), -90.);
            let inp = "90°00'00.00\"N 95°48′26.533″W 123.45m";
            assert_float_approx(latitude_parser(inp), 90.);
            let inp = "00°00'00.00\"N 95°48′26.533″W 123.45m";
            assert_float_approx(latitude_parser(inp), 0.);
            let inp = "00°00'00.00\"S 95°48′26.533″W 123.45m";
            assert_float_approx(latitude_parser(inp), 0.);
        }

        #[test]
        fn should_err_latitude() {
            let inp = "50.40′46.461″N 95°48′26.533″W 123.45m";
            assert!(latitude_parser(inp).is_err());
            let inp = "50.40′46.461.N 95°48′26.533″W 123.45m";
            assert!(latitude_parser(inp).is_err());
            let inp = "50°40.46.461″N 95°48′26.533″W 123.45m";
            assert!(latitude_parser(inp).is_err());
            let inp = "90°40′46.461″N 95°48′26.533″W 123.45m";
            assert!(latitude_parser(inp).is_err());
        }
    }
}

pub mod string_expression {
    use super::*;

    fn parse_north(inp: &str) -> IResult<&str, f64> {
        value(1., alt((tag("N"), tag("+"))))(inp)
    }

    fn parse_south(inp: &str) -> IResult<&str, f64> {
        value(-1., alt((tag("S"), tag("-"))))(inp)
    }

    fn parse_north_or_south(inp: &str) -> IResult<&str, f64> {
        alt((parse_north, parse_south))(inp)
    }

    fn is_char_digit(char: char) -> bool {
        char.is_ascii() && is_digit(char as u8)
    }

    fn parse_two<'a, F, O, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
    where
        F: Fn(&'a str) -> IResult<&'a str, O, E> + 'a,
        E: ParseError<&'a str>,
    {
        map_parser(take_while_m_n(2, 2, is_char_digit), inner)
    }

    fn parse_degree_integer(inp: &str) -> IResult<&str, f64> {
        map(parse_two(u8), |x| x as f64)(inp)
    }

    fn parse_degree_min_integer(inp: &str) -> IResult<&str, f64> {
        let (rem, (degrees, minutes)) = tuple((parse_two(u8), parse_two(u8)))(inp)?;

        if minutes >= 60 {
            Err(nom::Err::Failure(nom::error::Error::new(
                inp,
                nom::error::ErrorKind::Fail,
            )))
        } else {
            Ok((rem, (degrees as f64) + (minutes as f64 / 60.)))
        }
    }

    fn parse_degree_min_sec_integer(inp: &str) -> IResult<&str, f64> {
        let (rem, (degrees, minutes, seconds)) =
            tuple((parse_two(u8), parse_two(u8), parse_two(u8)))(inp)?;

        if minutes >= 60 || seconds >= 60 {
            Err(nom::Err::Failure(nom::error::Error::new(
                inp,
                nom::error::ErrorKind::Fail,
            )))
        } else {
            Ok((
                rem,
                (degrees as f64) + (minutes as f64 / 60.) + (seconds as f64 / 3600.),
            ))
        }
    }

    fn parse_decimal(inp: &str) -> IResult<&str, f64> {
        map_res(recognize(tuple((tag("."), digit0))), |x: &str| {
            x.parse::<f64>()
        })(inp)
    }

    fn parse_degree(inp: &str) -> IResult<&str, f64> {
        let (decimalstr, int) = parse_degree_integer(inp)?;
        let (rem, dec) = opt(parse_decimal)(decimalstr)?;
        Ok((rem, int + dec.unwrap_or(0.)))
    }

    fn parse_degree_minute(inp: &str) -> IResult<&str, f64> {
        let (decimalstr, int) = parse_degree_min_integer(inp)?;
        let (rem, dec) = opt(parse_decimal)(decimalstr)?;
        Ok((rem, int + dec.unwrap_or(0.) / 60.))
    }

    fn parse_degree_minute_second(inp: &str) -> IResult<&str, f64> {
        let (decimalstr, int) = parse_degree_min_sec_integer(inp)?;
        let (rem, dec) = opt(parse_decimal)(decimalstr)?;
        Ok((rem, int + dec.unwrap_or(0.) / 3600.))
    }

    /// Nom style parser for latitude. The beginning of the string slice must be the start of latitude.
    /// Returns Err if failed to parse, or latitude is greater than +/-90.0
    pub fn latitude_parser(inp: &str) -> IResult<&str, f64> {
        let (lat, mag) = parse_north_or_south(inp)?;
        let (rem, value) = alt((
            parse_degree_minute_second,
            parse_degree_minute,
            parse_degree,
        ))(lat)?;
        if value > 90.0 {
            Err(nom::Err::Failure(nom::error::Error::new(
                lat,
                nom::error::ErrorKind::Fail,
            )))
        } else {
            Ok((rem, mag * value))
        }
    }

    #[cfg(test)]
    mod lat_tests {
        use super::latitude_parser;
        use super::parse_north_or_south;
        use crate::parsers::common::assert_float_approx;

        #[test]
        fn should_parse_direction() {
            assert_eq!(parse_north_or_south("N"), Ok(("", 1.)));
            assert_eq!(parse_north_or_south("+"), Ok(("", 1.)));
            assert_eq!(parse_north_or_south("S"), Ok(("", -1.)));
            assert_eq!(parse_north_or_south("-"), Ok(("", -1.)));
            assert_eq!(parse_north_or_south("-123.123"), Ok(("123.123", -1.)));

            assert!(parse_north_or_south("n").is_err());
        }

        #[test]
        fn should_parse_dd_ddd() {
            assert_eq!(latitude_parser("+45.45"), Ok(("", 45.45)));
            assert_eq!(latitude_parser("N45.45"), Ok(("", 45.45)));
            assert_eq!(latitude_parser("-45.45"), Ok(("", -45.45)));
            assert_eq!(latitude_parser("S45.45"), Ok(("", -45.45)));
            assert_eq!(latitude_parser("S45"), Ok(("", -45.)));
            assert_eq!(latitude_parser("S45.1234"), Ok(("", -45.1234)));

            //Padding
            assert_eq!(latitude_parser("S45.45"), Ok(("", -45.45)));
            assert_eq!(latitude_parser("+05.45"), Ok(("", 5.45)));
            assert_eq!(latitude_parser("+05.05"), Ok(("", 5.05)));

            //Poles
            assert_eq!(latitude_parser("+90.0"), Ok(("", 90.0)));
            assert_eq!(latitude_parser("N90"), Ok(("", 90.0)));
            assert_eq!(latitude_parser("S90"), Ok(("", -90.0)));
            assert_eq!(latitude_parser("-90.0"), Ok(("", -90.0)));
        }

        #[test]
        fn should_error_dd_ddd() {
            assert!(latitude_parser("45.45").is_err());
            assert!(latitude_parser("n45.45").is_err());
            assert!(latitude_parser("+5.45").is_err());
            assert!(latitude_parser("North45.45").is_err());
            assert!(latitude_parser("145.45").is_err());
            assert!(latitude_parser("N99.45").is_err());
            assert!(latitude_parser("+90.1").is_err());
            assert!(latitude_parser("-90.1").is_err());
        }

        #[test]
        fn should_parse_ddmm_mmm() {
            assert_float_approx(latitude_parser("+4520.30"), 45.33833);
            assert_float_approx(latitude_parser("S4520.30"), -45.338333);
            assert_float_approx(latitude_parser("S4520.12304"), -45.335384);
            assert_float_approx(latitude_parser("S4520"), -45.33333);
            assert_float_approx(latitude_parser("S4500"), -45.);
        }

        #[test]
        fn should_error_ddmm_mmm() {
            assert!(latitude_parser("4545.45").is_err());
            assert!(latitude_parser("N4560.45").is_err());
            assert!(latitude_parser("N4560").is_err());
            assert!(latitude_parser("N4590.45").is_err());
        }

        #[test]
        fn should_parse_ddmmss_sss() {
            assert_float_approx(latitude_parser("+452018"), 45.338333);
            assert_float_approx(latitude_parser("S452018"), -45.338333);
            assert_float_approx(latitude_parser("S452000"), -45.33333);
            assert_float_approx(latitude_parser("S450000"), -45.);
            assert_float_approx(latitude_parser("S452035.1528"), -45.343098);
        }
    }
}
