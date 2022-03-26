///////////////////////////////////////////////////////////////////////////////
/// Generic error crate
/// When returning error most of the time you need mots of the time a code that
/// classifies the error. But often a detailed description is usefull to print
/// out in a log file.
/// This crate defines a generic error that has a generic error code.
#[derive(Debug)]
pub struct Error<Code> {
    pub code: Code,
    pub message: String,
}

impl<Code> Error<Code> {
    pub fn new(code: Code, message: &str) -> Error<Code> {
        Error {
            code: code,
            message: message.to_string(),
        }
    }

    pub fn error_code(&self) -> &Code {
        &self.code
    }
}

impl<Code: std::cmp::PartialEq> PartialEq for Error<Code> {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}
