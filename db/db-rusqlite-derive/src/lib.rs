use proc_macro::{self, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, Data, DataStruct, DeriveInput, Fields, GenericArgument, PathArguments, Type,
};

///////////////////////////////////////////////////////////////////////////////
struct Column {
    pub name: String,
    pub typ: String,
    pub optional: bool,
    pub primary_key: bool,
}
///////////////////////////////////////////////////////////////////////////////
struct Table {
    pub name: String,
    pub columns: Vec<Column>,
}
///////////////////////////////////////////////////////////////////////////////
fn parse_struct(data: &DataStruct, table: &mut Table) {
    match &data.fields {
        Fields::Named(fields) => fields.named.iter().for_each(|field| {
            if field.ident.is_none() {
                panic!("Field has no ident: {}", field.to_token_stream());
            }
            let mut f = Column {
                name: field.ident.as_ref().unwrap().to_string(),
                typ: String::new(),
                optional: false,
                primary_key: false,
            };

            // parse attributes
            field.attrs.iter().for_each(|attr| {
                if attr.path.get_ident().unwrap().eq("primarykey") {
                    f.primary_key = true;
                }
            });

            parse_type(&field.ty, &mut f);
            table.columns.push(f);
        }),
        unsupported => panic!("Unsupported fields type: {}", unsupported.to_token_stream()),
    }
}
///////////////////////////////////////////////////////////////////////////////
fn parse_type(ty: &Type, column: &mut Column) {
    match ty {
        Type::Path(path) => {
            let mut iter = path.path.segments.iter();
            // get first segment
            match iter.next() {
                Some(seg) => match seg.ident.to_string().as_str() {
                    "usize" => column.typ.push_str("INTEGER"),
                    "u32" => column.typ.push_str("INTEGER"),
                    "i32" => column.typ.push_str("INTEGER"),
                    "String" => column.typ.push_str("TEXT"),
                    "Option" => {
                        column.optional = true;
                        match &seg.arguments {
                            PathArguments::AngleBracketed(arg) => {
                                let mut args = arg.args.iter();
                                match args.next() {
                                    Some(arg) => match arg {
                                        GenericArgument::Type(ty2) => {
                                            parse_type(ty2, column);
                                        }
                                        unsupported => panic!(
                                            "Unsupported argument type: {}",
                                            unsupported.to_token_stream()
                                        ),
                                    },
                                    None => panic!(
                                        "No optional argument found: {}",
                                        arg.to_token_stream()
                                    ),
                                }
                            }
                            unsupported => panic!(
                                "Unsupported optional argument: {}",
                                unsupported.to_token_stream()
                            ),
                        }
                    }
                    unsupported => panic!("unsupported field data type: {}", unsupported),
                },
                None => panic!("No segment found: {}", path.to_token_stream()),
            }
        }
        unsupported => panic!("unsupported field type: {}", unsupported.to_token_stream()),
    }
}
///////////////////////////////////////////////////////////////////////////////
fn create_sql(table: &Table) -> String {
    let mut sql = String::new();
    sql.push_str("CREATE TABLE ");
    sql.push_str(&table.name.to_lowercase());
    sql.push_str("( ");
    let mut first_column = true;
    table.columns.iter().for_each(|column| {
        if !first_column {
            sql.push_str(", ");
        } else {
            first_column = false;
        }
        sql.push_str(&column.name);
        sql.push_str(" ");
        sql.push_str(&column.typ);
        if column.primary_key {
            sql.push_str(" PRIMARY KEY AUTOINCREMENT");
        } else if !column.optional {
            sql.push_str(" NOT NULL");
        }
    });
    sql.push_str(")");
    return sql;
}
///////////////////////////////////////////////////////////////////////////////
fn drop_sql(table: &Table) -> String {
    let mut sql = String::new();
    sql.push_str("DROP TABLE ");
    sql.push_str(&table.name.to_lowercase());
    return sql;
}
///////////////////////////////////////////////////////////////////////////////
#[proc_macro_derive(Create, attributes(primarykey))]
pub fn derive_create(input: TokenStream) -> TokenStream {
    //println!("{:#?}", input);
    let input = parse_macro_input!(input as DeriveInput);
    let struct_type = &input.ident;

    let mut table = Table {
        name: struct_type.to_string(),
        columns: Vec::new(),
    };

    match input.data {
        Data::Struct(data) => parse_struct(&data, &mut table),
        _unsupported => panic!("Unsupported data type is not a struct"),
    }

    let sql = create_sql(&table);
    let output = quote! {
        impl db::Create<rusqlite::Connection> for #struct_type {
            fn create(context: &rusqlite::Connection) -> Result<(), String> {
                context
                    .execute(
                        #sql,
                        [],
                    )
                    .or_else(|err| {
                        Err(format!("Error while executing sql {} ({})", #sql, err))
                    })?;
                Ok(())
            }
        }
    };

    println!("{}", output.to_string().as_str());
    output.into()
}
///////////////////////////////////////////////////////////////////////////////
#[proc_macro_derive(Drop)]
pub fn derive_drop(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_type = &input.ident;

    let mut table = Table {
        name: struct_type.to_string(),
        columns: Vec::new(),
    };

    match input.data {
        Data::Struct(data) => parse_struct(&data, &mut table),
        _unsupported => panic!("Unsupported data type is not a struct"),
    }

    let sql = drop_sql(&table);
    let output = quote! {
        impl db::Drop<rusqlite::Connection> for #struct_type {
            fn drop(context: &rusqlite::Connection) -> Result<(), String> {
                context
                    .execute(
                        #sql,
                        [],
                    )
                    .or_else(|err| {
                        Err(format!("Error while executing sql {} ({})", #sql, err))
                    })?;
                Ok(())
            }
        }
    };

    println!("{}", output.to_string().as_str());
    output.into()
}
