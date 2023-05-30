#[derive(Debug)]
pub enum Error {
    Parse = 1,
    Emission = 2,
    InvalidString = 3,
    UTF8InvalidSlice = 4,
}

impl From<Error> for i8 {
    fn from(v: Error) -> Self {
        v as i8
    }
}
