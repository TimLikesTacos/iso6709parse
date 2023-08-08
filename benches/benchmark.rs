use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use iso6709parse::{parse, parse_readable, parse_string_representation};
const READABLE: &str = "15°30′00.000″N 95°15′00.000″W";

pub fn iso6709readable(c: &mut Criterion) {
    c.bench_function("parse_readable", |b| {
        b.iter(|| parse_readable::<geo_types::Coord>(black_box(READABLE)))
    });
}

fn bench_readable(c: &mut Criterion) {
    let mut group = c.benchmark_group("readable_format");

    let strings = [
        "15°30′00.000″N 95°15′00.000″W",
        "15°30′00.000″N 95°15′00.000″W 123.45m",
    ];

    for str in strings.iter() {
        group.bench_with_input(BenchmarkId::new("iso6709parse", str), str, |b, str| {
            b.iter(|| parse::<geo_types::Coord>(black_box(str)))
        });
        group.bench_with_input(
            BenchmarkId::new("iso6709parse_readable", str),
            str,
            |b, str| b.iter(|| parse_readable::<geo_types::Coord>(str)),
        );
        group.bench_with_input(BenchmarkId::new("latlon", str), str, |b, str| {
            b.iter(|| latlon::parse(str))
        });
    }
    group.finish();
}

fn bench_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_representation_format");

    let strings = [
        "+12.10-021.10", //DD.DD
        "+12.10-021.10+2321CRSWGS_85",
        "+1223.101-02123.101", //DDMM.MM
        "+1223.101-02123.101+2321CRSWGS_85",
        "+122345.102-0212345.102", //DDMMSS.SSS
        "+122345.102-0212345.102+2321CRSWGS_85",
    ];

    for str in strings.iter() {
        group.bench_with_input(BenchmarkId::new("iso6709parse", str), str, |b, str| {
            b.iter(|| parse::<geo_types::Coord>(black_box(str)))
        });
        group.bench_with_input(
            BenchmarkId::new("iso6709parse_string", str),
            str,
            |b, str| b.iter(|| parse_string_representation::<geo_types::Coord>(black_box(str))),
        );
        group.bench_with_input(BenchmarkId::new("latlon", str), str, |b, str| {
            b.iter(|| latlon::parse(black_box(str)))
        });
    }
    group.finish();
}
criterion_group!(benches, bench_readable, bench_string);
criterion_main!(benches);
