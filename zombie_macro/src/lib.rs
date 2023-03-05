use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(Serialize, attributes(id))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    struct FieldDesc {
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
                fields.push(FieldDesc { name, path, tokens });
            }
        }
    } else {
        panic!("![derive(Serialize)] only works on structs");
    }

    let field_str = fields
        .iter()
        .map(|f: &FieldDesc| {
            format!(
                "{{name: {}, path: {}, tokens: {}}}",
                f.name, f.path, f.tokens
            )
        })
        .collect::<Vec<String>>()
        .join(",\n");

    let name = input.ident;

    quote! {
        #[automatically_derived]
        impl zombie::Serialize for #name {
            fn serialize(&self) { println!("serialized: {}", #field_str); }
        }
    }
    .into()
}
