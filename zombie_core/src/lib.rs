use anyhow::{anyhow, Result};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use std::io::{self, ErrorKind, Write};
use syn::{Data, DeriveInput, LitInt, Type};

#[derive(Clone, Copy, Debug)]
pub enum ProtoType {
    Int32,
    Int64,
    UInt32,
    UInt64,
    SInt32,
    SInt64,
    Bool,
    Enum,
    Fixed64,
    SFixed64,
    Double,
    String,
    Bytes,
    Message,
    Fixed32,
    SFixed32,
    Float,
}

pub enum WireType {
    VarInt = 0,
    I64 = 1,
    Len = 2,
    I32 = 3,
}

impl ProtoType {
    fn wiretype(&self) -> WireType {
        match self {
            Self::Int32
            | Self::Int64
            | Self::UInt32
            | Self::UInt64
            | Self::SInt32
            | Self::SInt64
            | Self::Bool
            | Self::Enum => WireType::VarInt,
            Self::Fixed64 | Self::SFixed64 | Self::Double => WireType::I64,
            Self::String | Self::Bytes | Self::Message => WireType::Len,
            Self::Fixed32 | Self::SFixed32 | Self::Float => WireType::I32,
        }
    }

    fn from_str(s: &str) -> Option<ProtoType> {
        match s {
            "int32" => Some(Self::Int32),
            "int64" => Some(Self::Int64),
            "uint32" => Some(Self::UInt32),
            "uint64" => Some(Self::UInt64),
            "sint32" => Some(Self::SInt32),
            "sint64" => Some(Self::SInt64),
            "bool" => Some(Self::Bool),
            "enum" => Some(Self::Enum),
            "fixed64" => Some(Self::Fixed64),
            "sfixed64" => Some(Self::SFixed64),
            "double" => Some(Self::Double),
            "string" => Some(Self::String),
            "bytes" => Some(Self::Bytes),
            "fixed32" => Some(Self::Fixed32),
            "sfixed32" => Some(Self::SFixed32),
            "float" => Some(Self::Float),
            _ => None,
        }
    }
}

impl ToTokens for ProtoType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote! { zombie::ProtoType:: });
        match *self {
            ProtoType::Int32 => tokens.extend(quote! { Int32 }),
            ProtoType::Int64 => tokens.extend(quote! { Int64 }),
            ProtoType::UInt32 => tokens.extend(quote! { UInt32 }),
            ProtoType::UInt64 => tokens.extend(quote! { UInt64 }),
            ProtoType::SInt32 => tokens.extend(quote! { SInt32 }),
            ProtoType::SInt64 => tokens.extend(quote! { SInt64 }),
            ProtoType::Bool => tokens.extend(quote! { Bool }),
            ProtoType::Enum => tokens.extend(quote! { Enum }),
            ProtoType::Fixed64 => tokens.extend(quote! { Fixed64 }),
            ProtoType::SFixed64 => tokens.extend(quote! { SFixed64 }),
            ProtoType::Double => tokens.extend(quote! { Double }),
            ProtoType::String => tokens.extend(quote! { String }),
            ProtoType::Bytes => tokens.extend(quote! { Bytes }),
            ProtoType::Message => tokens.extend(quote! { Message }),
            ProtoType::Fixed32 => tokens.extend(quote! { Fixed32 }),
            ProtoType::SFixed32 => tokens.extend(quote! { SFixed32 }),
            ProtoType::Float => tokens.extend(quote! { Float }),
        }
    }
}

pub trait Serialize {
    fn serialize_field(&self, id: u64, pbtype: ProtoType, w: &mut impl Write) -> io::Result<()>;
    fn serialize(&self, w: &mut impl Write) -> io::Result<()>;
}

pub fn write_tag(w: &mut impl Write, wiretype: WireType, id: u64) -> io::Result<()> {
    let ty = wiretype as u64;
    let tag = id << 3 | ty;
    write_uvarint(w, tag)
}

