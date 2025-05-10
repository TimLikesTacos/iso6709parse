use nom::character::complete::multispace0;
use nom::error::ParseError;
use nom::sequence::delimited;
use nom::Finish;
use nom::IResult;
use parsers::iso6709;

pub mod parsers {
    mod altitude;
    pub(crate) mod common;
    pub mod iso6709;
    mod latitude;
    mod longitude;
}
mod error;
use crate::error::ISO6709Error;

/// The struct that this library's parses create.  `geo_types` `Point` and `Coord` have the `Into` traits  
/// implemented for this struct, so using this struct is only needed if you wish to create your own struct or
/// enum that implements `From<ISO6709Coord>`  
#[derive(Debug, PartialEq, Clone)]
pub struct ISO6709Coord {
    pub lat: f64,
    pub lon: f64,
    pub altitude: Option<f64>,
}

impl From<ISO6709Coord> for geo_types::Point {
    fn from(value: ISO6709Coord) -> Self {
        geo_types::Point::new(value.lon, value.lat)
    }
}

impl From<ISO6709Coord> for geo_types::Coord {
    fn from(value: ISO6709Coord) -> Self {
        geo_types::Coord {
            x: value.lon,
            y: value.lat,
        }
    }
}

/// Parses a string in ISO6709 human readable format into any struct that implements `From<ISO6709Coord>`.  
/// Using a normal single quote `'` in place of `′`, and a double quote `"` in place of `″` is acceptable.  
/// An error will be returned if the resulting coordinate exceeds 90° for latitude and 180° for longitude in either direction.  
/// ```
/// # use iso6709parse::parse_readable;
/// let str = "15°30′00.000″N 95°15′00.000″W";
/// let geo_coord = parse_readable::<geo_types::Coord>(str).unwrap();
/// assert_eq!(geo_coord.x, -95.25);
/// assert_eq!(geo_coord.y, 15.5);
/// ```
pub fn parse_readable<T>(str: &str) -> Result<T, ISO6709Error>
where
    ISO6709Coord: Into<T>,
{
    let (_, ((lat, lon), altitude)) =
        trim(iso6709::human_readable::latlong_altitude_option_parser)(str).finish()?;
    Ok(ISO6709Coord { lat, lon, altitude }.into())
}

/// Parses a string in ISO6709 string representation format into any struct that implements `From<ISO6709Coord>`  
/// Supports the formats:  
/// DD.DDD  
/// DDMM.MMMM  
/// DDMMSS.SSSS  
/// and using either `+`/`-` or `N`/`S` and `E`/`W`.    
/// NOTE: digits less than 10 in the degree, minutes, or seconds column need to have a leading zero, as is IAW ISO6709  
/// An error will be returned if the resulting coordinate exceeds 90° for latitude and 180° for longitude in either direction.  
/// ```
/// # use iso6709parse::parse_string_representation;
/// let str = "N35.50W170.10+8712CRSWGS_85/";
/// let geo_coord = parse_string_representation::<geo_types::Coord>(str).unwrap();
/// assert_eq!(geo_coord.x, -170.1);
/// assert_eq!(geo_coord.y, 35.5);
/// ```
pub fn parse_string_representation<T>(str: &str) -> Result<T, ISO6709Error>
where
    ISO6709Coord: Into<T>,
{
    let (_, ((lat, lon), altitude)) =
        trim(iso6709::string_expression::latlong_altitude_option_parser)(str).finish()?;
    Ok(ISO6709Coord { lat, lon, altitude }.into())
}

/// Parse either of the two different formats.  
/// ```rust
///use iso6709parse::parse;
///
///let coord: geo_types::Coord = parse("N35.50W170.10+8712CRSWGS_85/").unwrap();
///assert_eq!(coord.y, 35.5);
///
///```
pub fn parse<T>(str: &str) -> Result<T, ISO6709Error>
where
    ISO6709Coord: Into<T>,
{
    match parse_readable(str) {
        Ok(x) => Ok(x),
        Err(_) => parse_string_representation(str),
    }
}

fn trim<'a, F, O, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E> + 'a,
    E: ParseError<&'a str>,
{
    delimited(multispace0, inner, multispace0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_readable_format() {
        let mut expected = ISO6709Coord {
            lat: 15.5,
            lon: -95.25,
            altitude: None,
        };

        let coord = "15°30′00.000″N 95°15′00.000″W";
        assert_eq!(parse_readable::<ISO6709Coord>(coord), Ok(expected.clone()));
        let coord = " 15°30′00.000″N 95°15′00.000″W";
        assert_eq!(parse_readable::<ISO6709Coord>(coord), Ok(expected.clone()));
        let coord = " 15°30′00.000″N 95°15′00.000″W ";
        assert_eq!(parse_readable::<ISO6709Coord>(coord), Ok(expected.clone()));

        expected.altitude = Some(123.45);
        let coord = "15°30′00.000″N 95°15′00.000″W 123.45m";
        assert_eq!(parse_readable::<ISO6709Coord>(coord), Ok(expected.clone()));
        let coord = " 15°30′00.000″N 95°15′00.000″W 123.45m ";
        assert_eq!(parse_readable::<ISO6709Coord>(coord), Ok(expected.clone()));
    }

    #[test]
    fn should_parse_string_format() {
        let mut expected = ISO6709Coord {
            lat: 35.5,
            lon: -170.1,
            altitude: None,
        };

        let coord = "N35.50W170.10/";
        assert_eq!(
            parse_string_representation::<ISO6709Coord>(coord),
            Ok(expected.clone())
        );
        let coord = " N35.50W170.10/ ";
        assert_eq!(
            parse_string_representation::<ISO6709Coord>(coord),
            Ok(expected.clone())
        );

        expected.altitude = Some(8712.);
        let coord = "N35.50W170.10+8712CRSWGS_85/";
        assert_eq!(
            parse_string_representation::<ISO6709Coord>(coord),
            Ok(expected.clone())
        );
    }

    #[test]
    fn should_parse_either() {
        let expected = ISO6709Coord {
            lat: 15.5,
            lon: -95.25,
            altitude: None,
        };
        let coord = "15°30′00.000″N 95°15′00.000″W";
        assert_eq!(parse::<ISO6709Coord>(coord), Ok(expected.clone()));

        let expected = ISO6709Coord {
            lat: 35.5,
            lon: -170.1,
            altitude: None,
        };

        let coord = "N35.50W170.10/";
        assert_eq!(parse::<ISO6709Coord>(coord), Ok(expected.clone()));
    }
}
