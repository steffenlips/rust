use di::error::{Error, ErrorCode};
use di::registry::{Registry, SimpleSession};
use proc_macro2::Ident;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::collections::HashSet;
use std::str::FromStr;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::FnArg;
use syn::Item;
use syn::Result;
use syn::ReturnType;
use syn::Token;
use syn::Type;

use di::service::Service;
use di_derive::inject;
use traitcast::Castable;
use traitcast_derive::Castable;

trait SimpleService: Service {
    fn foo(&self) -> u32;
    fn bar(&mut self) -> u32;
}

trait SampleService: Service {
    fn foo(&self) -> u32;
}
#[derive(Castable)]
#[Traits(SimpleService)]
struct SimpleServiceImpl {
    counter: u32,
}
impl SimpleServiceImpl {
    pub fn factory() -> Box<dyn Castable> {
        Box::new(SimpleServiceImpl { counter: 1 })
    }
}
impl SimpleService for SimpleServiceImpl {
    fn foo(&self) -> u32 {
        0
    }

    fn bar(&mut self) -> u32 {
        let res = self.counter;
        self.counter += 1;
        res
    }
}
impl Service for SimpleServiceImpl {}

#[derive(Castable)]
#[Traits(SampleService)]
struct SampleServiceImpl {}
impl SampleServiceImpl {
    pub fn factory() -> Box<dyn Castable> {
        Box::new(SampleServiceImpl {})
    }
}
impl SampleService for SampleServiceImpl {
    fn foo(&self) -> u32 {
        100
    }
}
impl Service for SampleServiceImpl {}

#[inject(injected_param)]
fn func_ref(explicit_param: u32, injected_param: &dyn SimpleService) -> u32 {
    explicit_param + injected_param.foo()
}
#[inject(injected_param)]
fn func_no_params(injected_param: &dyn SimpleService) -> u32 {
    injected_param.foo()
}
#[inject(injected_param, injected_param2)]
fn func_ref_with_two(
    explicit_param: u32,
    injected_param: &dyn SimpleService,
    injected_param2: &dyn SampleService,
) -> u32 {
    explicit_param + injected_param.foo() + injected_param2.foo()
}
#[inject(injected_param)]
fn func_mut(explicit_param: u32, injected_param: &mut dyn SimpleService) -> u32 {
    explicit_param + injected_param.bar()
}
#[inject(injected_param, injected_param2)]
fn func_mut_with_two(
    explicit_param: u32,
    injected_param: &mut dyn SimpleService,
    injected_param2: &dyn SampleService,
) -> u32 {
    explicit_param + injected_param.bar() + injected_param2.foo()
}
#[inject(injected_param)]
fn func_session(
    explicit_param: u32,
    #[session] session: &dyn Session,
    injected_param: &dyn SimpleService,
) -> u32 {
    explicit_param + injected_param.foo()
}

struct Args {
    pub vars: HashSet<Ident>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let vars = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        Ok(Args {
            vars: vars.into_iter().collect(),
        })
    }
}

