use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    ops::{Deref, DerefMut},
    sync::Mutex,
};

use error::Error;
///////////////////////////////////////////////////////////////////////////////
/// State machine
///
#[derive(PartialEq, Debug)]
pub enum ErrorCode {
    InvalidTransition,
    TransitionFailed,
}

pub struct StateMachineBase<T, S: Hash + Eq + Debug + Copy> {
    pub inner: T,
    state: S,
    leave_functions: HashMap<S, fn(&mut T, &S, &S) -> Result<(), Error<ErrorCode>>>,
    enter_functions: HashMap<S, fn(&mut T, &S, &S) -> Result<(), Error<ErrorCode>>>,
    transitions: HashSet<(S, S)>,
    state_mutex: Mutex<u8>,
}

impl<T, S: Hash + Eq + Debug + Copy> StateMachineBase<T, S> {
    pub fn new(inner: T, initial: S) -> StateMachineBase<T, S> {
        StateMachineBase::<T, S> {
            inner: inner,
            state: initial,
            leave_functions: HashMap::new(),
            enter_functions: HashMap::new(),
            transitions: HashSet::new(),
            state_mutex: Mutex::new(0),
        }
    }

    pub fn insert_enter(
        &mut self,
        state: S,
        callback: fn(&mut T, &S, &S) -> Result<(), Error<ErrorCode>>,
    ) {
        self.enter_functions.insert(state, callback);
    }

    pub fn insert_leave(
        &mut self,
        state: S,
        callback: fn(&mut T, &S, &S) -> Result<(), Error<ErrorCode>>,
    ) {
        self.leave_functions.insert(state, callback);
    }

    pub fn add_transition(&mut self, from: S, to: S) {
        self.transitions.insert((from, to));
    }

    pub fn exchange_state(&mut self, new_state: S) -> Result<(), Error<ErrorCode>> {
        let _guard = self.state_mutex.lock().or_else(|err| {
            Err(Error::new(
                ErrorCode::TransitionFailed,
                format!("Could not initiate transition ({err})").as_str(),
            ))
        })?;

        if self.state == new_state {
            return Ok(());
        }

        let key = &(self.state, new_state);
        if !self.transitions.is_empty() && !self.transitions.contains(key) {
            return Err(Error::new(
                ErrorCode::InvalidTransition,
                format!(
                    "Unallowed transition from {:?} to {:?}",
                    self.state, new_state
                )
                .as_str(),
            ));
        }

        let leave_func = self.leave_functions.get(&self.state);
        if leave_func.is_some() {
            leave_func.unwrap()(&mut self.inner, &self.state, &new_state)?;
        }

        let enter_func = self.enter_functions.get(&new_state);
        if enter_func.is_some() {
            enter_func.unwrap()(&mut self.inner, &self.state, &new_state)?;
        }
        self.state = new_state;
        Ok(())
    }
}

impl<T, S: Hash + Eq + Debug + Copy> Deref for StateMachineBase<T, S> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T, S: Hash + Eq + Debug + Copy> DerefMut for StateMachineBase<T, S> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}
