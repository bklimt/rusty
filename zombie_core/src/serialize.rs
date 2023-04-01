use crate::{
    descriptor::{extract_fields, FieldDesc},
    proto_type::{ProtoType, WireType},
};

use anyhow::Result;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::io::{self, ErrorKind, Write};
use syn::{Data, DataEnum, DataStruct, DeriveInput};

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

impl Serialize for i32 {
    fn serialize_field(&self, id: u64, pbtype: ProtoType, w: &mut impl Write) -> io::Result<()> {
        match pbtype {
            ProtoType::Int32 => {
                write_tag(w, WireType::VarInt, id)?;
                write_ivarint(w, i64::from(*self))
            }
            ProtoType::SInt32 => {
                write_tag(w, WireType::VarInt, id)?;
                write_uvarint(w, encode_zigzag(i64::from(*self)))
            }
            ProtoType::SFixed32 => {
                write_tag(w, WireType::I32, id)?;
                w.write_all(&self.to_le_bytes())
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
        match pbtype {
            ProtoType::Int64 => {
                write_tag(w, WireType::VarInt, id)?;
                write_ivarint(w, i64::from(*self))
            }
            ProtoType::SInt64 => {
                write_tag(w, WireType::VarInt, id)?;
                write_uvarint(w, encode_zigzag(i64::from(*self)))
            }
            ProtoType::SFixed64 => {
                write_tag(w, WireType::I64, id)?;
                w.write_all(&self.to_le_bytes())
            }
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid pbtype for i64: {:?}", pbtype),
            )),
        }
    }

    fn serialize(&self, w: &mut impl Write) -> io::Result<()> {
        write_ivarint(w, i64::from(*self))
    }
}

impl Serialize for u32 {
    fn serialize_field(&self, id: u64, pbtype: ProtoType, w: &mut impl Write) -> io::Result<()> {
        match pbtype {
            ProtoType::UInt32 => {
                write_tag(w, WireType::VarInt, id)?;
                write_uvarint(w, u64::from(*self))
            }
            ProtoType::Fixed32 => {
                write_tag(w, WireType::I32, id)?;
                w.write_all(&self.to_le_bytes())
            }
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid pbtype for u32: {:?}", pbtype),
            )),
        }
    }

    fn serialize(&self, w: &mut impl Write) -> io::Result<()> {
        write_uvarint(w, u64::from(*self))
    }
}

impl Serialize for u64 {
    fn serialize_field(&self, id: u64, pbtype: ProtoType, w: &mut impl Write) -> io::Result<()> {
        match pbtype {
            ProtoType::UInt64 => {
                write_tag(w, WireType::VarInt, id)?;
                write_uvarint(w, u64::from(*self))
            }
            ProtoType::Fixed64 => {
                write_tag(w, WireType::I64, id)?;
                w.write_all(&self.to_le_bytes())
            }
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid pbtype for u64: {:?}", pbtype),
            )),
        }
    }

    fn serialize(&self, w: &mut impl Write) -> io::Result<()> {
        write_uvarint(w, u64::from(*self))
    }
}

impl Serialize for bool {
    fn serialize_field(&self, id: u64, _pbtype: ProtoType, w: &mut impl Write) -> io::Result<()> {
        write_tag(w, WireType::VarInt, id)?;
        write_uvarint(w, if *self { 1 } else { 0 })
    }

    fn serialize(&self, w: &mut impl Write) -> io::Result<()> {
        write_uvarint(w, if *self { 1 } else { 0 })
    }
}

impl Serialize for f64 {
    fn serialize_field(&self, id: u64, _pbtype: ProtoType, w: &mut impl Write) -> io::Result<()> {
        write_tag(w, WireType::I64, id)?;
        w.write_all(&self.to_le_bytes())
    }

    fn serialize(&self, w: &mut impl Write) -> io::Result<()> {
        w.write_all(&self.to_le_bytes())
    }
}

impl Serialize for f32 {
    fn serialize_field(&self, id: u64, _pbtype: ProtoType, w: &mut impl Write) -> io::Result<()> {
        write_tag(w, WireType::I32, id)?;
        w.write_all(&self.to_le_bytes())
    }

