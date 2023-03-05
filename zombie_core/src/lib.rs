use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, LitInt};

pub trait Serialize {
    fn serialize(&self);
}

pub fn derive_serialize(input: DeriveInput) -> Result<TokenStream> {
    struct FieldDesc {
        id: usize,
        name: String,
        path: String,
        tokens: String,
    }
    let mut fields = Vec::new();

    if let Data::Struct(data) = input.data {
        for field in data.fields.iter() {
            for attr in field.attrs.iter() {
                let name = field
                    .ident
                    .as_ref()
                    .map_or("<unnamed>".to_string(), |i| i.to_string());
                let path = attr
                    .path
                    .get_ident()
                    .map_or("<no path>".to_string(), |i| i.to_string());
                let tokens = attr.tokens.to_string();
                let sid: LitInt = attr.parse_args()?;
                let id: usize = sid.base10_parse()?;
                fields.push(FieldDesc {
                    id,
                    name,
                    path,
                    tokens,
                });
            }
        }
    } else {
        panic!("![derive(Serialize)] only works on structs");
    }

    let field_str = fields
        .iter()
        .map(|f: &FieldDesc| {
            format!(
                "{{id: {}, name: {}, path: {}, tokens: {}}}",
                f.id, f.name, f.path, f.tokens
            )
        })
        .collect::<Vec<String>>()
        .join(",\n");

    let name = input.ident;

    let out: TokenStream = quote! {
        #[automatically_derived]
        impl zombie::Serialize for #name {
            fn serialize(&self) { println!("serialized: {}", #field_str); }
        }
    }
    .into();

    Ok(out)
}
