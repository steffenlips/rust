use std::sync::{Arc, Mutex};

use crate::error::{Error, ErrorCode};
///
pub trait EventBus {
    fn trigger_event(&mut self, event: Box<dyn Event>) -> Result<(), Error>;
}
///
trait SubscriptionRegistry {
    fn subscribe_event<T: Event>(
        &mut self,
        subscriber: Arc<Mutex<Box<dyn Subscriber<T>>>>,
    ) -> Result<(), Error>;

    fn unsubscribe_event<T: Event>(
        &mut self,
        subscriber: Arc<Mutex<Box<dyn Subscriber<T>>>>,
    ) -> Result<(), Error>;
}
///
trait Subscriber<T: Event> {
    fn notify(&mut self, event: &T);
}
/// An Event has to copy all dataNotImplemented
///
pub trait Event {}

//#[derive(Castable)]
//#[Traits(EventBus)]
pub struct EventBusDefault {}
impl EventBusDefault {
    pub fn new() -> EventBusDefault {
        EventBusDefault {}
    }
}
impl EventBus for EventBusDefault {
    fn trigger_event(&mut self, event: Box<dyn Event>) -> Result<(), Error> {
        Err(Error::new(ErrorCode::NotImplemented, ""))
    }
}
