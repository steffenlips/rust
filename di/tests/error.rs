use di::error::{Error, ErrorCode};

#[test]
fn error_equals() {
    let err1 = Error::new(ErrorCode::Uninitialized, "One message");
    let err2 = Error::new(ErrorCode::Uninitialized, "Another message");

    assert_eq!(err1, err2);
}
#[test]
fn error_not_equals() {
    let err1 = Error::new(ErrorCode::Uninitialized, "One message");
    let err2 = Error::new(ErrorCode::UnregisteredService, "One message");

    assert_ne!(err1, err2);
}