    fn serialize(&self, w: &mut impl Write) -> io::Result<()> {
        w.write_all(&self.to_le_bytes())
    }
}

impl Serialize for String {
    fn serialize_field(&self, id: u64, _pbtype: ProtoType, w: &mut impl Write) -> io::Result<()> {
        write_tag(w, WireType::Len, id)?;
        self.serialize(w)
    }

    fn serialize(&self, w: &mut impl Write) -> io::Result<()> {
        write_uvarint(w, self.len() as u64)?;
        w.write_all(self.as_bytes())
    }
}

impl Serialize for str {
    fn serialize_field(&self, id: u64, _pbtype: ProtoType, w: &mut impl Write) -> io::Result<()> {
        write_tag(w, WireType::Len, id)?;
        self.serialize(w)
    }

    fn serialize(&self, w: &mut impl Write) -> io::Result<()> {
        write_uvarint(w, self.len() as u64)?;
        w.write_all(self.as_bytes())
    }
}

impl Serialize for Vec<u8> {
    fn serialize_field(&self, id: u64, _pbtype: ProtoType, w: &mut impl Write) -> io::Result<()> {
        write_tag(w, WireType::Len, id)?;
        self.serialize(w)
    }

    fn serialize(&self, w: &mut impl Write) -> io::Result<()> {
        write_uvarint(w, self.len() as u64)?;
        w.write_all(&self[..])
    }
}

impl Serialize for &[u8] {
    fn serialize_field(&self, id: u64, _pbtype: ProtoType, w: &mut impl Write) -> io::Result<()> {
        write_tag(w, WireType::Len, id)?;
        self.serialize(w)
    }

    fn serialize(&self, w: &mut impl Write) -> io::Result<()> {
        write_uvarint(w, self.len() as u64)?;
        w.write_all(&self)
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize_field(&self, id: u64, pbtype: ProtoType, w: &mut impl Write) -> io::Result<()> {
        for item in self.iter() {
            item.serialize_field(id, pbtype, w)?;
        }
        Ok(())
    }

    fn serialize(&self, w: &mut impl Write) -> io::Result<()> {
        for item in self.iter() {
            item.serialize(w)?;
        }
        Ok(())
    }
}

impl<T: Serialize> Serialize for Option<T> {
    fn serialize_field(&self, id: u64, pbtype: ProtoType, w: &mut impl Write) -> io::Result<()> {
        match &self {
            Some(val) => val.serialize_field(id, pbtype, w),
            None => Ok(()),
        }
    }

    fn serialize(&self, w: &mut impl Write) -> io::Result<()> {
        match &self {
            Some(val) => val.serialize(w),
            None => Ok(()),
        }
    }
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

fn derive_serialize_struct(name: Ident, data: DataStruct) -> Result<TokenStream> {
    let fields = extract_fields(data)?;

    let fields = fields
        .into_iter()
        .map(|field| field.serialize_value_call())
        .collect::<Vec<TokenStream>>();

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

fn derive_serialize_enum(name: Ident, _data: DataEnum) -> Result<TokenStream> {
    let out: TokenStream = quote! {
        #[automatically_derived]
        impl zombie::Serialize for #name {
            fn serialize_field(&self, id: u64, pbtype: zombie::ProtoType, w: &mut impl std::io::Write) -> std::io::Result<()> {
                zombie::write_tag(w, zombie::WireType::VarInt, id)?;
                self.serialize(w)
            }

            fn serialize(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
                zombie::write_uvarint(w, self.clone() as u64)?;
                std::io::Result::Ok(())
            }
        }
    }
    .into();

    Ok(out)
}

pub fn derive_serialize(input: DeriveInput) -> Result<TokenStream> {
    match input.data {
        Data::Struct(data) => derive_serialize_struct(input.ident, data),
        Data::Enum(data) => derive_serialize_enum(input.ident, data),
        _ => panic!("![derive(Serialize)] only works on structs and enums"),
    }
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
    fn write_tag_works() {
        let mut buf: Vec<u8> = Vec::new();
        write_tag(&mut buf, WireType::Len, 9).unwrap();
        assert_eq!(buf, vec![0b1001010]);
    }
}
