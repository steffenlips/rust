use traitcast::Castable;

pub trait Service: Castable {
    fn initialize(&mut self) {}
    fn uninitialize(&mut self) {}
}

pub type ServiceFactory = fn() -> Box<dyn Castable>;
pub type ServiceName = fn() -> String;

// use trait cast
