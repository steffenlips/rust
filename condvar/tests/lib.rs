use std::time::Duration;

use condvar::{ConditionalVariable, OptionalConditionalVariable};

#[test]
fn boolean_conditional_var() {
    let var = ConditionalVariable::new(false);
    let mut var_clone = var.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        var_clone.notify(true);
        assert!(true);
    });

    var.wait(true);
    assert!(true);
}
#[test]
fn int_conditional_var() {
    let var = ConditionalVariable::new(0);
    let mut var_clone = var.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        var_clone.notify(1);
        std::thread::sleep(Duration::from_millis(100));
        var_clone.notify(2);
        assert!(true);
    });

    var.wait(2);
    assert!(true);
}

#[test]
fn int_conditional_var_multiple_threads() {
    let mut var = ConditionalVariable::new(0);
    let var_clone_1 = var.clone();

    let handle_1 = std::thread::spawn(move || {
        var_clone_1.wait(1);
    });

    let var_clone_2 = var.clone();
    let handle_2 = std::thread::spawn(move || {
        var_clone_2.wait(2);
    });

    std::thread::sleep(Duration::from_millis(100));
    var.notify(1);
    std::thread::sleep(Duration::from_millis(100));
    var.notify(2);

    handle_1.join().unwrap();
    handle_2.join().unwrap();

    assert!(true);
}

#[test]
fn int_conditional_var_multiple_threads_one_condition() {
    let mut var = ConditionalVariable::new(false);
    let var_clone_1 = var.clone();

    let handle_1 = std::thread::spawn(move || {
        var_clone_1.wait(true);
    });

    let var_clone_2 = var.clone();
    let handle_2 = std::thread::spawn(move || {
        var_clone_2.wait(true);
    });

    std::thread::sleep(Duration::from_millis(100));
    var.notify(true);

    handle_1.join().unwrap();
    handle_2.join().unwrap();

    assert!(true);
}

#[test]
fn option_conditional_var() {
    let var = OptionalConditionalVariable::<u32>::new();
    let mut var_clone = var.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        var_clone.notify(6);
        assert!(true);
    });

    let result = var.wait();
    let result = result.as_ref();
    assert!(*result == 6);
}
pub struct Test {
    pub val: u32,
}
#[test]
fn option_conditional_var_struct() {
    let var = OptionalConditionalVariable::<Test>::new();
    let mut var_clone = var.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        var_clone.notify(Test { val: 7 });
    });

    assert_eq!(var.wait().val, 7);
}
