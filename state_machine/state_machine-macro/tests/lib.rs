use error::Error;
use proc_macro2::Delimiter;
use proc_macro2::Group;

use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use quote::__private::ext::RepToTokensExt;
use state_machine::ErrorCode;
use state_machine_macro::state_machine;
use std::str::FromStr;
use std::{fmt::Debug, hash::Hash};

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum SimpleStates {
    State1,
    State2,
    State3,
}

pub struct SimpleStruct {
    leave_state_1_called: bool,
    enter_state_2_called: bool,
}

#[state_machine({
        SimpleStates::State1 => SimpleStates::State2
})]
impl SimpleStruct {
    #[leave_state(SimpleStates::State1)]
    fn leave_state_1(
        &mut self,
        _old_state: &SimpleStates,
        _new_state: &SimpleStates,
    ) -> Result<(), Error<ErrorCode>> {
        self.leave_state_1_called = true;
        Ok(())
    }
    #[enter_state(SimpleStates::State2)]
    fn enter_state_2(
        &mut self,
        _old_state: &SimpleStates,
        _new_state: &SimpleStates,
    ) -> Result<(), Error<ErrorCode>> {
        self.enter_state_2_called = true;
        Ok(())
    }

    pub fn regular(&self) {}
    pub fn regular_mut(&mut self) {}
}

