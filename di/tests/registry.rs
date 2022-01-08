use std::thread;
use std::time::Duration;

use di::error::{Error, ErrorCode};
use di::registry::{Registry, SimpleSession};
use di::service::Service;

use traitcast::Castable;
use traitcast_derive::Castable;

trait SimpleService: Service {
    fn foo(&self) -> bool;
    fn bar(&mut self) -> u32;
}
#[derive(Castable)]
#[Traits(SimpleService)]
struct SimpleServiceImpl {
    counter: u32,
}
impl SimpleServiceImpl {
    pub fn factory() -> Box<dyn Castable> {
        Box::new(SimpleServiceImpl { counter: 0 })
    }
}

impl Drop for SimpleServiceImpl {
    fn drop(&mut self) {
        println!("service uninitialized!")
    }
}
impl SimpleService for SimpleServiceImpl {
    fn foo(&self) -> bool {
        return true;
    }

    fn bar(&mut self) -> u32 {
        let res = self.counter;
        self.counter += 1;
        res
    }
}
impl Service for SimpleServiceImpl {
    fn initialize(&mut self) {
        println!("service initialized!")
    }
}

#[derive(Castable)]
#[Traits(SimpleService)]
struct AnotherSimpleServiceImpl {}
impl AnotherSimpleServiceImpl {
    pub fn factory() -> Box<dyn Castable> {
        Box::new(AnotherSimpleServiceImpl {})
    }
}
impl SimpleService for AnotherSimpleServiceImpl {
    fn foo(&self) -> bool {
        false
    }

    fn bar(&mut self) -> u32 {
        0
    }
}
impl Service for AnotherSimpleServiceImpl {}

#[test]
fn get_unregistered_service() {
    assert_eq!(
        Registry::get_service::<dyn SimpleService>(&SimpleSession::default()).err(),
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
        Registry::get_service::<dyn SimpleService>(&SimpleSession::default()).err(),
        Some(Error::new(ErrorCode::UnregisteredService, ""))
    );
}
#[test]
fn create_an_application_service() {
    Registry::register_service::<dyn SimpleService>(SimpleServiceImpl::factory).ok();
    let service = Registry::get_service::<dyn SimpleService>(&SimpleSession::default());
    assert_eq!(service.is_ok(), true);
    Registry::unregister_service::<dyn SimpleService>().ok();
}
#[test]
fn cast_a_service() {
    Registry::register_service::<dyn SimpleService>(SimpleServiceImpl::factory).ok();
    let service = Registry::get_service::<dyn SimpleService>(&SimpleSession::default());
    assert_eq!(service.is_ok(), true);

    let service = service.unwrap().clone();
    let service = service.lock().unwrap();
    let service = service.as_ref().query_ref::<dyn SimpleService>();
    assert_eq!(service.is_some(), true);
    assert_eq!(service.unwrap().foo(), true);
    Registry::unregister_service::<dyn SimpleService>().ok();
}
#[test]
fn cast_a_mutable_service() {
    Registry::register_service::<dyn SimpleService>(SimpleServiceImpl::factory).ok();
    let service = Registry::get_service::<dyn SimpleService>(&SimpleSession::default());
    assert_eq!(service.is_ok(), true);

    let service = service.unwrap().clone();
    let mut service = service.lock().unwrap();
    let service = service.as_mut().query_mut::<dyn SimpleService>();
    assert_eq!(service.is_some(), true);
    assert_eq!(service.unwrap().bar(), 0);
    Registry::unregister_service::<dyn SimpleService>().ok();
}
#[test]
fn get_same_service_in_a_thread() {
    Registry::register_service::<dyn SimpleService>(SimpleServiceImpl::factory).ok();
    let service = Registry::get_service::<dyn SimpleService>(&SimpleSession::default());

    let t = thread::spawn(move || {
        let service = Registry::get_service::<dyn SimpleService>(&SimpleSession::default());
        let service = service.unwrap().clone();

        thread::sleep(Duration::from_millis(200));
        {
            let mut service = service.lock().unwrap();
            let service = service.as_mut().query_mut::<dyn SimpleService>();
            assert_eq!(service.is_some(), true);
            assert_eq!(service.unwrap().bar(), 1);
        }
    });

    assert_eq!(service.is_ok(), true);
    {
        let service = service.unwrap().clone();
        let mut service = service.lock().unwrap();
        let service = service.as_mut().query_mut::<dyn SimpleService>();
        assert_eq!(service.is_some(), true);
        assert_eq!(service.unwrap().bar(), 0);
    }
    thread::sleep(Duration::from_millis(100));

    Registry::unregister_service::<dyn SimpleService>().ok();
    assert_eq!(
        Registry::get_service::<dyn SimpleService>(&SimpleSession::default()).err(),
        Some(Error::new(ErrorCode::UnregisteredService, ""))
    );
    t.join().unwrap();
}
#[test]
fn get_service_for_other_session() {
    Registry::register_service::<dyn SimpleService>(SimpleServiceImpl::factory).ok();
    let service = Registry::get_service::<dyn SimpleService>(&SimpleSession::default());

    let t = thread::spawn(move || {
        let session = SimpleSession::new();
        let service = Registry::get_service::<dyn SimpleService>(&session);
        let service = service.unwrap().clone();

        {
            let mut service = service.lock().unwrap();
            let service = service.as_mut().query_mut::<dyn SimpleService>();
            assert_eq!(service.is_some(), true);
            assert_eq!(service.unwrap().bar(), 0);
        }
    });

    assert_eq!(service.is_ok(), true);
    {
        let service = service.unwrap().clone();
        let mut service = service.lock().unwrap();
        let service = service.as_mut().query_mut::<dyn SimpleService>();
        assert_eq!(service.is_some(), true);
        assert_eq!(service.unwrap().bar(), 0);
    }
    thread::sleep(Duration::from_millis(100));

    Registry::unregister_service::<dyn SimpleService>().ok();
    assert_eq!(
        Registry::get_service::<dyn SimpleService>(&SimpleSession::default()).err(),
        Some(Error::new(ErrorCode::UnregisteredService, ""))
    );
    t.join().unwrap();
}
