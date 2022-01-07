use std::collections::HashSet;

use proc_macro::{self, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    FnArg, Ident, Item, Pat, Result, ReturnType, Token, Type,
};

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

#[proc_macro_attribute]
pub fn inject(attr: TokenStream, item: TokenStream) -> TokenStream {
    // read arguments to substitute
    let injected_args: Args = parse_macro_input!(attr);
    injected_args
        .vars
        .iter()
        .for_each(|arg| () /*println!("{}", arg)*/);

    let mut function_signature = quote!();
    let mut injected_services = quote!();
    let mut function_block = quote!();

    // read function
    let function = parse_macro_input!(item as Item);
    match function {
        Item::Fn(func) => {
            let mut parameter = quote!();
            let mut num_parameter = 0;
            func.sig.inputs.iter().for_each(|param| match param {
                FnArg::Receiver(recv) => {
                    //println!("{}", recv.to_token_stream());
                }
                FnArg::Typed(typ) => {
                    let param_name = match typ.pat.as_ref() {
                        Pat::Ident(ident) => &ident.ident,
                        _ => panic!("[inject(parameter)]: Unsupported pat type"),
                    };

                    let typref = match typ.ty.as_ref() {
                        Type::Reference(reference) => Some(reference),
                        _ => None, //panic!("[inject(parameter)]: Unsupported ty type"),
                    };

                    if !injected_args.vars.contains(param_name) {
                        parameter = quote!(#parameter);
                        if num_parameter > 0 {
                            parameter = quote!(#parameter, );
                            num_parameter += 1;
                        }
                        parameter = quote!(#parameter #typ);
                    } else if typref.is_some() {
                        let typref = typref.unwrap();
                        let elem = &typref.elem;
                        let mutability = typref.mutability;
                        injected_services = quote!(#injected_services
                        let #param_name = di::registry::Registry::get_service::<#elem>(&di::registry::SimpleSession::default())?; 
                        let #param_name = #param_name.clone();
                        let #mutability #param_name = #param_name.lock().unwrap();
                        let #param_name = #param_name.as_ref().query_ref::<#elem>().ok_or_else(|| Error::new(ErrorCode::Unimplemented, format!("Service of type {} not implemented", stringify!(#elem)).as_str()))?;
                        );
                    }
                }
            });

            let visibility = func.vis.to_token_stream();
            let name = func.sig.ident;
            let output = match func.sig.output {
                ReturnType::Default => quote!(),
                ReturnType::Type(_a, b) => b.to_token_stream(),
            };
            function_signature = quote! {
                #visibility fn #name (#parameter) -> std::result::Result<#output, di::error::Error>
            };
            function_block = func.block.to_token_stream();
        }
        _ => panic!("[inject(parameter)]: Macro only implemented for functions"),
    }

    let output = quote! {
        #function_signature{
            #injected_services
            let result = #function_block;
            Ok(result)
        }
    };

    println!("{}", output.to_string().as_str());
    output.into()
    //quote!().into()
}
