use anyhow::{anyhow, Result};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use std::io::{self, Write};
use syn::{Data, DeriveInput, LitInt, Type};

pub trait Serialize {
    fn serialize(&self);
}

fn write_uvarint(w: &mut impl Write, n: u64) -> io::Result<()> {
    let mut buf: [u8; 10] = [0; 10];
    let mut i = 0usize;
    let mut n = n;
    loop {
        let mut a = (n & 0b01111111) as u8;
        n = n >> 7;
        if n != 0 {
            a = a | 0b10000000;
        }
        buf[i] = a;
        i += 1;
        if n == 0 {
            break;
        }
    }
    w.write_all(&buf[0..i])
}

fn write_ivarint(w: &mut impl Write, n: i64) -> io::Result<()> {
    write_uvarint(w, u64::from_le_bytes(n.to_le_bytes()))
}

fn encode_zigzag(n: i64) -> u64 {
    let neg = n < 0;
    let mut n = n << 1;
    if neg {
        n = n ^ !0;
    }
    u64::from_le_bytes(n.to_le_bytes())
}

fn decode_zigzag(n: u64) -> i64 {
    let neg = (n & 1) != 0;
    let mut n = n >> 1;
    if neg {
        n = n ^ !0;
    }
    i64::from_le_bytes(n.to_le_bytes())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uvarint_serialize_zero() {
        let mut buf: Vec<u8> = Vec::new();
        write_uvarint(&mut buf, 0).unwrap();
        assert_eq!(buf, vec![0]);
    }

    #[test]
    fn uvarint_serialize_one() {
        let mut buf: Vec<u8> = Vec::new();
        write_uvarint(&mut buf, 1).unwrap();
        assert_eq!(buf, vec![1]);
    }

    #[test]
    fn uvarint_serialize_byte() {
        let mut buf: Vec<u8> = Vec::new();
        write_uvarint(&mut buf, 127).unwrap();
        assert_eq!(buf, vec![127]);
    }

    #[test]
    fn uvarint_serialize_example() {
        let mut buf: Vec<u8> = Vec::new();
        write_uvarint(&mut buf, 150).unwrap();
        assert_eq!(buf, vec![0x96, 0x01]);
    }

    #[test]
    fn ivarint_serialize_negative() {
        let mut buf: Vec<u8> = Vec::new();
        write_ivarint(&mut buf, -2).unwrap();
        assert_eq!(
            buf,
            vec![0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01]
        );
    }

    #[test]
    fn encode_sint_works() {
        assert_eq!(0, encode_zigzag(0));
        assert_eq!(1, encode_zigzag(-1));
        assert_eq!(2, encode_zigzag(1));
        assert_eq!(3, encode_zigzag(-2));
        assert_eq!(4, encode_zigzag(2));
        assert_eq!(0xfffffffe, encode_zigzag(0x7fffffff));
        assert_eq!(0xffffffff, encode_zigzag(-0x80000000));
    }

    #[test]
    fn decode_sint_works() {
        assert_eq!(0, decode_zigzag(0));
        assert_eq!(-1, decode_zigzag(1));
        assert_eq!(1, decode_zigzag(2));
        assert_eq!(-2, decode_zigzag(3));
        assert_eq!(2, decode_zigzag(4));
        assert_eq!(0x7fffffff, decode_zigzag(0xfffffffe));
        assert_eq!(-0x80000000, decode_zigzag(0xffffffff));
    }
}
