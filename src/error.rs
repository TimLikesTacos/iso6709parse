#[derive(Debug, PartialEq)]
pub struct ISO6709Error(String);

impl std::error::Error for ISO6709Error {}
impl std::fmt::Display for ISO6709Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to parse ISO6709 coordinate: {}", &self.0)
    }
}

impl From<nom::error::Error<&'_ str>> for ISO6709Error {
    fn from(value: nom::error::Error<&'_ str>) -> Self {
        ISO6709Error(value.to_string())
    }
}