#[test]
fn parse_attribute_stream() {
    let s = "injected_param1, injected_param2";
    let tokens = TokenStream::from_str(s).unwrap();
    println!("attr: \"{}\"", tokens.to_string());
    let attr_args = syn::parse2::<Args>(tokens).unwrap();
    assert_eq!(
        attr_args
            .vars
            .contains(&Ident::new("injected_param1", Span::call_site())),
        true
    );
    assert_eq!(
        attr_args
            .vars
            .contains(&Ident::new("injected_param2", Span::call_site())),
        true
    );
}
#[test]
fn parse_function_stream() {
    let s = "fn func_ref(explicit_param: u32, #[session] session: u32, injected_param: &dyn SimpleService) -> u32 {
        explicit_param + injected_param.foo()
    }
    ";
    let tokens = TokenStream::from_str(s).unwrap();
    println!("attr: \"{}\"", tokens.to_string());

    let function = syn::parse2::<Item>(tokens).unwrap();
    match function {
        Item::Fn(func) => {
            println!("{}", func.vis.to_token_stream());
            println!("{}", func.sig.ident);
            func.sig.inputs.iter().for_each(|param| match param {
                FnArg::Receiver(recv) => {
                    println!("{}", recv.to_token_stream());
                }
                FnArg::Typed(typ) => {
                    let str = match typ.ty.as_ref() {
                        Type::Array(_) => String::from("TypeArray"),
                        Type::BareFn(_) => String::from("TypeBareFn"),
                        Type::Group(_) => String::from("TypeGroup"),
                        Type::ImplTrait(_) => String::from("TypeImplTrait"),
                        Type::Infer(_) => String::from("TypeInfer"),
                        Type::Macro(_) => String::from("TypeMacro"),
                        Type::Never(_) => String::from("TypeNever"),
                        Type::Paren(_) => String::from("TypeParen"),
                        Type::Path(type_path) => type_path.path.get_ident().unwrap().to_string(),
                        Type::Ptr(_) => String::from("TypePtr"),
                        Type::Reference(_) => String::from("TypeReference"),
                        Type::Slice(_) => String::from("TypeSlice"),
                        Type::TraitObject(_) => String::from("TypeTraitObject"),
                        Type::Tuple(_) => String::from("TypeTuple"),
                        Type::Verbatim(_) => String::from("TokenStream"),
                        _ => panic!("[inject(parameter)]: Unsupported ty type"),
                    };
                    println!("{}", str);
                }
            });
            match func.sig.output {
                ReturnType::Default => println!("-> void"),
                ReturnType::Type(_a, b) => println!("{}", b.to_token_stream().to_string()),
            }
        }
        _ => panic!("[inject(parameter)]: Macro only implemented for functions"),
    }
}

#[test]
fn injects_non_existing_service_as_reference() {
    assert_eq!(
        func_ref(1),
        Err(Error::new(ErrorCode::UnregisteredService, ""))
    );
}
#[test]
fn injects_existing_service_as_reference() {
    Registry::register_service::<dyn SimpleService>(SimpleServiceImpl::factory).unwrap();
    assert_eq!(func_ref(1), Ok(1));
    Registry::unregister_service::<dyn SimpleService>().unwrap();
}
#[test]

fn injects_existing_service_as_reference_no_params() {
    Registry::register_service::<dyn SimpleService>(SimpleServiceImpl::factory).unwrap();
    assert_eq!(func_no_params(), Ok(0));
    Registry::unregister_service::<dyn SimpleService>().unwrap();
}
#[test]
fn injects_two_existing_service_as_reference() {
    Registry::register_service::<dyn SimpleService>(SimpleServiceImpl::factory).unwrap();
    Registry::register_service::<dyn SampleService>(SampleServiceImpl::factory).unwrap();
    assert_eq!(func_ref_with_two(1), Ok(101));
    Registry::unregister_service::<dyn SimpleService>().unwrap();
    Registry::unregister_service::<dyn SampleService>().unwrap();
}
#[test]
fn injects_existing_service_as_mutable() {
    Registry::register_service::<dyn SimpleService>(SimpleServiceImpl::factory).unwrap();
    assert_eq!(func_mut(1), Ok(2));
    assert_eq!(func_mut(1), Ok(3));
    Registry::unregister_service::<dyn SimpleService>().unwrap();
}
#[test]
fn injects_two_existing_service_as_mutable() {
    Registry::register_service::<dyn SimpleService>(SimpleServiceImpl::factory).unwrap();
    Registry::register_service::<dyn SampleService>(SampleServiceImpl::factory).unwrap();
    assert_eq!(func_mut_with_two(1), Ok(102));
    Registry::unregister_service::<dyn SimpleService>().unwrap();
    Registry::unregister_service::<dyn SampleService>().unwrap();
}
#[test]
fn injects_existing_service_as_reference_with_session() {
    let session = SimpleSession::new();
    Registry::register_service::<dyn SimpleService>(SimpleServiceImpl::factory).unwrap();
    assert_eq!(func_session(1, &session), Ok(1));
    Registry::unregister_service::<dyn SimpleService>().unwrap();
}
