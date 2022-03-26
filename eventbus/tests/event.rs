use di_derive::inject;
use eventbus::event::{Event, EventBus};

mod common;

struct SimpleEvent {
    value: u32,
}
impl SimpleEvent {
    pub fn new(value: u32) -> SimpleEvent {
        SimpleEvent { value }
    }
}
impl Event for SimpleEvent {}

//#[inject(event_bus)]
fn func_trigger_event(event_bus: &mut dyn common::EventBusService) {
    let event = SimpleEvent::new(1);
    //assert_eq!(event_bus.trigger_event(Box::new(event)), Ok(()));
}

#[test]
fn trigger_event() {
    common::setup();
    //func_trigger_event().unwrap();
    common::teardown();
}
#[test]
fn subscribe_event() {}
#[test]
fn already_subscribed() {}
#[test]
fn unsubscribe_event() {}
#[test]
fn not_subscribed_on_unsubscribe() {}
