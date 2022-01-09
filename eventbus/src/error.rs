#[derive(PartialEq, Debug)]
pub enum ErrorCode {
    AlreadySubscribed,
    NotSubscribed,
    NotImplemented,
}
#[derive(Debug)]
pub struct Error {
    pub code: ErrorCode,
    pub message: String,
}

impl Error {
    pub fn new(code: ErrorCode, message: &str) -> Error {
        Error {
            code: code,
            message: message.to_string(),
        }
    }

    pub fn error_code(&self) -> &ErrorCode {
        &self.code
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}