#[test]
fn parse_attribute_stream() {
    let s = "{
        SimpleStates::State1 => SimpleStates::State2,
        SimpleStates::State2 => SimpleStates::State1
    }";
    let tokens = TokenStream::from_str(s).unwrap();
    let group = syn::parse2::<Group>(tokens).unwrap();
    assert_eq!(group.delimiter().eq(&Delimiter::Brace), true);
    let mut transitions = String::new();

    let mut enum_str = String::new();
    let mut iter = group.stream().into_iter();
    loop {
        let mut current = iter
            .next()
            .expect("Incomplete transition. Missing State type");
        transitions.push_str("\nresult.add_transition(");
        transitions.push_str(
            &match current {
                TokenTree::Ident(id) => {
                    if enum_str.is_empty() {
                        enum_str.push_str(&id.to_string());
                    } else if !enum_str.eq(&id.to_string()) {
                        panic!("unexpected state type");
                    }
                    id
                }
                x => panic!("Expected ident but get {:?}", x),
            }
            .to_string(),
        );
        current = iter.next().expect("Incomplete transition. Missing '::'");
        transitions.push_str(
            &match current {
                TokenTree::Punct(id) => {
                    if id.as_char() != ':' {
                        panic!("Expected punct(:) but get {:?}", id)
                    }
                    id
                }
                x => panic!("Expected punct but get {:?}", x),
            }
            .to_string(),
        );
        current = iter.next().expect("Incomplete transition. Missing '::'");
        transitions.push_str(
            &match current {
                TokenTree::Punct(id) => {
                    if id.as_char() != ':' {
                        panic!("Expected punct(:) but get {:?}", id)
                    }
                    id
                }
                x => panic!("Expected punct but get {:?}", x),
            }
            .to_string(),
        );
        current = iter
            .next()
            .expect("Incomplete transition. Missing state value");
        transitions.push_str(
            &match current {
                TokenTree::Ident(id) => id,
                x => panic!("Expected ident but get {:?}", x),
            }
            .to_string(),
        );
        current = iter.next().expect("Incomplete transition. Missing '=>'");
        match current {
            TokenTree::Punct(id) => {
                if id.as_char() != '=' {
                    panic!("Expected punct(=) but get {:?}", id)
                }
            }
            x => panic!("Expected punct but get {:?}", x),
        }

        current = iter.next().expect("Incomplete transition. Missing '=>'");
        match current {
            TokenTree::Punct(id) => {
                if id.as_char() != '>' {
                    panic!("Expected punct(>) but get {:?}", id)
                }
            }
            x => panic!("Expected punct but get {:?}", x),
        }
        current = iter
            .next()
            .expect("Incomplete transition. Missing State type");
        transitions.push_str(", ");
        transitions.push_str(
            &match current {
                TokenTree::Ident(id) => {
                    if !enum_str.eq(&id.to_string()) {
                        panic!("unexpected state type");
                    }
                    id
                }
                x => panic!("Expected ident but get {:?}", x),
            }
            .to_string(),
        );
        current = iter.next().expect("Incomplete transition. Missing '::'");
        transitions.push_str(
            &match current {
                TokenTree::Punct(id) => {
                    if id.as_char() != ':' {
                        panic!("Expected punct(:) but get {:?}", id)
                    }
                    id
                }
                x => panic!("Expected punct but get {:?}", x),
            }
            .to_string(),
        );
        current = iter.next().expect("Incomplete transition. Missing '::'");
        transitions.push_str(
            &match current {
                TokenTree::Punct(id) => {
                    if id.as_char() != ':' {
                        panic!("Expected punct(:) but get {:?}", id)
                    }
                    id
                }
                x => panic!("Expected punct but get {:?}", x),
            }
            .to_string(),
        );
        current = iter
            .next()
            .expect("Incomplete transition. Missing state value");
        transitions.push_str(
            &match current {
                TokenTree::Ident(id) => id,
                x => panic!("Expected ident but get {:?}", x),
            }
            .to_string(),
        );
        transitions.push_str(");");
        let next = iter.next();
        if next.is_none() {
            break;
        }
        current = next.unwrap();
        match current {
            TokenTree::Punct(id) => {
                if id.as_char() != ',' {
                    panic!("Expected punct(,) but get {:?}", id)
                }
            }
            x => panic!("Expected punct but get {:?}", x),
        }
    }

    assert_eq!(enum_str, "SimpleStates");
    println!("{}", transitions);
}
#[test]
fn parse_function_stream() {
    let s = "impl SimpleStruct {
        #[previous_attr]
        #[leave_state(SimpleStates::State1)]
        #[next_attr]
        fn leave_state_1(
            &mut self,
            _old_state: &SimpleStates,
            _new_state: &SimpleStates,
        ) -> Result<(), Error<ErrorCode>> {
            self.leave_state_1_called = true;
            Ok(())
        }
    }
    ";
    let tokens = TokenStream::from_str(s).unwrap();
    let mut iter = tokens.into_iter();
    let mut impl_str = String::new();
    let mut impl_sm = String::new();
    let mut struct_str = String::new();
    impl_sm.push_str(
        "pub struct StateMachine<T, S> {
        phantom_t: PhantomData<T>,
        phantom_s: PhantomData<S>,
    }
    impl StateMachine<SimpleStruct, SimpleStates> {
        pub fn new(
            inner: SimpleStruct,
            initial: SimpleStates,
        ) -> StateMachineBase<SimpleStruct, SimpleStates> {
            let mut result = StateMachineBase::<SimpleStruct, SimpleStates>::new(inner, initial);
    ",
    );
    // write everthing until group
    let group_stream = loop {
        let current = iter.next().expect("Unexpected end of token stream");
        let group = match current {
            TokenTree::Group(group) => {
                impl_str.push_str("{");
                Some(group)
            }
            TokenTree::Ident(ident) => {
                struct_str = ident.to_string();
                impl_str.push_str(&ident.to_string());
                impl_str.push_str(" ");
                None
            }
            x => {
                impl_str.push_str(&x.to_string());
                impl_str.push_str(" ");
                None
            }
        };
        if group.is_some() {
            break group.unwrap().stream();
        }
    };

    iter = group_stream.into_iter();
    let mut entered = false;
    let mut fn_started = false;
    loop {
        let current = iter.next();
        if current.is_none() {
            break;
        }
        let current = current.unwrap();

        match current {
            TokenTree::Punct(ref punct) => {
                if punct.as_char() != '#' {
                    impl_str.push_str(&punct.to_string());
                } else {
                    let group = iter.next().expect("Missing attribute definition");
                    // check if it is leave of enter
                    let inner_current = group.next();
                    match inner_current {
                        Some(TokenTree::Group(g)) => {
                            let mut gstream_iter = g.stream().into_iter();
                            let t = gstream_iter.next();
                            match t {
                                Some(TokenTree::Ident(ident)) => {
                                    let ident_str = ident.to_string();
                                    if ident_str.eq("leave_state") || ident_str.eq("enter_state") {
                                        if ident_str.eq("leave_state") {
                                            impl_sm.push_str("\nresult.insert_enter(");
                                        } else {
                                            impl_sm.push_str("\nresult.insert_leave(");
                                        }
                                        match gstream_iter.next() {
                                            Some(TokenTree::Group(gr)) => {
                                                impl_sm.push_str(&gr.stream().to_string());
                                            }
                                            _ => {
                                                panic!("Syntax error in enter_state or leave_state")
                                            }
                                        }
                                        impl_sm.push_str(", ");
                                        impl_sm.push_str(&struct_str);
                                        impl_sm.push_str("::");
                                        entered = true;
                                    } else {
                                        impl_str.push_str(" ");
                                        impl_str.push_str(&current.to_string());
                                        impl_str.push_str(&group.to_string());
                                    }
                                }
                                Some(_) => {
                                    impl_str.push_str(" ");
                                    impl_str.push_str(&current.to_string());
                                    impl_str.push_str(&group.to_string());
                                }
                                None => (),
                            };
                        }
                        Some(_) => {
                            impl_str.push_str(" ");
                            impl_str.push_str(&current.to_string());
                            impl_str.push_str(&group.to_string());
                        }
                        None => (),
                    }
                }
            }
            x => {
                if x.to_string().eq("fn") && entered {
                    fn_started = true;
                } else if fn_started && entered {
                    impl_sm.push_str(&x.to_string());
                    impl_sm.push_str(");");
                    entered = false;
                    fn_started = false;
                }
                impl_str.push_str(" ");
                impl_str.push_str(&x.to_string());
            }
        }
    }

    impl_sm.push_str(
        "
        result
    }
}
    ",
    );
    impl_str.push_str("}");
    println!("{}", impl_sm);
    println!("{}", impl_str);

    assert_eq!(struct_str, "SimpleStruct");
}

