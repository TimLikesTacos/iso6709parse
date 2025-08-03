use nom::IResult;

pub mod human_readable {
    use super::*;
    use crate::parsers::altitude::human_readable::*;
    use crate::parsers::latitude::human_readable::*;
    use crate::parsers::longitude::human_readable::*;
    use nom::character::complete::space1;
    use nom::combinator::opt;
    use nom::sequence::{preceded, separated_pair};
    use nom::Parser;

    /// Parser to obtain lat long
    ///
    ///
    /// ```
    /// # use iso6709parse::parsers::iso6709::human_readable::latlong_parser;
    /// let coord = "15°30′00.000″N 95°15′00.000″W";
    /// assert_eq!(latlong_parser(coord), Ok(("", (15.5, -95.25))));
    ///
    /// let coord = "15°30′00.000″N 95°15′00.000″W 123.45m";
    /// assert_eq!(latlong_parser(coord), Ok((" 123.45m", (15.5, -95.25))));
    /// ```
    ///  
    pub fn latlong_parser(inp: &str) -> IResult<&str, (f64, f64)> {
        separated_pair(latitude_parser, space1, longitude_parser).parse(inp)
    }

    /// Parser to obtain lat long and altitude. Note that the lat, long are within their own tuple, inside the output tuple.
    /// Since the `CRS` statement is required for altitude, it is parsed and discarded from the remaining string
    ///
    ///
    /// ```
    /// # use iso6709parse::parsers::iso6709::human_readable::latlong_altitude_parser;
    /// let coord = "15°30′00.000″N 95°15′00.000″W";
    /// assert!(latlong_altitude_parser(coord).is_err());
    ///
    /// let coord = "15°30′00.000″N 95°15′00.000″W 123.45m";
    /// assert_eq!(latlong_altitude_parser(coord), Ok(("m", ((15.5, -95.25), 123.45))));
    /// ```
    ///  
    pub fn latlong_altitude_parser(inp: &str) -> IResult<&str, ((f64, f64), f64)> {
        separated_pair(latlong_parser, space1, altitude_parser).parse(inp)
    }

    /// Parser to obtain lat long and altitude if the altitude is present. Note that the lat, long are within their own tuple, inside the output tuple.
    /// Since the `CRS` statement is required for altitude, it is parsed and discarded from the remaining string
    ///
    ///
    /// ```
    /// # use iso6709parse::parsers::iso6709::human_readable::latlong_altitude_option_parser;
    /// let coord = "15°30′00.000″N 95°15′00.000″W";
    /// assert_eq!(latlong_altitude_option_parser(coord), Ok(("", ((15.5, -95.25), None))));
    ///
    /// let coord = "15°30′00.000″N 95°15′00.000″W 123.45m";
    /// assert_eq!(latlong_altitude_option_parser(coord), Ok(("m", ((15.5, -95.25), Some(123.45)))));
    /// ```
    ///  
    pub fn latlong_altitude_option_parser(inp: &str) -> IResult<&str, ((f64, f64), Option<f64>)> {
        (latlong_parser, opt(preceded(space1, altitude_parser))).parse(inp)
    }

    #[cfg(test)]
    mod human_readable_tests {
        use super::*;

        #[test]
        fn should_parse_readable() {
            let coord = "15°30′00.000″N 95°15′00.000″W";
            assert_eq!(latlong_parser(coord), Ok(("", (15.5, -95.25))));

            let coord = "15°30′00.000″N 95°15′00.000″W 123.45m";
            assert_eq!(latlong_parser(coord), Ok((" 123.45m", (15.5, -95.25))));
        }

        #[test]
        fn should_parse_readable_altitude() {
            let coord = "15°30′00.000″N 95°15′00.000″W";
            assert!(latlong_altitude_parser(coord).is_err());

            let coord = "15°30′00.000″N 95°15′00.000″W 123.45m";
            assert_eq!(
                latlong_altitude_parser(coord),
                Ok(("m", ((15.5, -95.25), 123.45)))
            );
        }
    }
}

pub mod string_expression {
    use super::*;
    pub(crate) use crate::parsers::altitude::string_expression::altitude_parser;
    pub use crate::parsers::latitude::string_expression::latitude_parser;
    pub use crate::parsers::longitude::string_expression::longitude_parser;
    use nom::combinator::opt;
    use nom::Parser;

