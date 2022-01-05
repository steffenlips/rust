use traitcast::Castable;

pub trait Service: Castable {}

pub type ServiceFactory = fn() -> Box<dyn Castable>;
pub type ServiceName = fn() -> String;

// use trait cast
