use proc_macro::{self, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, GenericArgument, PathArguments, Type,
};

#[proc_macro_derive(Create, attributes(primarykey))]
pub fn derive(input: TokenStream) -> TokenStream {
    //println!("{:#?}", input);
    let input = parse_macro_input!(input as DeriveInput);
    let struct_type = &input.ident;

    let mut sql = String::new();
    sql.push_str("CREATE TABLE ");
    sql.push_str(&struct_type.to_string().to_lowercase());
    sql.push_str("( ");
    /*sql.push_str(
        "CREATE TABLE person (
        id    INTEGER PRIMARY KEY,
        name  TEXT NOT NULL,
        password  TEXT
    )",
    );*/

    let mut first_column = true;
    match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named.iter().for_each(|field| {
                if field.ident.is_none() {
                    return;
                }

                let mut is_primary_key = false;

                field.attrs.iter().for_each(|attr| {
                    println!("attr: {:?}", attr.to_token_stream());
                    if attr.path.get_ident().unwrap().eq("primarykey") {
                        is_primary_key = true;
                    }                    
                });

                let mut data_type = None;
                let mut optional = false;
                match &field.ty {
                    Type::Path(path) => {
                        let mut iter = path.path.segments.iter();
                        match iter.next() {
                            Some(seg) => {
                                println!("Path segement: {:?}", seg.to_token_stream());
                                match seg.ident.to_string().as_str() {
                                    "u32" => data_type = Some("INTEGER".to_owned()),
                                    "String" => data_type = Some("TEXT".to_owned()),
                                    "Option" => {
                                        optional = true;
                                        println!(
                                            "Path segement arg: {:?}",
                                            seg.arguments.to_token_stream()
                                        );
                                        match &seg.arguments {
                                            PathArguments::AngleBracketed(arg) => {
                                                println!(
                                                    "Argtype angle: {:?}",
                                                    arg.to_token_stream()
                                                );
                                                match arg.args.iter().next() {
                                                    Some(x) => match x {
                                                        GenericArgument::Type(ty) => match ty {
                                                            Type::Path(p) => {
                                                                let mut iter = p.path.segments.iter();
                                                                match iter.next() {
                                                                    Some(seg) => {
                                                                        println!("Path segement: {:?}", seg.to_token_stream());
                                                                        match seg.ident.to_string().as_str() {
                                                                            "u32" => data_type = Some("INTEGER".to_owned()),
                                                                            "String" => data_type = Some("TEXT".to_owned()),
                                                                            _=>(),
                                                                        }
                                                                    },
                                                                    None =>(),
                                                                }
                                                            },
                                                            _ =>(),
                                                        }
                                                        ,
                                                        _ => (),
                                                    },

                                                    None => (),
                                                }
                                            }
                                            x => println!("Argtype: {:?}", x.to_token_stream()),
                                        }
                                    }
                                    x => println!("unexpected type {}", x),
                                }
                            }
                            None => (),
                        }
                    }
                    x => println!("Type: {:?}", x.to_token_stream().to_string()),
                }
                match data_type {
                    Some(t) => {
                        if !first_column {
                            sql.push_str(",") 
                        }
                        sql.push_str(" ");
                        sql.push_str(&field.ident.as_ref().unwrap().to_string().to_lowercase());
                        sql.push_str(" ");
                        sql.push_str(&t);
                        if is_primary_key{
                            sql.push_str(" PRIMARY KEY");
                        }
                        else if !optional {
                            sql.push_str(" NOT NULL");
                        }
                        first_column = false;
                    }
                    None => (),
                }
            }),
            _ => (),
        },
        _ => (),
    }
    sql.push_str(")");
    let output = quote! {
        impl db::Create<rusqlite::Connection> for #struct_type {
            fn create(context: &rusqlite::Connection) -> Result<(), String> {
                context
                    .execute(
                        #sql,
                        [],
                    )
                    .unwrap();
                Ok(())
            }
        }
    };

    println!("{}", output.to_string().as_str());
    output.into()
}
