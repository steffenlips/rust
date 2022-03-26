use error::Error;
use std::sync::{Arc, Mutex};

///
pub enum ErrorCode {
    NotImplemented,
}

///
pub trait EventBus {
    fn trigger_event(&mut self, event: Box<dyn Event>) -> Result<(), Error<ErrorCode>>;
}
///
trait SubscriptionRegistry {
    fn subscribe_event<T: Event>(
        &mut self,
        subscriber: Arc<Mutex<Box<dyn Subscriber<T>>>>,
    ) -> Result<(), Error<ErrorCode>>;

    fn unsubscribe_event<T: Event>(
        &mut self,
        subscriber: Arc<Mutex<Box<dyn Subscriber<T>>>>,
    ) -> Result<(), Error<ErrorCode>>;
}
///
trait Subscriber<T: Event> {
    fn notify(&mut self, event: &T);
}
/// An Event has to copy all dataNotImplemented
///
pub trait Event {}

pub struct EventBusDefault {}
impl EventBusDefault {
    pub fn new() -> EventBusDefault {
        EventBusDefault {}
    }
}
impl EventBus for EventBusDefault {
    fn trigger_event(&mut self, event: Box<dyn Event>) -> Result<(), Error<ErrorCode>> {
        Err(Error::new(ErrorCode::NotImplemented, ""))
    }
}
