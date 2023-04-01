use crate::{
    descriptor::{extract_fields, FieldDesc},
    proto_type::{ProtoType, WireType},
};

use anyhow::Result;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::io::{self, ErrorKind, Read};
use syn::{Data, DataEnum, DataStruct, DeriveInput, LitInt};

pub trait Deserialize {
    fn deserialize_field(
        &mut self,
        prototype: ProtoType,
        wiretype: WireType,
        r: impl Read,
    ) -> io::Result<()>;

    fn deserialize(&mut self, r: impl Read) -> io::Result<()>;
}

pub fn read_tag(r: impl Read) -> io::Result<Option<(u64, WireType)>> {
    match read_uvarint(r) {
        Ok(tag) => {
            let ty = (tag & 0b00000111) as u8;
            let ty = WireType::try_from(ty)?;
            let id = tag >> 3;
            Ok(Some((id, ty)))
        }
        Err(error) => match error.kind() {
            ErrorKind::UnexpectedEof => Ok(None),
            _ => Err(error),
        },
    }
}

fn read_byte(r: impl Read) -> io::Result<u8> {
    let mut buf: [u8; 1] = [0];
    r.read_exact(&mut buf)?;
    Ok(buf[0])
}

pub fn read_uvarint(r: impl Read) -> io::Result<u64> {
    let mut n = 0u64;
    let mut shift = 0u8;
    let mut more = true;
    while more {
        let mut b = read_byte(r)?;
        more = (b & 0b10000000) != 0;
        b = b & 0b01111111;

        // TODO(klimt): Check that shift is valid.

        let m = (b as u64) << shift;
        shift += 7;

        n = n | m;
    }
    Ok(n)
}

fn read_ivarint(r: impl Read) -> io::Result<i64> {
    let n = read_uvarint(r)?;
    Ok(i64::from_le_bytes(n.to_le_bytes()))
}

fn decode_zigzag(n: u64) -> i64 {
    let neg = (n & 1) != 0;
    let mut n = n >> 1;
    if neg {
        n = n ^ !0;
    }
    i64::from_le_bytes(n.to_le_bytes())
}

// TODO(klimt): Do I need the proto definition to know if it's a signed field?
impl Deserialize for i32 {
    fn deserialize_field(
        &mut self,
        prototype: ProtoType,
        wiretype: WireType,
        r: impl Read,
    ) -> io::Result<()> {
        match wiretype {
            WireType::I32 => {
                let mut buffer: [u8; 4] = [0; 4];
                r.read_exact(&mut buffer)?;
                *self = i32::from_le_bytes(buffer);
                Ok(())
            }
            WireType::VarInt => {
                // TODO(klimt): Check for overflow.
                let n = read_ivarint(r)?;
                *self = n as i32;
                Ok(())
            }
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid wiretype for i32: {:?}", wiretype),
            )),
        }
    }

    fn deserialize(&mut self, r: impl Read) -> io::Result<()> {
        let n = read_ivarint(r)?;
        *self = n as i32;
        Ok(())
    }
}

/*
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
}*/

impl FieldDesc {
    fn deserialize_value_clause(&self) -> TokenStream {
        let ident = &self.name;
        let id = self.id;
        let ty = self.ty;
        quote! {
            #id => self.#ident.deserialize_field(#ty, wiretype, r)?
        }
    }
}

fn derive_deserialize_struct(name: Ident, data: DataStruct) -> Result<TokenStream> {
    let fields = extract_fields(data)?;

    let fields = fields
        .into_iter()
        .map(|field| field.deserialize_value_clause())
        .collect::<Vec<TokenStream>>();

    let out: TokenStream = quote! {
        #[automatically_derived]
        impl zombie::Deserialize for #name {
            fn deserialize_field(
                &mut self,
                prototype: zombie::ProtoType,
                wiretype: zombie::WireType,
                r: &mut impl std::io::Read
            ) -> std::io::Result<()> {
                let len = zombie::read_uvarint(r)?;
                let v = vec![0u8; len as usize];
                r.read_exact(&mut v)
                //self.deserialize(&mut v)
            }

            fn deserialize(&mut self, r: &mut impl std::io::Read) -> std::io::Result<()> {
                while let Some((id, wiretype)) = zombie::read_tag(r)? {
                    match id {
                        #(#fields),*,
                    }
                }
                Ok(())
            }
        }
    }
    .into();

    Ok(out)
}

/*
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
*/

pub fn derive_deserialize(input: DeriveInput) -> Result<TokenStream> {
    match input.data {
        Data::Struct(data) => derive_deserialize_struct(input.ident, data),
        //Data::Enum(data) => derive_deserialize_enum(input.ident, data),
        _ => panic!("![derive(Deserialize)] only works on structs"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uvarint_deserialize_zero() {
        let buf: Vec<u8> = vec![0];
        let n = read_uvarint(&mut &buf[..]).unwrap();
        assert_eq!(0, n);
    }

    #[test]
    fn uvarint_deserialize_one() {
        let buf: Vec<u8> = vec![1];
        let n = read_uvarint(&mut &buf[..]).unwrap();
        assert_eq!(1, n);
    }

    #[test]
    fn uvarint_deserialize_byte() {
        let buf: Vec<u8> = vec![127];
        let n = read_uvarint(&mut &buf[..]).unwrap();
        assert_eq!(127, n);
    }

    #[test]
    fn uvarint_deserialize_example() {
        let buf: Vec<u8> = vec![0x96, 0x01];
        let n = read_uvarint(&mut &buf[..]).unwrap();
        assert_eq!(150, n);
    }

    #[test]
    fn ivarint_deserialize_negative() {
        let buf: Vec<u8> = vec![0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01];
        let n = read_ivarint(&mut &buf[..]).unwrap();
        assert_eq!(-2, n);
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
