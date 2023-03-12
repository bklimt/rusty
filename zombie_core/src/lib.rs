use anyhow::{anyhow, Result};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use std::io::{self, Write};
use syn::{Data, DeriveInput, LitInt, Type};

pub trait Serialize {
    fn serialize(&self);
}

fn write_varint(w: &mut impl Write, n: u64) -> io::Result<()> {
    let mut buf: [u8; 10] = [0; 10];
    let mut i = 0usize;
    loop {
        let a = (n & 0b01111111) as u8;
        let n = n >> 7;
        buf[i] = a;
        i += 1;
        if n == 0 {
            break;
        }
    }
    w.write_all(&buf[0..i])
}

impl Serialize for i32 {
    fn serialize(&self) {
        println!("serializing i32 {}", self);
    }
}

#[derive(Clone, Copy, Debug)]
enum ProtoType {
    Int32,
}

#[derive(Debug)]
struct FieldDesc {
    id: usize,
    name: Ident,
    ty: ProtoType,
}

impl FieldDesc {
    fn serialize_value_call(&self) -> TokenStream {
        let ident = &self.name;
        quote! {
            self.#ident.serialize()
        }
    }
}

pub fn derive_serialize(input: DeriveInput) -> Result<TokenStream> {
    let mut fields = Vec::new();

    if let Data::Struct(data) = input.data {
        for field in data.fields.iter() {
            let ident = field
                .ident
                .as_ref()
                .ok_or_else(|| anyhow!("no ident for field"))?
                .clone();
            let ty: ProtoType = match field.ty.clone() {
                Type::Array(_) => Err(anyhow!("unsupported type: array")),
                Type::BareFn(_) => Err(anyhow!("unsupported type: bare fn")),
                Type::Group(_) => Err(anyhow!("unsupported type: group")),
                Type::ImplTrait(_) => Err(anyhow!("unsupported type: impl trait")),
                Type::Infer(_) => Err(anyhow!("unsupported type: infer")),
                Type::Macro(_) => Err(anyhow!("unsupported type: macro")),
                Type::Never(_) => Err(anyhow!("unsupported type: never")),
                Type::Paren(_) => Err(anyhow!("unsupported type: paren")),
                Type::Path(path) => {
                    if path.path.is_ident("i32") {
                        Ok(ProtoType::Int32)
                    } else {
                        Err(anyhow!("unsupported type: path"))
                    }
                }
                Type::Ptr(_) => Err(anyhow!("unsupported type: ptr")),
                Type::Reference(_) => Err(anyhow!("unsupported type: reference")),
                Type::Slice(_) => Err(anyhow!("unsupported type: slice")),
                Type::TraitObject(_) => Err(anyhow!("unsupported type: trait object")),
                Type::Tuple(_) => Err(anyhow!("unsupported type: tuple")),
                Type::Verbatim(_) => Err(anyhow!("unsupported type: verbatim")),
                _ => Err(anyhow!(
                    "unsupported type: {}",
                    field.ty.to_token_stream().to_string()
                )),
            }?;

            let id_attr = field
                .attrs
                .iter()
                .find(|attr| attr.path.is_ident("id"))
                .ok_or_else(|| anyhow!("no id attribute for field {}", ident.to_string()))?;

            let sid: LitInt = id_attr.parse_args()?;
            let id: usize = sid.base10_parse()?;
            fields.push(FieldDesc {
                id,
                name: ident.clone(),
                ty: ty.clone(),
            });
        }
    } else {
        panic!("![derive(Serialize)] only works on structs");
    }

    let fields = fields
        .into_iter()
        .map(|field| field.serialize_value_call())
        .collect::<Vec<TokenStream>>();

    let name = input.ident;

    let out: TokenStream = quote! {
        #[automatically_derived]
        impl zombie::Serialize for #name {
            fn serialize(&self) {
                #(#fields);*
            }
        }
    }
    .into();

    Ok(out)
}
