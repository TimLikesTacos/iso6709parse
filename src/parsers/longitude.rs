use nom::AsChar;
use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while_m_n;
use nom::character::complete::{digit1, u8};
use nom::combinator::{map, opt, value};
use nom::combinator::{map_parser, map_res, recognize};
use nom::error::ParseError;

pub mod human_readable {
    use super::*;
    use crate::parsers::common::human_readable::*;
    //     50°40′46.461″N 95°48′26.533″W 123.45m
    //     50°03′46.461″S 125°48′26.533″E 978.90m

    fn parse_east(inp: &str) -> IResult<&str, f64> {
        value(1., tag("E")).parse(inp)
    }

    fn parse_west(inp: &str) -> IResult<&str, f64> {
        value(-1., tag("W")).parse(inp)
    }

    fn parse_east_or_west(inp: &str) -> IResult<&str, f64> {
        alt((parse_east, parse_west)).parse(inp)
    }

    pub fn longitude_parser(inp: &str) -> IResult<&str, f64> {
        let (rem, deg) = parse_degree(inp)?;
        let (rem, min) = parse_minutes(rem)?;
        let (rem, sec) = parse_seconds(rem)?;
        let (rem, mag) = parse_east_or_west(rem)?;
        let value = deg + min / 60. + sec / 3600.;
        if value > 180.0 {
            Err(nom::Err::Failure(nom::error::Error::new(
                inp,
                nom::error::ErrorKind::Fail,
            )))
        } else {
            Ok((rem, mag * value))
        }
    }

    #[cfg(test)]
    mod lon_test {
        use super::*;
        use crate::parsers::common::assert_float_approx;

        #[test]
        fn should_parse_logitude() {
            let inp = "95°48′26.533″W 123.45m";
            assert_float_approx(longitude_parser(inp), -95.80737);
            let inp = "95°48′26.533″E 123.45m";
            assert_float_approx(longitude_parser(inp), 95.80737);
            let inp = "95°48'26.533″W 123.45m";
            assert_float_approx(longitude_parser(inp), -95.80737);
            let inp = r#"95°48′26.533"W 123.45m"#;
            assert_float_approx(longitude_parser(inp), -95.80737);
            let inp = "95°48′26.533″W 123.45m";
            assert_float_approx(longitude_parser(inp), -95.80737);
            let inp = "180°00′0″W 123.45m";
            assert_float_approx(longitude_parser(inp), -180.);
            let inp = "180°00′0″E 123.45m";
            assert_float_approx(longitude_parser(inp), 180.);
        }

        #[test]
        fn should_err_longitude() {
            let inp = "95.48′26.533″W 123.45m";
            assert!(longitude_parser(inp).is_err());
            let inp = "95°48.26.533″W 123.45m";
            assert!(longitude_parser(inp).is_err());
            let inp = "95°48′26.533.W 123.45m";
            assert!(longitude_parser(inp).is_err());
            let inp = "180°′1.″W 123.45m";
            assert!(longitude_parser(inp).is_err());
            let inp = "180°′1.″E 123.45m";
            assert!(longitude_parser(inp).is_err());
            let inp = "95.48′26.533″ 123.45m";
            assert!(longitude_parser(inp).is_err());
        }
    }
}
pub mod string_expression {
    use super::*;

    fn parse_east(inp: &str) -> IResult<&str, f64> {
        value(1., alt((tag("E"), tag("+")))).parse(inp)
    }

    fn parse_west(inp: &str) -> IResult<&str, f64> {
        value(-1., alt((tag("W"), tag("-")))).parse(inp)
    }

    fn parse_east_or_west(inp: &str) -> IResult<&str, f64> {
        alt((parse_east, parse_west)).parse(inp)
    }

    fn is_char_digit(char: char) -> bool {
        char.is_ascii() && AsChar::is_dec_digit(char as u8)
    }

