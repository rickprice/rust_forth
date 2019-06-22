use std::option;

/// This Enum lists the errors that the Forth Interpreter might return
#[derive(Debug)]
pub enum ForthError {
    UnknownError,
    UnknownToken(String),
    PopOfEmptyStack,
    InvalidSyntax(String),
    Io(std::io::Error),
}

/// Convert io::Errors to a ForthError so our Interpreter functions can
/// return a single Error type.
impl From<std::io::Error> for ForthError {
    fn from(err: std::io::Error) -> ForthError {
        ForthError::Io(err)
    }
}

/// Helper to convert ForthError codes to numeric codes for exit()
impl From<ForthError> for i32 {
    fn from(err: ForthError) -> Self {
        match err {
            ForthError::UnknownError => 2,
            ForthError::UnknownToken(_) => 3,
            ForthError::PopOfEmptyStack => 4,
            ForthError::InvalidSyntax(_) => 5,
            ForthError::Io(_) => 6,
        }
    }
}

/// Helper to convert a Some/None return to a ForthError error code.
impl From<option::NoneError> for ForthError {
    fn from(_: option::NoneError) -> Self {
        ForthError::UnknownError
    }
}