    /// Parser to obtain lat long
    ///
    ///
    /// ```
    /// # use iso6709parse::parsers::iso6709::string_expression::latlong_parser;
    /// let coord = "+1200.00-02130.00";
    /// assert_eq!(latlong_parser(coord), Ok(("", (12.0, -21.5))));
    ///
    /// let coord = "+1200.00-02130.00+2321CRS_WGS_85/";
    /// assert_eq!(latlong_parser(coord), Ok(("+2321CRS_WGS_85/", (12.0, -21.5))));
    /// ```
    ///  
    pub fn latlong_parser(inp: &str) -> IResult<&str, (f64, f64)> {
        (latitude_parser, longitude_parser).parse(inp)
    }

    /// Parser to obtain lat long and altitude. Note that the lat, long are within their own tuple, inside the output tuple.
    /// Since the `CRS` statement is required for altitude, it is parsed and discarded from the remaining string
    ///
    ///
    /// ```
    /// # use iso6709parse::parsers::iso6709::string_expression::latlong_altitude_parser;
    /// let coord = "+1200.00-02130.00";
    /// assert!(latlong_altitude_parser(coord).is_err());
    ///
    /// let coord = "+1200.00-02130.00+2321CRSWGS_85";
    /// assert_eq!(latlong_altitude_parser(coord), Ok(("WGS_85", ((12.0, -21.5), 2321.0))));
    /// ```
    ///  
    pub fn latlong_altitude_parser(inp: &str) -> IResult<&str, ((f64, f64), f64)> {
        (latlong_parser, altitude_parser).parse(inp)
    }

    // Parser to obtain lat long and, if exists, the altitude. Note that the lat, long are within their own tuple, inside the output tuple.
    /// Since the `CRS` statement is required for altitude, it is parsed and discarded from the remaining string
    ///
    ///
    /// ```
    /// # use iso6709parse::parsers::iso6709::string_expression::latlong_altitude_option_parser;
    /// let coord = "+1200.00-02130.00";
    /// assert_eq!(latlong_altitude_option_parser(coord), Ok(("", ((12.0, -21.5), None))));
    ///
    /// let coord = "+1200.00-02130.00+2321CRSWGS_85";
    /// assert_eq!(latlong_altitude_option_parser(coord), Ok(("WGS_85", ((12.0, -21.5), Some(2321.0)))));
    /// ```
    ///  
    pub fn latlong_altitude_option_parser(inp: &str) -> IResult<&str, ((f64, f64), Option<f64>)> {
        (latlong_parser, opt(altitude_parser)).parse(inp)
    }
    #[cfg(test)]
    mod string_expression_tests {
        use super::*;

        #[test]
        fn should_parse_latlong() {
            assert_eq!(latlong_parser("+35.50+170.00"), Ok(("", (35.5, 170.0))));
            assert_eq!(latlong_parser("+35.50-170.10"), Ok(("", (35.5, -170.1))));
            assert_eq!(latlong_parser("+35-170"), Ok(("", (35., -170.))));
            assert_eq!(latlong_parser("+05.50-070.10"), Ok(("", (5.5, -70.1))));
            assert_eq!(latlong_parser("N35.50W170.10"), Ok(("", (35.5, -170.1))));

            assert_eq!(latlong_parser("+3530+17030"), Ok(("", (35.5, 170.5))));
            assert_eq!(latlong_parser("+3530.0-17030.0"), Ok(("", (35.5, -170.5))));

            assert_eq!(latlong_parser("+05.50-070.10"), Ok(("", (5.5, -70.1))));
            assert_eq!(
                latlong_parser("N35.50W170.10+8712CRSWGS_85/"),
                Ok(("+8712CRSWGS_85/", (35.5, -170.1)))
            )
        }

        #[test]
        fn should_parse_latlong_altitude() {
            assert_eq!(
                latlong_altitude_parser("N35.50W170.10+8712CRSWGS_85/"),
                Ok(("WGS_85/", ((35.5, -170.1), 8712.)))
            );
            assert_eq!(
                latlong_altitude_parser("N35.50W170.10-8712CRSWGS_85/"),
                Ok(("WGS_85/", ((35.5, -170.1), -8712.)))
            );
            assert_eq!(
                latlong_altitude_parser("N35.50W170.10-8712.5CRSWGS_85/"),
                Ok(("WGS_85/", ((35.5, -170.1), -8712.5)))
            )
        }
    }
}
