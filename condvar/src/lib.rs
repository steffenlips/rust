use std::sync::{Arc, Condvar, Mutex};

///////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
pub struct ConditionalVariable<T> {
    inner: Arc<(Mutex<T>, Condvar)>,
}
///////////////////////////////////////////////////////////////////////////////
impl<T: std::cmp::PartialEq> ConditionalVariable<T> {
    pub fn new(initial: T) -> ConditionalVariable<T> {
        ConditionalVariable {
            inner: Arc::new((Mutex::new(initial), Condvar::new())),
        }
    }

    pub fn wait(&self, desired_value: T) {
        let clone = self.inner.clone();
        let (lock, cvar) = &*clone;
        let mut current = lock.lock().unwrap();
        while *current != desired_value {
            current = cvar.wait(current).unwrap();
        }
    }

    pub fn notify(&mut self, new_value: T) {
        let clone = self.inner.clone();
        let (lock, cvar) = &*clone;
        let mut current = lock.lock().unwrap();
        *current = new_value;
        cvar.notify_all();
    }
}
///////////////////////////////////////////////////////////////////////////////
//#[derive(Clone)]
pub struct OptionalConditionalVariable<T> {
    inner: Arc<(Mutex<Option<Box<T>>>, Condvar)>,
}
///////////////////////////////////////////////////////////////////////////////
impl<T> OptionalConditionalVariable<T> {
    pub fn new() -> OptionalConditionalVariable<T> {
        OptionalConditionalVariable {
            inner: Arc::new((Mutex::new(None), Condvar::new())),
        }
    }

    pub fn wait(&self) -> Box<T> {
        let clone = self.inner.clone();
        let (lock, cvar) = &*clone;
        let mut current = lock.lock().unwrap();
        while (*current).is_none() {
            current = cvar.wait(current).unwrap();
        }
        (*current).take().unwrap()
    }

    pub fn notify(&mut self, new_value: T) {
        let clone = self.inner.clone();
        let (lock, cvar) = &*clone;
        let mut current = lock.lock().unwrap();
        let _ = current.insert(Box::new(new_value));
        cvar.notify_all();
    }
}
///////////////////////////////////////////////////////////////////////////////
impl<T> Clone for OptionalConditionalVariable<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
