use anyhow::{anyhow, Result};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{GenericArgument, Path, Type};

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

fn get_param_type(ty: &str, path: &Path) -> Option<Type> {
    if path.segments.len() != 1 {
        return None;
    }
    let first = path.segments.first().unwrap();
    if first.ident.to_string() != ty {
        return None;
    }
    match &first.arguments {
        syn::PathArguments::AngleBracketed(args) => {
            if args.args.len() != 1 {
                return None;
            }
            match args.args.first().unwrap() {
                GenericArgument::Type(arg) => Some(arg.clone()),
                _ => None,
            }
        }
        _ => None,
    }
}

fn get_vec_type(path: &Path) -> Option<Type> {
    get_param_type("Vec", path)
}

fn get_option_type(path: &Path) -> Option<Type> {
    get_param_type("Option", path)
}

pub fn infer_proto_type(ty: &Type) -> Result<ProtoType> {
    match ty.clone() {
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
            } else if path.path.is_ident("bool") {
                Ok(ProtoType::Bool)
            } else if path.path.is_ident("f32") {
                Ok(ProtoType::Fixed32)
            } else if path.path.is_ident("f64") {
                Ok(ProtoType::Fixed64)
            } else if path.path.is_ident("String") {
                Ok(ProtoType::String)
            } else if path.path.is_ident("str") {
                Ok(ProtoType::String)
            } else if let Some(vec_type) = get_vec_type(&path.path) {
                if let Type::Path(vec_type_path) = &vec_type {
                    if vec_type_path.path.is_ident("u8") {
                        Ok(ProtoType::Bytes)
                    } else {
                        infer_proto_type(&vec_type)
                    }
                } else {
                    infer_proto_type(&vec_type)
                }
            } else if let Some(opt_type) = get_option_type(&path.path) {
                infer_proto_type(&opt_type)
            } else {
                // We have to assume this is some type that can handle itself.
                Ok(ProtoType::Other)
            }
        }
        Type::Ptr(_) => Err(anyhow!("unsupported type: ptr")),
        Type::Reference(r) => infer_proto_type(r.elem.as_ref()),
        Type::Slice(s) => {
            if let Type::Path(path) = s.elem.as_ref() {
                if path.path.is_ident("u8") {
                    Ok(ProtoType::Bytes)
                } else {
                    Err(anyhow!(
                        "unsupported slice type: {:?}",
                        path.path.to_token_stream()
                    ))
                }
            } else {
                Err(anyhow!("unsupported type: slice"))
            }
        }
        Type::TraitObject(_) => Err(anyhow!("unsupported type: trait object")),
        Type::Tuple(_) => Err(anyhow!("unsupported type: tuple")),
        Type::Verbatim(_) => Err(anyhow!("unsupported type: verbatim")),
        _ => Err(anyhow!(
            "unsupported type: {}",
            ty.to_token_stream().to_string()
        )),
    }
}