pub fn write_uvarint(w: &mut impl Write, n: u64) -> io::Result<()> {
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
    fn serialize_field(&self, id: u64, pbtype: ProtoType, w: &mut impl Write) -> io::Result<()> {
        match pbtype {
            ProtoType::Int32 | ProtoType::Int64 | ProtoType::UInt32 | ProtoType::UInt64 => {
                write_tag(w, WireType::VarInt, id)?;
                write_ivarint(w, i64::from(*self))
            }
            ProtoType::SInt32 | ProtoType::SInt64 => {
                write_tag(w, WireType::VarInt, id)?;
                write_uvarint(w, encode_zigzag(i64::from(*self)))
            }
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid pbtype for i32: {:?}", pbtype),
            )),
        }
    }

    fn serialize(&self, w: &mut impl Write) -> io::Result<()> {
        write_ivarint(w, i64::from(*self))
    }
}

impl Serialize for i64 {
    fn serialize_field(&self, id: u64, pbtype: ProtoType, w: &mut impl Write) -> io::Result<()> {
        write_tag(w, WireType::VarInt, id)?;
        write_ivarint(w, i64::from(*self))
    }

    fn serialize(&self, w: &mut impl Write) -> io::Result<()> {
        write_ivarint(w, i64::from(*self))
    }
}

impl Serialize for u32 {
    fn serialize_field(&self, id: u64, pbtype: ProtoType, w: &mut impl Write) -> io::Result<()> {
        write_tag(w, WireType::VarInt, id)?;
        write_uvarint(w, u64::from(*self))
    }

    fn serialize(&self, w: &mut impl Write) -> io::Result<()> {
        write_uvarint(w, u64::from(*self))
    }
}

impl Serialize for u64 {
    fn serialize_field(&self, id: u64, pbtype: ProtoType, w: &mut impl Write) -> io::Result<()> {
        write_tag(w, WireType::VarInt, id)?;
        write_uvarint(w, u64::from(*self))
    }

    fn serialize(&self, w: &mut impl Write) -> io::Result<()> {
        write_uvarint(w, u64::from(*self))
    }
}

#[derive(Debug)]
struct FieldDesc {
    id: u64,
    name: Ident,
    ty: ProtoType,
}

impl FieldDesc {
    fn serialize_value_call(&self) -> TokenStream {
        let id = self.id;
        let ident = &self.name;
        let ty = self.ty;
        quote! {
            self.#ident.serialize_field(#id, #ty, w)?
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

            let type_attr = field.attrs.iter().find(|attr| attr.path.is_ident("pbtype"));
            let type_attr = if let Some(attr) = type_attr {
                let id: Ident = attr.parse_args()?;
                let s = id.to_string();
                let pt =
                    ProtoType::from_str(&s).ok_or_else(|| anyhow!("invalid proto type {}", s))?;
                Some(pt)
            } else {
                None
            };

            let type_inferred: ProtoType = match field.ty.clone() {
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
                    } else if path.path.is_ident("i64") {
                        Ok(ProtoType::Int64)
                    } else if path.path.is_ident("u32") {
                        Ok(ProtoType::UInt32)
                    } else if path.path.is_ident("u64") {
                        Ok(ProtoType::UInt64)
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

            let ty = type_attr.unwrap_or(type_inferred);

            let id_attr = field
                .attrs
                .iter()
                .find(|attr| attr.path.is_ident("id"))
                .ok_or_else(|| anyhow!("no id attribute for field {}", ident.to_string()))?;

            let sid: LitInt = id_attr.parse_args()?;
            let id: u64 = sid.base10_parse()?;
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
            fn serialize_field(&self, id: u64, pbtype: zombie::ProtoType, w: &mut impl std::io::Write) -> std::io::Result<()> {
                zombie::write_tag(w, zombie::WireType::Len, id)?;
                let mut v = Vec::new();
                self.serialize(&mut v)?;
                zombie::write_uvarint(w, v.len() as u64)?;
                w.write_all(&v[..])
            }

            fn serialize(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
                #(#fields);*;
                std::io::Result::Ok(())
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
