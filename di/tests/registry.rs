use std::any::Any;

use di::error::{Error, ErrorCode};
use di::registry::Registry;
use di::registry::APPLICATION;
use di::service::Service;
trait SimpleService: Service {
    fn foo(&self);
}
struct SimpleServiceImpl {}
impl SimpleServiceImpl {
    pub fn factory() -> Box<dyn Service + Send> {
        Box::new(SimpleServiceImpl {})
    }
}
impl SimpleService for SimpleServiceImpl {
    fn foo(&self) {}
}
impl Service for SimpleServiceImpl {}

struct AnotherSimpleServiceImpl {}
impl AnotherSimpleServiceImpl {
    pub fn factory() -> Box<dyn Service + Send> {
        Box::new(AnotherSimpleServiceImpl {})
    }
}
impl SimpleService for AnotherSimpleServiceImpl {
    fn foo(&self) {}
}
impl Service for AnotherSimpleServiceImpl {}

#[test]
fn get_unregistered_service() {
    assert_eq!(
        Registry::get_service::<dyn SimpleService>(&APPLICATION).err(),
        Some(Error::new(ErrorCode::UnregisteredService, ""))
    );
}

#[test]
fn register_a_simple_service() {
    assert_eq!(
        Registry::register_service::<dyn SimpleService>(SimpleServiceImpl::factory),
        Ok(())
    );
    Registry::unregister_service::<dyn SimpleService>().ok();
}
#[test]
fn register_already_registered_service() {
    assert_eq!(
        Registry::register_service::<dyn SimpleService>(SimpleServiceImpl::factory),
        Ok(())
    );
    assert_eq!(
        Registry::register_service::<dyn SimpleService>(AnotherSimpleServiceImpl::factory),
        Err(Error::new(ErrorCode::AlreadyRegisteredService, ""))
    );
    Registry::unregister_service::<dyn SimpleService>().ok();
}
#[test]
fn unregister_registered_service() {
    Registry::register_service::<dyn SimpleService>(SimpleServiceImpl::factory).ok();
    assert_eq!(Registry::unregister_service::<dyn SimpleService>(), Ok(()));
    assert_eq!(
        Registry::get_service::<dyn SimpleService>(&APPLICATION).err(),
        Some(Error::new(ErrorCode::UnregisteredService, ""))
    );
}
#[test]
fn create_an_application_service() {
    Registry::register_service::<dyn SimpleService>(SimpleServiceImpl::factory).ok();
    let service = Registry::get_service::<dyn SimpleService>(&APPLICATION);
    assert_eq!(service.is_ok(), true);
    Registry::unregister_service::<dyn SimpleService>().ok();
}
#[test]
fn cast_a_service() {
    let test = SimpleServiceImpl {};

    Registry::register_service::<dyn SimpleService>(SimpleServiceImpl::factory).ok();
    let service = Registry::get_service::<dyn SimpleService>(&APPLICATION);
    assert_eq!(service.is_ok(), true);

    let service = service.unwrap().clone();
    let mut service = service.lock().unwrap();
    let service: &mut dyn Any = unsafe { service.as_mut() as &mut dyn Any };
    //let service = service.downcast_mut::<dyn SimpleService>();
    //let service = unsafe { &mut *(*service as *mut dyn Any as *mut dyn SimpleService) };

    Registry::unregister_service::<dyn SimpleService>().ok();
}
