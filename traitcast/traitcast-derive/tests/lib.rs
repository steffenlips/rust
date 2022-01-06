use proc_macro2::TokenStream;
use std::str::FromStr;
use syn::ItemStruct;
use syn::Meta;
use syn::NestedMeta;

use quote::quote;
use traitcast_derive::Castable;

trait Service {}
trait SimpleService {
    fn foo(&self) -> bool;
    fn bar(&mut self) -> bool;
}
trait OtherService {
    fn meth(&self) -> bool;
}
#[derive(Castable)]
#[Traits(SimpleService)]
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

#[test]
fn parser() {
    // struct sample
    let s = "#[Traits(Service
    )]\nstruct ServiceImpl {}";

    // create a new token stream from our string
    let tokens = TokenStream::from_str(s).unwrap();

    // build the AST: note the syn::parse2() method rather than the syn::parse() one
    // which is meant for "real" procedural macros
    let ast: ItemStruct = syn::parse2(tokens).unwrap();

    // save our struct type for future use
    let struct_type = ast.ident;
    assert_eq!(struct_type.to_string(), "ServiceImpl");

    let mut ifaces: TokenStream = quote!();

    let attrs = ast.attrs;
    attrs.iter().for_each(|attr| match attr.path.get_ident() {
        Some(ident) => match ident.to_string().as_str() {
            "Traits" => match attr.parse_meta().unwrap() {
                Meta::List(list) => list.nested.iter().for_each(|data| match data {
                    /*NestedMeta::Lit(lit) => {
                        (match lit {
                            Lit::Str(str) => println!("{}", str.value()),
                            _ => panic!("[derivce(Callable)]: Unsupported nested meta lit type"),
                        })
                    }*/
                    NestedMeta::Meta(meta) => match meta {
                        Meta::Path(path) => {
                            let iface = path
                                .get_ident()
                                .expect("[derivce(Callable)]: no ident found");
                            assert_eq!(iface.to_string(), "Service");
                            let token = quote!({
                                else if id == ::std::any::TypeId::of::<dyn #iface>() {
                                    let x = ::std::ptr::null::<#struct_type>() as *const dyn #iface;
                                    let vt = unsafe { ::std::mem::transmute::<_, TraitObject>(x).vtable };
                                    Some(vt)
                                }
                            });
                            ifaces = quote!(#ifaces #token);
                        }
                        _ => (),
                    },
                    _ => panic!("[derivce(Callable)]: Unsupported nested meta type"),
                }),
                _ => panic!("[derivce(Callable)]: Unsupported meta type"),
            },
            x => panic!("[derivce(Callable)]: Unsupported attribute: {}", x),
        },
        None => panic!("[derivce(Callable)]: Found attribute without ident"),
    });

    let intro = quote! {
        impl Castable for #struct_type {
            fn query_vtable(&self, id: TypeId) -> Optn<VTable> {
                if id == ::std::any::TypeId::of::<#struct_type>() {
                    Some(VTable::none())
                } #ifaces else {
                    None
                }
            }
        }
    };

    let intro = intro.to_string();
    println!("{}", intro);
}
#[test]
fn cast_ref_succeeded() {
    let service_impl = ServiceImpl {};
    let castable = &service_impl as &dyn traitcast::Castable;
    let simple_service = castable.query_ref::<dyn SimpleService>();
    assert_eq!(simple_service.is_some(), true);
    assert_eq!(simple_service.unwrap().foo(), true);
}
#[test]
fn cast_self_succeeded() {
    let service_impl = ServiceImpl {};
    let castable = &service_impl as &dyn traitcast::Castable;
    let simple_service = castable.query_ref::<ServiceImpl>();
    assert_eq!(simple_service.is_some(), true);
    assert_eq!(simple_service.unwrap().foo(), true);
}
#[test]
fn cast_mut_succeeded() {
    let mut service_impl = ServiceImpl {};
    let ref_service_impl: &mut ServiceImpl = &mut service_impl;
    let castable = ref_service_impl as &mut dyn traitcast::Castable;
    let simple_service = castable.query_mut::<dyn SimpleService>();
    assert_eq!(simple_service.is_some(), true);
    assert_eq!(simple_service.unwrap().bar(), true);
}
#[test]
fn cast_ref_failed() {
    let service_impl = ServiceImpl {};
    let castable = &service_impl as &dyn traitcast::Castable;
    let other_service = castable.query_ref::<dyn OtherService>();
    assert_eq!(other_service.is_none(), true);
}
