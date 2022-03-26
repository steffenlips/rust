use std::str::FromStr;

use proc_macro::{self, token_stream::IntoIter, Delimiter, TokenStream, TokenTree};
use quote::quote;
use syn::{ImplItem, ImplItemMethod, ItemImpl, Type};

#[proc_macro_attribute]
pub fn state_machine(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut transitions = Transitions::new();
    transitions.parse(attr);
    //println!("Transitions: {:?}", transitions);

    let mut state_functions = StateFunctions::new();
    state_functions.parse(item);
    //println!("StateFunctions: {:?}", state_functions);

    let previous = state_functions.updated_impl;

    let sm_struct = quote!(
        pub struct StateMachine<T, S: std::hash::Hash + Eq + std::fmt::Debug + Copy>(
            state_machine::StateMachineBase<T, S>,
        );
    );

    let mut sm_impl = String::new();
    sm_impl.push_str("\nimpl StateMachine<");
    sm_impl.push_str(&state_functions.struct_name);
    sm_impl.push_str(", ");
    sm_impl.push_str(&transitions.state_enum);
    sm_impl.push_str("> {");

    sm_impl.push_str("\npub fn new(inner: ");
    sm_impl.push_str(&state_functions.struct_name);
    sm_impl.push_str(", initial: ");
    sm_impl.push_str(&transitions.state_enum);
    sm_impl.push_str(") -> StateMachine<");
    sm_impl.push_str(&state_functions.struct_name);
    sm_impl.push_str(", ");
    sm_impl.push_str(&transitions.state_enum);
    sm_impl.push_str("> {");

    sm_impl.push_str("\nlet mut result = StateMachine {");
    sm_impl.push_str("\n0: state_machine::StateMachineBase::new(inner, initial)};");

    state_functions.enter_functions.iter().for_each(|f| {
        sm_impl.push_str("\nresult.insert_enter(");
        sm_impl.push_str(&f.0);
        sm_impl.push_str(", ");
        sm_impl.push_str(&state_functions.struct_name);
        sm_impl.push_str("::");
        sm_impl.push_str(&f.1);
        sm_impl.push_str(");");
    });

    state_functions.leave_functions.iter().for_each(|f| {
        sm_impl.push_str("\nresult.insert_leave(");
        sm_impl.push_str(&f.0);
        sm_impl.push_str(", ");
        sm_impl.push_str(&state_functions.struct_name);
        sm_impl.push_str("::");
        sm_impl.push_str(&f.1);
        sm_impl.push_str(");");
    });

    transitions.transitions.iter().for_each(|f| {
        sm_impl.push_str("\nresult.add_transition(");
        sm_impl.push_str(&f.0);
        sm_impl.push_str(", ");
        sm_impl.push_str(&f.1);
        sm_impl.push_str(");");
    });

    sm_impl.push_str("result");

    sm_impl.push_str("}");

    sm_impl.push_str("}");

    sm_impl.push_str(
        "\nimpl<T, S: std::hash::Hash + Eq + std::fmt::Debug + Copy> std::ops::Deref for StateMachine<T, S> {
        type Target = state_machine::StateMachineBase<T, S>;
        fn deref(&self) -> &state_machine::StateMachineBase<T, S> {
            &self.0
        }
    }
    
    impl<T, S: std::hash::Hash + Eq + std::fmt::Debug + Copy> std::ops::DerefMut for StateMachine<T, S> {
        fn deref_mut(&mut self) -> &mut state_machine::StateMachineBase<T, S> {
            &mut self.0
        }
    }",
    );

    let sm_impl = proc_macro2::TokenStream::from_str(sm_impl.as_str()).expect("msg");

    let output = quote!(
        #previous
        #sm_struct
        #sm_impl
    );
    //println!("{}", output.to_string().as_str());
    output.into()
}

#[derive(Debug)]
struct StateFunctions {
    struct_name: String,
    enter_functions: Vec<(String, String)>,
    leave_functions: Vec<(String, String)>,
    updated_impl: proc_macro2::TokenStream,
    group_impl: proc_macro2::TokenStream,
}
impl StateFunctions {
    fn new() -> StateFunctions {
        StateFunctions {
            struct_name: String::new(),
            enter_functions: Vec::new(),
            leave_functions: Vec::new(),
            updated_impl: quote!(),
            group_impl: quote!(),
        }
    }

    fn parse(&mut self, stream: TokenStream) {
        let item = syn::parse::<ItemImpl>(stream)
            .expect("[state_machine] Only available for implementations");

        self.parse_impl(item);
    }

