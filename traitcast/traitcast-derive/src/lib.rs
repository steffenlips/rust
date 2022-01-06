use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Meta, NestedMeta};

#[proc_macro_derive(Castable, attributes(Traits))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_type = &input.ident;

    let mut ifaces = quote!();

    let attrs = input.attrs;
    attrs.iter().for_each(|attr| match attr.path.get_ident() {
        Some(ident) => match ident.to_string().as_str() {
            "Traits" => match attr.parse_meta().unwrap() {
                Meta::List(list) => list.nested.iter().for_each(|data| match data {
                    NestedMeta::Meta(meta) => match meta {
                        Meta::Path(path) => {
                            let iface = path
                                .get_ident()
                                .expect("[derivce(Callable)]: no ident found");
                            let token = quote!(
                                else if id == ::std::any::TypeId::of::<dyn #iface>() {
                                    let x = ::std::ptr::null::<#struct_type>() as *const dyn #iface;
                                    let vt = unsafe { ::std::mem::transmute::<_, traitcast::TraitObject>(x).vtable };
                                    return Some(vt);
                                }
                            );
                            ifaces = quote!(#ifaces #token);
                        }
                        _ => (),
                    },
                    _ => (),
                }),
                _ => (),
            },
            x => panic!("[derivce(Callable)]: Unsupported attribute: {}", x),
        },
        None => panic!("[derivce(Callable)]: Found attribute without ident"),
    });

    let output = quote! {
        impl traitcast::Castable for #struct_type {
            fn query_vtable(&self, id: ::std::any::TypeId) -> Option<traitcast::VTable> {
                if id == ::std::any::TypeId::of::<#struct_type>() {
                    return Some(traitcast::VTable::none());
                }
                #ifaces
                else {
                    None
                }
            }
        }
    };

    //println!("{}", output.to_string().as_str());
    output.into()
}
