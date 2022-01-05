use std::any::TypeId;

/// This is the layout of a trait object in rust
/// The trait to be implemented by every struct that wants to support
pub struct VTable(*const ());
impl VTable {
    pub fn none() -> VTable {
        VTable(std::ptr::null())
    }
}
//unsafe impl Send for VTable {}
//unsafe impl Sync for VTable {}
pub struct TraitObject {
    pub data: *const (),
    pub vtable: VTable,
}

/// trait casting
pub trait Castable: Send {
    fn query_vtable(&self, id: TypeId) -> Option<VTable>;
}
/// Implementation of the cast
impl dyn Castable {
    pub fn query_ref<U: ?Sized + 'static>(&self) -> Option<&U> {
        if let Some(vtable) = self.query_vtable(::std::any::TypeId::of::<U>()) {
            unsafe {
                let data = self as *const Self;
                let u = TraitObject {
                    data: data as *const (),
                    vtable: vtable,
                };
                Some(*::std::mem::transmute::<_, &&U>(&u))
            }
        } else {
            None
        }
    }

    pub fn query_mut<U: ?Sized + 'static>(&mut self) -> Option<&mut U> {
        if let Some(vtable) = self.query_vtable(::std::any::TypeId::of::<U>()) {
            unsafe {
                let data = self as *mut Self;
                let mut u = TraitObject {
                    data: data as *mut (),
                    vtable: vtable,
                };
                Some(*::std::mem::transmute::<_, &mut &mut U>(&mut u))
            }
        } else {
            None
        }
    }
}
