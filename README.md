# ISO6709 parser

[![Rust CI](https://github.com/TimLikesTacos/iso6709parse/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/TimLikesTacos/iso6709parse/actions/workflows/rust-ci.yml)

This library uses the `nom` crate to create parsers to quickly convert ISO6709 formatted strings.  This results in a much faster parse than using Regex based libraries, from 4 to 10x faster.

`From` traits have been implemented for the `geo_types` crate for easy conversion from strings.

Supports formats for latitude with `N` or `S` and `E` and `W` instead of `+` or `-`:  
`±DD.DD`  
`±DDMM.MMM`
`±DDMMSS.SSS`

for longitude:
`±DDD.DDD`
`±DDDMM.MMM`
`±DDDMMSS.SSS`

along with altitude when properly formatted IAW ISO6709, for example `+1200.00-02130.00+2321CRS_WGS_85/`

Also supports the "Human Readable" format:
`DD°MM′SS.SSS″N DDD°MM′SS.SSS″W`  


/// ```rust
///use iso6709parse::parse;
///
///let coord: geo_types::Coord = parse("N35.50W170.10+8712CRSWGS_85/").unwrap();
///assert_eq!(coord.y, 35.5);
///
///```

