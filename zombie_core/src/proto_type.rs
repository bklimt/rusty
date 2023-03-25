use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

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
    Other,
}

pub enum WireType {
    VarInt = 0,
    I64 = 1,
    Len = 2,
    I32 = 5,
}

impl ProtoType {
    pub fn from_str(s: &str) -> Option<ProtoType> {
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
            ProtoType::Other => tokens.extend(quote! { Other }),
        }
    }
}