    fn parse_two<'a, F, O, E>(inner: F) -> impl Parser<&'a str, Output = O, Error = E>
    where
        F: Fn(&'a str) -> IResult<&'a str, O, E> + 'a,
        E: ParseError<&'a str>,
    {
        map_parser(take_while_m_n(2, 2, is_char_digit), inner)
    }

    fn parse_three<'a, F, O, E>(inner: F) -> impl Parser<&'a str, Output = O, Error = E>
    where
        F: Fn(&'a str) -> IResult<&'a str, O, E> + 'a,
        E: ParseError<&'a str>,
    {
        map_parser(take_while_m_n(3, 3, is_char_digit), inner)
    }

    fn parse_degree_integer(inp: &str) -> IResult<&str, f64> {
        map(parse_three(u8), |x| x as f64).parse(inp)
    }

    fn parse_degree_min_integer(inp: &str) -> IResult<&str, f64> {
        let (rem, (degrees, minutes)) = (parse_three(u8), parse_two(u8)).parse(inp)?;

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
            (parse_three(u8), parse_two(u8), parse_two(u8)).parse(inp)?;

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

    pub fn parse_decimal(inp: &str) -> IResult<&str, f64> {
        map_res(recognize((tag("."), digit1)), |x: &str| x.parse::<f64>()).parse(inp)
    }

    fn parse_degree(inp: &str) -> IResult<&str, f64> {
        let (decimalstr, int) = parse_degree_integer(inp)?;
        let (rem, dec) = opt(parse_decimal).parse(decimalstr)?;
        Ok((rem, int + dec.unwrap_or(0.)))
    }

    fn parse_degree_minute(inp: &str) -> IResult<&str, f64> {
        let (decimalstr, int) = parse_degree_min_integer(inp)?;
        let (rem, dec) = opt(parse_decimal).parse(decimalstr)?;
        Ok((rem, int + dec.unwrap_or(0.) / 60.))
    }

    fn parse_degree_minute_second(inp: &str) -> IResult<&str, f64> {
        let (decimalstr, int) = parse_degree_min_sec_integer(inp)?;
        let (rem, dec) = opt(parse_decimal).parse(decimalstr)?;
        Ok((rem, int + dec.unwrap_or(0.) / 3600.))
    }

    pub fn longitude_parser(inp: &str) -> IResult<&str, f64> {
        let (lat, mag) = parse_east_or_west(inp)?;
        // Order matters for the next line!
        let (rem, value) = alt((
            parse_degree_minute_second,
            parse_degree_minute,
            parse_degree,
        ))
        .parse(lat)?;
        if value > 180.0 {
            Err(nom::Err::Failure(nom::error::Error::new(
                lat,
                nom::error::ErrorKind::Fail,
            )))
        } else {
            Ok((rem, mag * value))
        }
    }

    #[cfg(test)]
    mod long_tests {
        use super::longitude_parser;
        use super::parse_decimal;
        use nom::IResult;

        fn assert_float_no_remaining<E: std::fmt::Debug>(
            expected: IResult<&str, f64, E>,
            actual: f64,
        ) {
            let expected = expected.unwrap();
            let difference = (expected.1 - actual).abs();
            assert!(
                (expected.1 - actual).abs() < 0.0001f64,
                "Difference: {difference}, expected: {0}, actual: {actual}",
                expected.1
            );
        }

        #[test]
        fn should_parse_decimal() {
            let input = ".50";
            assert_float_no_remaining(parse_decimal(input), 0.5)
        }

        #[test]
        fn should_parse_ddd_ddd() {
            assert_eq!(longitude_parser("+145.45"), Ok(("", 145.45)));
            assert_eq!(longitude_parser("E145.45"), Ok(("", 145.45)));
            assert_eq!(longitude_parser("-145.45"), Ok(("", -145.45)));
            assert_eq!(longitude_parser("W145.45"), Ok(("", -145.45)));
            assert_eq!(longitude_parser("W145"), Ok(("", -145.)));
            assert_eq!(longitude_parser("W145.1234"), Ok(("", -145.1234)));

            //Padding
            assert_eq!(longitude_parser("W045.45"), Ok(("", -45.45)));
            assert_eq!(longitude_parser("+005.45"), Ok(("", 5.45)));
            assert_eq!(longitude_parser("+005.05"), Ok(("", 5.05)));

            //Meridan
            assert_eq!(longitude_parser("+180.0"), Ok(("", 180.0)));
            assert_eq!(longitude_parser("E180"), Ok(("", 180.0)));
            assert_eq!(longitude_parser("W180"), Ok(("", -180.0)));
            assert_eq!(longitude_parser("-180.0"), Ok(("", -180.0)));
        }

        #[test]
        fn should_error_dd_ddd() {
            assert!(longitude_parser("45.45").is_err());
            assert!(longitude_parser("w45.45").is_err());
            assert!(longitude_parser("+5.45").is_err());
            assert!(longitude_parser("West45.45").is_err());
            assert!(longitude_parser("145.45").is_err());
            assert!(longitude_parser("N129.45").is_err());
            assert!(longitude_parser("+180.1").is_err());
            assert!(longitude_parser("-180.1").is_err());
        }

        #[test]
        fn should_parse_ddmm_mmm() {
            assert_float_no_remaining(longitude_parser("+14520.30"), 145.338333);
            // assert_float_no_remaining(longitude_parser("W14520.30"), -145.338333);
            // assert_float_no_remaining(longitude_parser("W14520.12304"), -145.335384);
            // assert_float_no_remaining(longitude_parser("W14520"), -145.33333);
            // assert_float_no_remaining(longitude_parser("W14500"), -145.);
        }

        #[test]
        fn should_error_ddmm_mmm() {
            assert!(longitude_parser("4545.45").is_err());
            assert!(longitude_parser("N4560.45").is_err());
            assert!(longitude_parser("N4560").is_err());
            assert!(longitude_parser("N4590.45").is_err());
            assert!(
                longitude_parser("N45a5.45").is_err(),
                "{:?}",
                longitude_parser("N45a5.45")
            );
        }

        #[test]
        fn should_parse_dddmmss_sss() {
            assert_float_no_remaining(longitude_parser("+1452018"), 145.338333);
            assert_float_no_remaining(longitude_parser("W1452018"), -145.338333);
            assert_float_no_remaining(longitude_parser("W1452000"), -145.33333);
            assert_float_no_remaining(longitude_parser("W1450000"), -145.);
            assert_float_no_remaining(longitude_parser("W1452035.1528"), -145.343098);
        }
    }
}
