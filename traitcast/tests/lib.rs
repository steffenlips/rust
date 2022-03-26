use std::any::TypeId;

use traitcast::{Castable, TraitObject, VTable};
trait Service: Castable {}
trait SimpleService {
    fn foo(&self) -> bool;
    fn bar(&mut self) -> bool;
}
trait OtherService {
    fn meth(&self) -> bool;
}
struct ServiceImpl {}
impl SimpleService for ServiceImpl {
    fn foo(&self) -> bool {
        println!("immutable foo called");
        return true;
    }

    fn bar(&mut self) -> bool {
        println!("mutable bar called");
        return true;
    }
}
impl ServiceImpl {}
impl Service for ServiceImpl {}
impl Castable for ServiceImpl {
    fn query_vtable(&self, id: TypeId) -> Option<VTable> {
        if id == ::std::any::TypeId::of::<ServiceImpl>() {
            Some(VTable::none())
        } else if id == ::std::any::TypeId::of::<dyn SimpleService>() {
            let x = ::std::ptr::null::<ServiceImpl>() as *const dyn SimpleService;
            let vt = unsafe { ::std::mem::transmute::<_, TraitObject>(x).vtable };
            Some(vt)
        } else {
            None
        }
    }
}

impl ServiceImpl {}

#[test]
fn cast_ref_succeeded() {
    let service_impl = ServiceImpl {};
    let castable = &service_impl as &dyn Castable;
    let simple_service = castable.query_ref::<dyn SimpleService>();
    assert_eq!(simple_service.is_some(), true);
    assert_eq!(simple_service.unwrap().foo(), true);
}
#[test]
fn cast_mut_succeeded() {
    let mut service_impl = ServiceImpl {};
    let ref_service_impl: &mut ServiceImpl = &mut service_impl;
    let castable = ref_service_impl as &mut dyn Castable;
    let simple_service = castable.query_mut::<dyn SimpleService>();
    assert_eq!(simple_service.is_some(), true);
    assert_eq!(simple_service.unwrap().bar(), true);
}
#[test]
fn cast_ref_failed() {
    let service_impl = ServiceImpl {};
    let castable = &service_impl as &dyn Castable;
    let other_service = castable.query_ref::<dyn OtherService>();
    assert_eq!(other_service.is_none(), true);
}
