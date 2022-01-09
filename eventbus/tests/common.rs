use di::{registry::Registry, service::Service};
use eventbus::{
    error::Error,
    event::{Event, EventBus, EventBusDefault},
};
use traitcast::Castable;
use traitcast_derive::Castable;

// create a service from default implementation
#[derive(Castable)]
#[Traits(EventBusService)]
struct EventBusDefaultMock(EventBusDefault);
pub trait EventBusService: EventBus + Service {}
impl EventBusDefaultMock {}
impl EventBus for EventBusDefaultMock {
    fn trigger_event(&mut self, event: Box<dyn Event>) -> Result<(), Error> {
        EventBusDefault::trigger_event(&mut self.0, event)
    }
}
impl EventBusService for EventBusDefaultMock {}

impl Service for EventBusDefaultMock {}

impl EventBusDefaultMock {
    pub fn new() -> EventBusDefaultMock {
        EventBusDefaultMock {
            0: EventBusDefault::new(),
        }
    }
}

pub fn setup() {
    Registry::register_service::<dyn EventBusService>(|| -> Box<dyn Castable> {
        Box::new(EventBusDefaultMock::new())
    })
    .unwrap();
}

pub fn teardown() {
    Registry::unregister_service::<dyn EventBusService>().unwrap();
}