#[test]
fn enter_new_state() {
    let mut sm = StateMachine::<SimpleStruct, SimpleStates>::new(
        SimpleStruct {
            enter_state_2_called: false,
            leave_state_1_called: false,
        },
        SimpleStates::State1,
    );
    assert_eq!(sm.exchange_state(SimpleStates::State2).is_ok(), true);
    assert_eq!(sm.inner.enter_state_2_called, true);
}

#[test]
fn leave_old_state() {
    let mut sm = StateMachine::<SimpleStruct, SimpleStates>::new(
        SimpleStruct {
            enter_state_2_called: false,
            leave_state_1_called: false,
        },
        SimpleStates::State1,
    );
    assert_eq!(sm.exchange_state(SimpleStates::State2).is_ok(), true);
    assert_eq!(sm.inner.leave_state_1_called, true);
}
#[test]
fn invalid_transition() {
    let mut sm = StateMachine::<SimpleStruct, SimpleStates>::new(
        SimpleStruct {
            enter_state_2_called: false,
            leave_state_1_called: false,
        },
        SimpleStates::State1,
    );
    let res = sm.exchange_state(SimpleStates::State3);
    assert_eq!(res.is_err(), true);
    assert_eq!(
        res.err(),
        Some(Error::new(ErrorCode::InvalidTransition, ""))
    );
}
#[test]
fn deref_regular() {
    let mut sm = StateMachine::<SimpleStruct, SimpleStates>::new(
        SimpleStruct {
            enter_state_2_called: false,
            leave_state_1_called: false,
        },
        SimpleStates::State1,
    );
    sm.regular();
    sm.regular_mut();
}