    fn parse_impl(&mut self, item_impl: ItemImpl) {
        item_impl.attrs.iter().for_each(|attr| {
            let previous = self.updated_impl.clone();
            self.updated_impl = quote! {
                #previous
                #attr
            };
            //self.updated_impl
            //   .push_str(&attr.to_token_stream().to_string());
            //self.updated_impl.push_str(" ");
        });
        match item_impl.defaultness {
            Some(def) => {
                let previous = &self.updated_impl.clone();
                self.updated_impl = quote! {
                    #previous
                    #def
                };
                //self.updated_impl
                //    .push_str(&def.to_token_stream().to_string());
                //self.updated_impl.push_str(" ");
            }
            None => (),
        }
        match item_impl.unsafety {
            Some(uns) => {
                let previous = &self.updated_impl.clone();
                self.updated_impl = quote! {
                    #previous
                    #uns
                };
                //self.updated_impl
                //    .push_str(&uns.to_token_stream().to_string());
                //self.updated_impl.push_str(" ");
            }
            None => (),
        }
        let previous = &self.updated_impl.clone();
        let tok = &item_impl.impl_token;
        let generics = &item_impl.generics;
        self.updated_impl = quote! {
            #previous #tok #generics
        };
        //self.updated_impl
        //    .push_str(&item_impl.impl_token.to_token_stream().to_string());
        //self.updated_impl.push_str(" ");
        //self.updated_impl
        //    .push_str(&item_impl.generics.to_token_stream().to_string());
        //self.updated_impl.push_str(" ");
        match item_impl.trait_ {
            Some(ref tr) => {
                match tr.0 {
                    Some(bang) => {
                        let previous = &self.updated_impl.clone();
                        self.updated_impl = quote! {
                            #previous
                            #bang
                        };
                        //self.updated_impl
                        //    .push_str(&bang.to_token_stream().to_string());
                        //self.updated_impl.push_str(" ");
                    }
                    None => (),
                }
                let previous = &self.updated_impl.clone();
                let tr1 = &tr.1;
                let tr2 = &tr.2;
                self.updated_impl = quote! {
                    #previous #tr1 #tr2
                };
                //self.updated_impl
                //    .push_str(&tr.1.to_token_stream().to_string());
                //self.updated_impl.push_str(" ");
                //self.updated_impl
                //    .push_str(&tr.2.to_token_stream().to_string());
                //self.updated_impl.push_str(" ");
            }
            None => (),
        }
        let previous = &self.updated_impl.clone();
        let ty = &item_impl.self_ty;
        self.updated_impl = quote! {
            #previous #ty
        };
        //self.updated_impl
        //    .push_str(&item_impl.self_ty.to_token_stream().to_string());

        match &item_impl.self_ty.as_ref() {
            &Type::Path(x) => x.path.segments.iter().for_each(|seg| {
                if self.struct_name.is_empty() {
                    self.struct_name = seg.ident.to_string();
                }
            }),
            _ => panic!("[state_machine] Hä?"),
        }

        //self.updated_impl.push_str(" { ");

        item_impl
            .items
            .iter()
            .for_each(|item| self.parse_item(item));

        let previous = &self.updated_impl.clone();
        let group = &self.group_impl;
        self.updated_impl = quote! {
            #previous {
                #group
            }
        };

        //self.updated_impl.push_str(" }");
    }

    fn parse_item(&mut self, item: &ImplItem) {
        match item {
            ImplItem::Method(method) => self.parse_method(method),
            x => {
                let previous = self.group_impl.clone();
                self.group_impl = quote! (
                    #previous
                    #x
                )
            }
        }
    }

    fn parse_method(&mut self, method: &ImplItemMethod) {
        method.attrs.iter().for_each(|attr| {
            if attr.path.get_ident().is_some() {
                match attr.path.get_ident().unwrap().to_string().as_str() {
                    "leave_state" => {
                        self.leave_functions
                            .push((attr.tokens.to_string(), method.sig.ident.to_string()));
                        return;
                    }
                    "enter_state" => {
                        self.enter_functions
                            .push((attr.tokens.to_string(), method.sig.ident.to_string()));
                        return;
                    }
                    _ => (),
                }
            }
            let previous = self.group_impl.clone();
            self.group_impl = quote! (
                #previous
                #attr
            )
        });

        let previous = self.group_impl.clone();
        let vis = &method.vis;
        self.group_impl = quote! (
            #previous
            #vis
        );

        //self.updated_impl
        //    .push_str(&method.vis.to_token_stream().to_string());
        //self.updated_impl.push_str(" ");
        match method.defaultness {
            Some(def) => {
                let previous = self.group_impl.clone();
                self.group_impl = quote! (
                    #previous #def
                );

                //self.updated_impl
                //    .push_str(&def.to_token_stream().to_string());
                //self.updated_impl.push_str(" ");
            }
            None => (),
        }

        let previous = self.group_impl.clone();
        let sig = &method.sig;
        let block = &method.block;
        self.group_impl = quote! (
            #previous #sig
            #block
        );

        //self.updated_impl
        //    .push_str(&method.sig.to_token_stream().to_string());
        //self.updated_impl.push_str(" ");
        //self.updated_impl
        //    .push_str(&method.block.to_token_stream().to_string());
    }
}

