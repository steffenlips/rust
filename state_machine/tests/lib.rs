use std::{fmt::Debug, hash::Hash};

use state_machine::{ErrorCode, StateMachineBase};

use error::Error;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum SimpleStates {
    State1,
    State2,
    State3,
}

pub struct SimpleStruct {
    leave_state_1_called: bool,
    enter_state_2_called: bool,
}

impl SimpleStruct {
    fn leave_state_1(
        &mut self,
        _old_state: &SimpleStates,
        _new_state: &SimpleStates,
    ) -> Result<(), Error<ErrorCode>> {
        self.leave_state_1_called = true;
        Ok(())
    }

    fn enter_state_2(
        &mut self,
        _old_state: &SimpleStates,
        _new_state: &SimpleStates,
    ) -> Result<(), Error<ErrorCode>> {
        self.enter_state_2_called = true;
        Ok(())
    }

    pub fn regular(&self) {}
    pub fn regular_mut(&mut self) {}
}

pub trait MyTrait {
    fn foo(&self);
}

/// Code to generate
pub struct StateMachine<T, S: Hash + Eq + Debug + Copy>(StateMachineBase<T, S>);

impl StateMachine<SimpleStruct, SimpleStates> {
    pub fn new(
        inner: SimpleStruct,
        initial: SimpleStates,
    ) -> StateMachine<SimpleStruct, SimpleStates> {
        let mut result = StateMachine::<SimpleStruct, SimpleStates> {
            0: StateMachineBase::<SimpleStruct, SimpleStates>::new(inner, initial),
        };
        result
            .0
            .insert_enter(SimpleStates::State2, SimpleStruct::enter_state_2);
        result
            .0
            .insert_leave(SimpleStates::State1, SimpleStruct::leave_state_1);
        result
            .0
            .add_transition(SimpleStates::State1, SimpleStates::State2);
        result
    }
}
impl<T, S: Hash + Eq + Debug + Copy> std::ops::Deref for StateMachine<T, S> {
    type Target = StateMachineBase<T, S>;
    fn deref(&self) -> &StateMachineBase<T, S> {
        &self.0
    }
}

impl<T, S: Hash + Eq + Debug + Copy> std::ops::DerefMut for StateMachine<T, S> {
    fn deref_mut(&mut self) -> &mut StateMachineBase<T, S> {
        &mut self.0
    }
}

impl MyTrait for StateMachine<SimpleStruct, SimpleStates> {
    fn foo(&self) {}
}

#[test]
fn enter_new_state() {
    let mut sm = StateMachine::<SimpleStruct, SimpleStates>::new(
        SimpleStruct {
            enter_state_2_called: false,
            leave_state_1_called: false,
        },
        SimpleStates::State1,
    );
    assert_eq!(sm.exchange_state(SimpleStates::State2).is_ok(), true);
    assert_eq!(sm.inner.enter_state_2_called, true);
}

#[test]
fn leave_old_state() {
    let mut sm = StateMachine::<SimpleStruct, SimpleStates>::new(
        SimpleStruct {
            enter_state_2_called: false,
            leave_state_1_called: false,
        },
        SimpleStates::State1,
    );
    assert_eq!(sm.exchange_state(SimpleStates::State2).is_ok(), true);
    assert_eq!(sm.inner.leave_state_1_called, true);
}
#[test]
fn invalid_transition() {
    let mut sm = StateMachine::<SimpleStruct, SimpleStates>::new(
        SimpleStruct {
            enter_state_2_called: false,
            leave_state_1_called: false,
        },
        SimpleStates::State1,
    );
    let res = sm.exchange_state(SimpleStates::State3);
    assert_eq!(res.is_err(), true);
    assert_eq!(
        res.err(),
        Some(Error::new(ErrorCode::InvalidTransition, ""))
    );
}
#[test]
fn deref_regular() {
    let mut sm = StateMachine::<SimpleStruct, SimpleStates>::new(
        SimpleStruct {
            enter_state_2_called: false,
            leave_state_1_called: false,
        },
        SimpleStates::State1,
    );
    sm.regular();
    sm.regular_mut();
    sm.foo();
}
