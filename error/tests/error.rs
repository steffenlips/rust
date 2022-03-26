use error::Error;

#[derive(PartialEq, Debug)]
pub enum ErrorCode {
    Code1,
    Code2,
}

#[test]
fn error_equals() {
    let err1 = Error::<ErrorCode>::new(ErrorCode::Code1, "One message");
    let err2 = Error::<ErrorCode>::new(ErrorCode::Code1, "Another message");

    assert_eq!(err1, err2);
}
#[test]
fn error_not_equals() {
    let err1 = Error::<ErrorCode>::new(ErrorCode::Code1, "One message");
    let err2 = Error::<ErrorCode>::new(ErrorCode::Code2, "One message");

    assert_ne!(err1, err2);
}
