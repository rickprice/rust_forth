use std::option;

#[derive(Debug)]
pub enum ForthError {
    UnknownError,
    UnknownToken,
    PopOfEmptyStack,
    XParseErrorUserNum,
    XParseErrorGroupNum,
    InvalidInitializationLine,
    InvalidSyntax(String),
    Io(std::io::Error),
}

impl From<std::io::Error> for ForthError {
    fn from(err: std::io::Error) -> ForthError {
        ForthError::Io(err)
    }
}

impl From<ForthError> for i32 {
    fn from(err: ForthError) -> Self {
        match err {
            ForthError::UnknownError => 2,
            ForthError::UnknownToken => 3,
            ForthError::PopOfEmptyStack => 4,
            ForthError::XParseErrorUserNum => 5,
            ForthError::XParseErrorGroupNum => 6,
            ForthError::InvalidInitializationLine => 7,
            ForthError::InvalidSyntax(_) => 8,
            ForthError::Io(_) => 9,
        }
    }
}

impl From<option::NoneError> for ForthError {
    fn from(_: option::NoneError) -> Self {
        ForthError::UnknownError
    }
}