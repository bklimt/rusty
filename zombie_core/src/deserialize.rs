use crate::{
    descriptor::{extract_fields, FieldDesc},
    proto_type::{ProtoType, WireType},
};

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::{
    io::{self, ErrorKind, Read},
    string::FromUtf8Error,
};
use syn::{Data, DataEnum, DataStruct, DeriveInput};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeserializeError {
    #[error("io error")]
    IoError(#[from] io::Error),
    #[error("utf-8 error")]
    Utf8Error(#[from] FromUtf8Error),
    #[error("type error: `{0}`")]
    TypeError(String),
}

pub trait DeserializeField {
    fn deserialize_field(
        &mut self,
        prototype: ProtoType,
        wiretype: WireType,
        r: &mut impl Read,
    ) -> Result<(), DeserializeError>;
}

pub trait Deserialize {
    fn deserialize(&mut self, r: &mut impl Read) -> Result<(), DeserializeError>;
}

pub fn read_tag(r: &mut impl Read) -> Result<Option<(u64, WireType)>, DeserializeError> {
    match read_uvarint(r) {
        Ok(tag) => {
            let ty = (tag & 0b00000111) as u8;
            let ty = WireType::try_from(ty)?;
            let id = tag >> 3;
            Ok(Some((id, ty)))
        }
        Err(error) => match error.kind() {
            ErrorKind::UnexpectedEof => Ok(None),
            _ => Err(DeserializeError::IoError(error)),
        },
    }
}

fn read_byte(r: &mut impl Read) -> io::Result<u8> {
    let mut buf: [u8; 1] = [0];
    r.read_exact(&mut buf)?;
    Ok(buf[0])
}

pub fn read_uvarint(r: &mut impl Read) -> io::Result<u64> {
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

fn read_ivarint(r: &mut impl Read) -> Result<i64, DeserializeError> {
    let n = read_uvarint(r)?;
    Ok(i64::from_le_bytes(n.to_le_bytes()))
}

fn read_fixed_i32(r: &mut impl Read) -> Result<i32, DeserializeError> {
    let mut buffer: [u8; 4] = [0; 4];
    r.read_exact(&mut buffer)?;
    Ok(i32::from_le_bytes(buffer))
}

fn read_fixed_i64(r: &mut impl Read) -> Result<i64, DeserializeError> {
    let mut buffer: [u8; 8] = [0; 8];
    r.read_exact(&mut buffer)?;
    Ok(i64::from_le_bytes(buffer))
}

fn read_fixed_u32(r: &mut impl Read) -> Result<u32, DeserializeError> {
    let mut buffer: [u8; 4] = [0; 4];
    r.read_exact(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

fn read_fixed_u64(r: &mut impl Read) -> Result<u64, DeserializeError> {
    let mut buffer: [u8; 8] = [0; 8];
    r.read_exact(&mut buffer)?;
    Ok(u64::from_le_bytes(buffer))
}

pub fn read_i32(r: &mut impl Read, wiretype: WireType) -> Result<i32, DeserializeError> {
    match wiretype {
        WireType::I32 => read_fixed_i32(r),
        // TODO(klimt): Add bounds checking.
        WireType::I64 => Ok(read_fixed_i64(r)? as i32),
        WireType::VarInt => Ok(read_ivarint(r)? as i32),
        _ => Err(DeserializeError::TypeError(format!(
            "invalid wiretype for i32: {:?}",
            wiretype
        ))),
    }
}

pub fn read_i64(r: &mut impl Read, wiretype: WireType) -> Result<i64, DeserializeError> {
    match wiretype {
        WireType::I32 => Ok(read_fixed_i32(r)? as i64),
        WireType::I64 => read_fixed_i64(r),
        WireType::VarInt => read_ivarint(r),
        _ => Err(DeserializeError::TypeError(format!(
            "invalid wiretype for i64: {:?}",
            wiretype
        ))),
    }
}

pub fn read_u32(r: &mut impl Read, wiretype: WireType) -> Result<u32, DeserializeError> {
    match wiretype {
        WireType::I32 => read_fixed_u32(r),
        // TODO(klimt): Add bounds checking.
        WireType::I64 => Ok(read_fixed_u64(r)? as u32),
        WireType::VarInt => Ok(read_uvarint(r)? as u32),
        _ => Err(DeserializeError::TypeError(format!(
            "invalid wiretype for u32: {:?}",
            wiretype
        ))),
    }
}

pub fn read_u64(r: &mut impl Read, wiretype: WireType) -> Result<u64, DeserializeError> {
    match wiretype {
        WireType::I32 => Ok(read_fixed_u32(r)? as u64),
        WireType::I64 => read_fixed_u64(r),
        WireType::VarInt => Ok(read_uvarint(r)?),
        _ => Err(DeserializeError::TypeError(format!(
            "invalid wiretype for u64: {:?}",
            wiretype
        ))),
    }
}

fn read_int(
    r: &mut impl Read,
    wiretype: WireType,
    prototype: ProtoType,
) -> Result<i64, DeserializeError> {
    match prototype {
        ProtoType::Int32 | ProtoType::SFixed32 => Ok(read_i32(r, wiretype)? as i64),
        ProtoType::Int64 | ProtoType::SFixed64 => read_i64(r, wiretype),
        ProtoType::UInt32 | ProtoType::Fixed32 => Ok(read_u32(r, wiretype)? as i64),
        // TODO(klimt): Maybe do bounds-checking here instead.
        ProtoType::UInt64 | ProtoType::Fixed64 => Err(DeserializeError::TypeError(
            "attempted to read u64 field as signed".to_owned(),
        )),
        ProtoType::SInt32 | ProtoType::SInt64 => Ok(decode_zigzag(read_u64(r, wiretype)?)),
        ProtoType::Bool => read_i64(r, wiretype),
        ProtoType::Enum => read_i64(r, wiretype),
        _ => Err(DeserializeError::TypeError(format!(
            "attempted to read int value for {:?}",
            prototype
        ))),
    }
}

fn read_uint(
    r: &mut impl Read,
    wiretype: WireType,
    prototype: ProtoType,
) -> Result<u64, DeserializeError> {
    match prototype {
        ProtoType::Int32
        | ProtoType::SFixed32
        | ProtoType::Int64
        | ProtoType::SFixed64
        | ProtoType::SInt32
        | ProtoType::SInt64 => Err(DeserializeError::TypeError(format!(
            "attempted to read unsigned value for {:?}",
            prototype
        ))),
        ProtoType::UInt32 | ProtoType::Fixed32 => Ok(read_u32(r, wiretype)? as u64),
        ProtoType::UInt64 | ProtoType::Fixed64 => read_u64(r, wiretype),
        ProtoType::Bool => read_u64(r, wiretype),
        ProtoType::Enum => read_u64(r, wiretype),
        _ => Err(DeserializeError::TypeError(format!(
            "attempted to read uint value for {:?}",
            prototype
        ))),
    }
}

fn decode_zigzag(n: u64) -> i64 {
    let neg = (n & 1) != 0;
    let mut n = n >> 1;
    if neg {
        n = n ^ !0;
    }
    i64::from_le_bytes(n.to_le_bytes())
}

impl DeserializeField for i32 {
    fn deserialize_field(
        &mut self,
        prototype: ProtoType,
        wiretype: WireType,
        r: &mut impl Read,
    ) -> Result<(), DeserializeError> {
        // TODO(klimt): Bounds checking?
        *self = read_int(r, wiretype, prototype)? as i32;
        Ok(())
    }
}

impl DeserializeField for i64 {
    fn deserialize_field(
        &mut self,
        prototype: ProtoType,
        wiretype: WireType,
        r: &mut impl Read,
    ) -> Result<(), DeserializeError> {
        *self = read_int(r, wiretype, prototype)?;
        Ok(())
    }
}

impl DeserializeField for u32 {
    fn deserialize_field(
        &mut self,
        prototype: ProtoType,
        wiretype: WireType,
        r: &mut impl Read,
    ) -> Result<(), DeserializeError> {
        // TODO(klimt): Bounds checking?
        *self = read_uint(r, wiretype, prototype)? as u32;
        Ok(())
    }
}

impl DeserializeField for u64 {
    fn deserialize_field(
        &mut self,
        prototype: ProtoType,
        wiretype: WireType,
        r: &mut impl Read,
    ) -> Result<(), DeserializeError> {
        *self = read_uint(r, wiretype, prototype)?;
        Ok(())
    }
}

impl DeserializeField for bool {
    fn deserialize_field(
        &mut self,
        prototype: ProtoType,
        wiretype: WireType,
        r: &mut impl Read,
    ) -> Result<(), DeserializeError> {
        *self = read_uint(r, wiretype, prototype)? != 0;
        Ok(())
    }
}

pub fn read_float(r: &mut impl Read, wiretype: WireType) -> Result<f64, DeserializeError> {
    match wiretype {
        WireType::I32 => {
            let mut buffer: [u8; 4] = [0; 4];
            r.read_exact(&mut buffer)?;
            Ok(f32::from_le_bytes(buffer) as f64)
        }
        WireType::I64 => {
            let mut buffer: [u8; 8] = [0; 8];
            r.read_exact(&mut buffer)?;
            Ok(f64::from_le_bytes(buffer))
        }
        _ => Err(DeserializeError::TypeError(format!(
            "invalid wiretype for float: {:?}",
            wiretype
        ))),
    }
}

impl DeserializeField for f64 {
    fn deserialize_field(
        &mut self,
        _prototype: ProtoType,
        wiretype: WireType,
        r: &mut impl Read,
    ) -> Result<(), DeserializeError> {
        *self = read_float(r, wiretype)?;
        Ok(())
    }
}

impl DeserializeField for f32 {
    fn deserialize_field(
        &mut self,
        _prototype: ProtoType,
        wiretype: WireType,
        r: &mut impl Read,
    ) -> Result<(), DeserializeError> {
        *self = read_float(r, wiretype)? as f32;
        Ok(())
    }
}

pub fn read_len(r: &mut impl Read) -> io::Result<Vec<u8>> {
    let len = read_uvarint(r)?;
    let len = len as usize;
    let mut v = vec![0u8; len];
    r.read_exact(&mut v[..])?;
    Ok(v)
}

impl DeserializeField for String {
    fn deserialize_field(
        &mut self,
        _prototype: ProtoType,
        wiretype: WireType,
        r: &mut impl Read,
    ) -> Result<(), DeserializeError> {
        if let WireType::Len = wiretype {
            let v = read_len(r)?;
            *self = String::from_utf8(v)?;
            Ok(())
        } else {
            Err(DeserializeError::TypeError(format!(
                "invalid wiretype for string: {:?}",
                wiretype
            )))
        }
    }
}

impl DeserializeField for Vec<u8> {
    fn deserialize_field(
        &mut self,
        _prototype: ProtoType,
        wiretype: WireType,
        r: &mut impl Read,
    ) -> Result<(), DeserializeError> {
        if let WireType::Len = wiretype {
            *self = read_len(r)?;
            Ok(())
        } else {
            Err(DeserializeError::TypeError(format!(
                "invalid wiretype for Vec<u8>: {:?}",
                wiretype
            )))
        }
    }
}

impl<T: DeserializeField + Default> DeserializeField for Vec<T> {
    fn deserialize_field(
        &mut self,
        prototype: ProtoType,
        wiretype: WireType,
        r: &mut impl Read,
    ) -> Result<(), DeserializeError> {
        let mut item = T::default();
        item.deserialize_field(prototype, wiretype, r)?;
        self.push(item);
        Ok(())
    }
}

impl<T: DeserializeField + Default> DeserializeField for Option<T> {
    fn deserialize_field(
        &mut self,
        prototype: ProtoType,
        wiretype: WireType,
        r: &mut impl Read,
    ) -> Result<(), DeserializeError> {
        let mut item: T = T::default();
        item.deserialize_field(prototype, wiretype, r)?;
        *self = Some(item);
        Ok(())
    }
}

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

fn derive_deserialize_struct(name: Ident, data: DataStruct) -> anyhow::Result<TokenStream> {
    let fields = extract_fields(data)?;

    let fields = fields
        .into_iter()
        .map(|field| field.deserialize_value_clause())
        .collect::<Vec<TokenStream>>();

    let out: TokenStream = quote! {
        #[automatically_derived]
        impl zombie::DeserializeField for #name {
            fn deserialize_field(
                &mut self,
                prototype: zombie::ProtoType,
                wiretype: zombie::WireType,
                r: &mut impl std::io::Read
            ) -> Result<(), zombie::DeserializeError> {
                let len = zombie::read_uvarint(r)?;
                let len = len as usize;
                let mut v = vec![0u8; len];
                r.read_exact(&mut v[..])?;
                self.deserialize(&mut &v[..])
            }
        }

        impl zombie::Deserialize for #name {
            fn deserialize(&mut self, r: &mut impl std::io::Read) -> Result<(), zombie::DeserializeError> {
                while let Some((id, wiretype)) = zombie::read_tag(r)? {
                    match id {
                        #(#fields),*,
                        _ => {},
                    }
                }
                Ok(())
            }
        }
    }
    .into();

    Ok(out)
}

fn derive_deserialize_enum(name: Ident, _data: DataEnum) -> anyhow::Result<TokenStream> {
    let out: TokenStream = quote! {
        #[automatically_derived]
        impl zombie::DeserializeField for #name {
            fn deserialize_field(
                &mut self,
                prototype: zombie::ProtoType,
                wiretype: zombie::WireType,
                r: &mut impl std::io::Read,
            ) -> Result<(), zombie::DeserializeError> {
                let n = zombie::read_uvarint(r)?;
                *self = #name :: try_from(n)?;
                Ok(())
            }
        }
    }
    .into();

    Ok(out)
}

pub fn derive_deserialize(input: DeriveInput) -> anyhow::Result<TokenStream> {
    match input.data {
        Data::Struct(data) => derive_deserialize_struct(input.ident, data),
        Data::Enum(data) => derive_deserialize_enum(input.ident, data),
        _ => panic!("![derive(Deserialize)] only works on structs and enums"),
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

    #[test]
    fn read_exact_into_vec() {
        let src = vec![5u8, 4, 3, 2, 1, 0];
        let mut dst = vec![0u8; 4];
        (&src[..]).read_exact(&mut dst[..]).unwrap();
        assert_eq!(vec![5u8, 4, 3, 2], dst);
    }

    #[test]
    fn read_tag_works() {
        let buf = [0b1001010u8];
        let (id, wiretype) = read_tag(&mut &buf[..]).unwrap().unwrap();
        assert_eq!(9, id);
        assert_eq!(WireType::Len as i32, wiretype as i32);
    }
}
