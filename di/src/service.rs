use std::any::Any;

pub trait Service: Any {}

pub type ServiceFactory = fn() -> Box<dyn Service + Send>;
pub type ServiceName = fn() -> String;
