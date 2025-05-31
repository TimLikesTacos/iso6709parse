#![no_main]
use geo_types::Coord;
use iso6709parse;
use iso6709parse::ISO6709Error;
use libfuzzer_sys::fuzz_target;

type Result<T> = std::result::Result<T, ISO6709Error>;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _: Result<Coord> = iso6709parse::parse(s);
    }
    if let Ok(s) = std::str::from_utf8(data) {
        let _: Result<Coord> = iso6709parse::parse_readable(s);
    }
    if let Ok(s) = std::str::from_utf8(data) {
        let _: Result<Coord> = iso6709parse::parse_string_representation(s);
    }
});