#[derive(Debug)]
struct Transitions {
    state_enum: String,
    transitions: Vec<(String, String)>,
}
impl Transitions {
    fn new() -> Transitions {
        Transitions {
            state_enum: String::new(),
            transitions: Vec::new(),
        }
    }

    fn parse(&mut self, stream: TokenStream) {
        let mut iter = stream.into_iter();
        let token_tree = iter.next();
        match token_tree {
            Some(TokenTree::Group(group)) => {
                if !group.delimiter().eq(&Delimiter::Brace) {
                    panic!("[state_machine] Wrong transition syntax. Expected curly braces");
                }
                self.parse_group(group.stream());
            }
            x => panic!("[state_machine] Unexpected transition token {:?}", x),
        }
    }

    fn parse_group(&mut self, stream: TokenStream) {
        let mut iter = stream.into_iter();
        loop {
            let from = self.parse_state(&mut iter);
            self.parse_arrow(&mut iter);
            let to = self.parse_state(&mut iter);
            self.transitions.push((from, to));
            if self.parse_comma(&mut iter).is_none() {
                break;
            }
        }
    }

    fn parse_state(&mut self, iter: &mut IntoIter) -> String {
        let mut state = String::new();
        let mut current = iter
            .next()
            .expect("[state_machine] Incomplete transition. Missing State type");

        state.push_str(
            &match current {
                TokenTree::Ident(id) => {
                    if self.state_enum.is_empty() {
                        self.state_enum.push_str(&id.to_string());
                    } else if !self.state_enum.eq(&id.to_string()) {
                        panic!("[state_machine] Multiple state enums used");
                    }
                    id
                }
                x => panic!("[state_machine] Expected ident but get {:?}", x),
            }
            .to_string(),
        );
        current = iter
            .next()
            .expect("[state_machine] Incomplete transition. Missing '::'");
        state.push_str(
            &match current {
                TokenTree::Punct(id) => {
                    if id.as_char() != ':' {
                        panic!("[state_machine] Expected punct(:) but get {:?}", id)
                    }
                    id
                }
                x => panic!("[state_machine] Expected punct but get {:?}", x),
            }
            .to_string(),
        );
        current = iter
            .next()
            .expect("[state_machine] Incomplete transition. Missing '::'");
        state.push_str(
            &match current {
                TokenTree::Punct(id) => {
                    if id.as_char() != ':' {
                        panic!("[state_machine] Expected punct(:) but get {:?}", id)
                    }
                    id
                }
                x => panic!("[state_machine] Expected punct but get {:?}", x),
            }
            .to_string(),
        );
        current = iter
            .next()
            .expect("[state_machine] Incomplete transition. Missing state value");
        state.push_str(
            &match current {
                TokenTree::Ident(id) => id,
                x => panic!("[state_machine] Expected ident but get {:?}", x),
            }
            .to_string(),
        );

        state
    }

    fn parse_arrow(&mut self, iter: &mut IntoIter) {
        let mut state = String::new();
        let mut current = iter
            .next()
            .expect("[state_machine] Incomplete transition. Missing =");

        state.push_str(
            &match current {
                TokenTree::Punct(id) => {
                    if id.as_char() != '=' {
                        panic!("[state_machine] Expected punct(=) but get {:?}", id)
                    }
                    id
                }
                x => panic!("[state_machine] Expected punct but get {:?}", x),
            }
            .to_string(),
        );
        current = iter
            .next()
            .expect("[state_machine] Incomplete transition. Missing '>'");
        state.push_str(
            &match current {
                TokenTree::Punct(id) => {
                    if id.as_char() != '>' {
                        panic!("[state_machine] Expected punct(>) but get {:?}", id)
                    }
                    id
                }
                x => panic!("[state_machine] Expected punct but get {:?}", x),
            }
            .to_string(),
        );
    }

    fn parse_comma(&mut self, iter: &mut IntoIter) -> Option<()> {
        let current = iter.next();

        match current {
            Some(TokenTree::Punct(id)) => {
                if id.as_char() != ',' {
                    panic!("[state_machine] Expected punct(,) but get {:?}", id)
                }
                return Some(());
            }
            None => return None,
            x => panic!("[state_machine] ] Expected punct but get {:?}", x),
        }
    }
}
